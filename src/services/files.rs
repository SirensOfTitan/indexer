use notify::{FsEventWatcher, RecursiveMode};
use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    time::Duration,
};

use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use rayon::prelude::*;

use async_recursion::async_recursion;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt,
};
use sha2::{Digest, Sha256};
use std::io;
use tokio::fs::DirEntry;

#[async_recursion]
/// Given a directory, will return all files under it.
async fn visit_dirs(dir: &Path) -> anyhow::Result<Vec<DirEntry>> {
    if !dir.is_dir() {
        Ok(vec![])
    } else {
        let mut current_dir = tokio::fs::read_dir(&dir).await?;

        let mut result = vec![];
        while let Some(entry) = current_dir.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let mut sub_results = visit_dirs(&path).await?;
                result.append(&mut sub_results);
            } else {
                result.push(entry);
            }
        }

        Ok(result)
    }
}

/// A service that deals with operations against files
pub struct FilesService {
    root_dir: PathBuf,
}

type DebouncedEventReceiver = Receiver<notify::Result<Vec<DebouncedEvent>>>;

impl<'a> FilesService {
    pub fn new(root_dir: PathBuf) -> FilesService {
        FilesService { root_dir }
    }
    /// Given a root directory, will read the entire file tree recursively
    /// under it.
    pub async fn read_tree(&self) -> anyhow::Result<Vec<DirEntry>> {
        visit_dirs(&self.root_dir).await
    }

    fn hash_file<'b>(&self, path: &'b PathBuf) -> anyhow::Result<(&'b PathBuf, Vec<u8>)> {
        let mut file = File::open(path)?;
        let mut sha256 = Sha256::new();
        io::copy(&mut file, &mut sha256)?;

        Ok((path, sha256.finalize().to_vec()))
    }

    /// Given a list of files, will return hashes of each file result.
    pub fn hash_files<'b>(&self, files: &'b [PathBuf]) -> HashMap<&'b PathBuf, Vec<u8>> {
        // Just ensure everything passed is a file, to be defensive.
        let file_paths = files.iter().filter(|x| x.is_file()).collect::<Vec<_>>();

        file_paths
            .par_iter()
            .filter_map(|path| self.hash_file(path).ok())
            .collect::<HashMap<_, _>>()
    }

    pub fn watch<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> notify::Result<(Debouncer<FsEventWatcher>, DebouncedEventReceiver)> {
        let (mut tx, rx) = channel(1);

        // Debouncer MUST not be dropped for watching to persist.
        let mut debouncer = new_debouncer(Duration::from_secs(4), move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        })?;

        debouncer
            .watcher()
            .watch(path.as_ref(), RecursiveMode::Recursive)?;

        Ok((debouncer, rx))
    }
}
