use std::{collections::HashMap, ops::Deref, path::PathBuf};

use futures::stream::{self, StreamExt};

use crate::{
    context::Context,
    entity::types::{
        file::{CreateFileProps, File},
        file_embedding::{CreateFileEmbeddingProps, FileEmbedding},
    },
    services::embeddings::FileFragment,
};

use super::{embeddings::EmbeddingsService, files::FilesService};

pub struct IndexerService {
    embeddings: EmbeddingsService,
    files: FilesService,
    context: Context,
}

impl IndexerService {
    pub fn try_new(context: Context, root_dir: PathBuf) -> anyhow::Result<Self> {
        let embeddings = EmbeddingsService::try_new()?;
        Ok(IndexerService {
            context,
            embeddings,
            files: FilesService::new(root_dir),
        })
    }

    pub async fn index_files(&self, paths: &[PathBuf]) -> anyhow::Result<()> {
        // Get file hashes for each path.
        let hashes = self.files.hash_files(paths);

        // Update file hashes in file table
        let changed_paths = File::create_many(
            &self.context,
            hashes
                .into_iter()
                .map(|(path, hash)| {
                    CreateFileProps::builder()
                        .path(path.to_path_buf())
                        .hash(hash)
                        .build()
                })
                .collect(),
        )
        .await?
        .into_iter()
        .map(|x| x.path.0.to_path_buf())
        .collect::<Vec<_>>();

        if changed_paths.is_empty() {
            return Ok(());
        }

        // Get embeddings for each file
        let fragments_to_index = stream::iter(changed_paths)
            .filter_map(|x| async move {
                match self.embeddings.parse_file(&x).await {
                    Ok(parsed) => Some(async move { (x, parsed) }),
                    Err(_) => None,
                }
            })
            .boxed()
            .buffer_unordered(10)
            .collect::<HashMap<_, _>>()
            .await;

        let embedding_texts = fragments_to_index
            .iter()
            .flat_map(|(path, text)| {
                text.iter().map(|x| match x {
                    FileFragment::Heading(inside) => (inside.to_string(), path.to_path_buf()),
                    FileFragment::Paragraph(inside) => (inside.to_string(), path.to_path_buf()),
                })
            })
            .collect::<HashMap<_, _>>();

        let embeddings_map = self
            .embeddings
            .embeddings(&embedding_texts.keys().cloned().collect::<Vec<_>>()[..])
            .await?;

        FileEmbedding::create_many(
            &self.context,
            embedding_texts
                .iter()
                .filter_map(|(text, path)| {
                    embeddings_map.get(text.deref()).map(|embedding| {
                        CreateFileEmbeddingProps::builder()
                            .embedding(embedding.to_owned())
                            .file_path(path.to_path_buf())
                            .contents(text.to_string())
                            .build()
                    })
                })
                .collect(),
        )
        .await?;

        Ok(())
    }
}
