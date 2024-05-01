use candle_core::{DType, Device};
use candle_nn::VarBuilder;
use candle_transformers::models::llama as model;
use candle_transformers::models::llama::{Llama, LlamaConfig};
use hf_hub::{api::tokio::ApiBuilder, Repo, RepoType};
use tokenizers::Tokenizer;

use crate::config::Config;
use crate::platform::cache_dir;

use super::{
    types::{ActiveInferStateStatus, InferState, InferStateProcessors, InferStateStatus},
    utils::{choose_device, hub_load_safetensors},
    USE_FLASH_ATTN,
};

#[derive(Debug)]
pub struct AIService {
    llama: Llama,
    cache: model::Cache,
    tokenizer: Tokenizer,
    device: Device,
}

impl AIService {
    pub async fn try_new() -> anyhow::Result<Self> {
        let config = Config::load().await;
        let api = ApiBuilder::new()
            .with_token(config.huggingface_token)
            .with_cache_dir(cache_dir())
            .build()?;

        let api = api.repo(Repo::with_revision(
            "meta-llama/Meta-Llama-3-8B-Instruct".to_string(),
            RepoType::Model,
            "main".to_string(),
        ));

        let tokenizer_filename = api.get("tokenizer.json").await?;
        let config_filename = api.get("config.json").await?;

        let config: LlamaConfig = serde_json::from_slice(&std::fs::read(config_filename)?)?;
        let config = config.into_config(USE_FLASH_ATTN);

        let model_files = hub_load_safetensors(&api, "model.safetensors.index.json").await?;

        let device = choose_device()?;
        let vb =
            unsafe { VarBuilder::from_mmaped_safetensors(&model_files, DType::BF16, &device)? };

        Ok(AIService {
            device: device.clone(),
            llama: Llama::load(vb, &config)?,
            cache: model::Cache::new(true, DType::BF16, &config, &device)?,
            tokenizer: Tokenizer::from_file(tokenizer_filename).map_err(anyhow::Error::msg)?,
        })
    }

    pub fn infer<'a>(
        &'a mut self,
        prompt: &'a str,
    ) -> anyhow::Result<impl futures::Stream<Item = anyhow::Result<String>> + 'a> {
        let tokens = self
            .tokenizer
            .encode(prompt, true)
            .map_err(anyhow::Error::msg)?
            .get_ids()
            .to_vec();

        let processors = InferStateProcessors::builder()
            .tokenizer(&self.tokenizer)
            .eos_token_id(self.tokenizer.token_to_id("<|eot_id|>"))
            .llama(&self.llama)
            .cache(&mut self.cache)
            .device(&self.device)
            .build();

        Ok(futures::stream::unfold(
            InferState::builder()
                .processors(processors)
                .status(InferStateStatus::Active(
                    ActiveInferStateStatus::builder().tokens(tokens).build(),
                ))
                .build(),
            move |mut state| {
                Box::pin(async move {
                    let next_token = state.next();
                    next_token.map(|x| (x, state))
                })
            },
        ))
    }
}
