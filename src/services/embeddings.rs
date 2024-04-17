use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use dataloader::{non_cached::Loader, BatchFn};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use once_cell::sync::Lazy;
use regex::Regex;

static HEADING_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\*+\s+(.+)$").unwrap());

struct EmbeddingBatchFn(TextEmbedding);

impl BatchFn<String, Vec<f32>> for EmbeddingBatchFn {
    async fn load(&mut self, keys: &[String]) -> HashMap<String, Vec<f32>> {
        let results = self
            .0
            .embed(keys.to_vec(), Some(keys.len()))
            .unwrap_or_default();

        keys.iter()
            .zip(results)
            .map(|(key, emb)| (key.to_owned(), emb.to_owned()))
            .collect::<HashMap<_, _>>()
    }
}

pub struct EmbeddingsService {
    loader: Loader<String, Vec<f32>, EmbeddingBatchFn>,
}

#[derive(Debug)]
pub enum FileFragment {
    Heading(String),
    Paragraph(String),
}

impl EmbeddingsService {
    pub fn try_new() -> anyhow::Result<Self> {
        let model = TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::BGESmallENV15,
            show_download_progress: true,
            ..Default::default()
        })?;

        let batch_fn = EmbeddingBatchFn(model);
        Ok(EmbeddingsService {
            loader: Loader::new(batch_fn).with_max_batch_size(256),
        })
    }

    pub async fn parse_file(&self, file: &PathBuf) -> anyhow::Result<Vec<FileFragment>> {
        let file_contents = tokio::fs::read_to_string(&file).await?;

        Ok(file_contents
            .lines()
            .map(|line| {
                if let Some(captures) = HEADING_REGEX.captures(line) {
                    FileFragment::Heading(captures[0].to_string())
                } else {
                    FileFragment::Paragraph(line.to_string())
                }
            })
            .fold(vec![], |mut acc, x| {
                if let Some(FileFragment::Paragraph(last_contents)) = acc.last_mut() {
                    if let FileFragment::Paragraph(contents) = x {
                        last_contents.push('\n');
                        last_contents.push_str(&contents);
                    } else {
                        acc.push(x);
                    }
                } else {
                    acc.push(x);
                }

                acc
            }))
    }

    pub async fn embeddings<'b>(
        &self,
        texts: &[String],
    ) -> anyhow::Result<HashMap<String, Vec<f32>>> {
        self.loader
            .try_load_many(texts.to_vec())
            .await
            .context("Could not load from dataloader.")
    }

    pub async fn embedding(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        self.loader
            .try_load(text.to_string())
            .await
            .context("Could not load from dataloader.")
    }
}
