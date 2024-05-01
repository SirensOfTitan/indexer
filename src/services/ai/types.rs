use candle_core::{Device, Tensor};
use candle_transformers::{
    generation::LogitsProcessor,
    models::llama::{self, Llama},
};
use tokenizers::Tokenizer;
use typed_builder::TypedBuilder;

use super::{MAX_TOKENS, REPEAT_LAST_N, REPEAT_PENALTY};

#[derive(TypedBuilder)]
pub struct InferStateProcessors<'a> {
    tokenizer: &'a Tokenizer,
    llama: &'a Llama,
    cache: &'a mut llama::Cache,
    device: &'a Device,

    eos_token_id: Option<u32>,

    #[builder(default = LogitsProcessor::new(299792458, Some(0.7), Some(0.8)))]
    logits_processor: LogitsProcessor,
}

#[derive(TypedBuilder)]
pub struct InferState<'a> {
    processors: InferStateProcessors<'a>,
    status: InferStateStatus,
}

pub enum InferStateStatus {
    Active(ActiveInferStateStatus),
    Done,
}

/// Used to hold state for an inferrence stream.
#[derive(TypedBuilder)]
pub struct ActiveInferStateStatus {
    /// The encoded tokens in the request and response.
    pub tokens: Vec<u32>,

    #[builder(default = 0)]
    pub index_pos: usize,

    #[builder(default = 0)]
    pub tokens_generated: usize,
}

impl<'a> InferState<'a> {
    fn transition(&mut self) -> anyhow::Result<()> {
        self.status = match self.status {
            InferStateStatus::Done => InferStateStatus::Done,
            InferStateStatus::Active(ref active) => {
                if active.tokens_generated >= MAX_TOKENS {
                    InferStateStatus::Done
                } else if active.tokens.last() == self.processors.eos_token_id.as_ref() {
                    InferStateStatus::Done
                } else {
                    let (context_size, context_index) =
                        if self.processors.cache.use_kv_cache && active.index_pos > 0 {
                            (1, active.index_pos)
                        } else {
                            (active.tokens.len(), 0)
                        };

                    let ctxt = &active.tokens[active.tokens.len().saturating_sub(context_size)..];
                    let input = Tensor::new(ctxt, self.processors.device)?.unsqueeze(0)?;

                    let logits = self.processors.llama.forward(
                        &input,
                        context_index,
                        self.processors.cache,
                    )?;
                    let logits = logits.squeeze(0)?;
                    let logits = {
                        let start_at = active.tokens.len().saturating_sub(REPEAT_LAST_N);
                        candle_transformers::utils::apply_repeat_penalty(
                            &logits,
                            REPEAT_PENALTY,
                            &active.tokens[start_at..],
                        )?
                    };

                    let next_token = self.processors.logits_processor.sample(&logits)?;

                    let mut new_tokens = active.tokens.clone();
                    new_tokens.push(next_token);

                    InferStateStatus::Active(
                        ActiveInferStateStatus::builder()
                            .tokens(new_tokens)
                            .index_pos(active.index_pos + ctxt.len())
                            .tokens_generated(active.tokens_generated + 1)
                            .build(),
                    )
                }
            }
        };

        Ok(())
    }

    /// Transitions itself to its next state and emits the next token in the set.
    pub fn next(&mut self) -> Option<anyhow::Result<String>> {
        let _ = self.transition();
        match self.status {
            InferStateStatus::Active(ref active) => {
                let last_token = active.tokens.last();
                if let Some(last_token) = last_token {
                    Some(
                        self.processors
                            .tokenizer
                            .decode(&[*last_token], true)
                            .map_err(anyhow::Error::msg),
                    )
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
