use std::{fs::File, path::{Path, PathBuf}};

use rayon::prelude::*;

use async_recursion::async_recursion;
use sha2::{Digest, Sha256};
use tokio::fs::DirEntry;
use std::io;

pub struct HashFileResult {
    pub path: PathBuf,
    pub hash: Vec<u8>,
}

/// A service that deals with operations against files
pub struct FilesService<'a> {
    root_dir: &'a PathBuf,
}

impl<'a> FilesService<'a> {
    pub fn new(root_dir: &'a PathBuf) -> FilesService<'a> {
        FilesService {
            root_dir
        }
    }
    /// Given a root directory, will read the entire file tree recursively
    /// under it.
    pub async fn read_tree(&self) -> anyhow::Result<Vec<DirEntry>> {
        self.visit_dirs(self.root_dir).await
    }

    #[async_recursion]
    /// Given a directory, will return all files under it.
    async fn visit_dirs(&self, dir: &Path) -> anyhow::Result<Vec<DirEntry>> {
        if !dir.is_dir() {
            Ok(vec![])
        } else {
            let mut current_dir = tokio::fs::read_dir(&dir).await?;

            let mut result = vec![];
            while let Some(entry) = current_dir.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    let mut sub_results = self.visit_dirs(&path).await?;
                    result.append(&mut sub_results);
                } else {
                    result.push(entry);
                }
            }

            Ok(result)
        }
    }

    fn hash_file(&self, path: PathBuf) -> anyhow::Result<HashFileResult> {
        let mut file = File::open(&path)?;
        let mut sha256 = Sha256::new();
        io::copy(&mut file, &mut sha256)?;

        Ok(HashFileResult {
            path,
            hash: sha256.finalize().to_vec(),
        })
    }

    /// Given a list of files, will return hashes of each file result.
    pub fn hash_files(&self, files: &[DirEntry]) -> Vec<HashFileResult> {
        // Just ensure everything passed is a file, to be defensive.
        let file_paths = files
            .iter()
            .filter_map(|file| {
                let path = file.path();
                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        file_paths
            .par_iter()
            .filter_map(|path| self.hash_file(path.to_path_buf()).ok())
            .collect::<Vec<_>>()
    }
}
