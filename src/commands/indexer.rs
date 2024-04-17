use std::path::PathBuf;

use async_trait::async_trait;
use clap::{Args, Parser, Subcommand};
use futures::StreamExt;

use crate::{
    context::Context,
    services::{files::FilesService, indexer::IndexerService},
};

use super::Executor;

#[derive(Args, Debug)]
struct RunArgs {
    #[arg(env, long)]
    /// The path to index.
    watch_path: PathBuf,
}

#[derive(Args, Debug)]
struct FindFilesArgs {
    #[arg(long)]
    /// The root path to find files within.
    path: PathBuf,

    #[arg(long, default_value = "org,md", value_delimiter = ',')]
    /// File types to find.
    types: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run(RunArgs),

    /// Finds all files of type and prints them to console.
    /// For debugging.
    FindFiles(FindFilesArgs),
}

#[derive(Parser, Debug)]
pub struct Indexer {
    #[command(subcommand)]
    command: Commands,
}

#[async_trait]
impl Executor for Indexer {
    async fn execute(&self) -> anyhow::Result<()> {
        let context = Context::default();
        match self.command {
            Commands::Run(ref args) => {
                let files_svc = FilesService::new(args.watch_path.to_path_buf());
                let files = files_svc.read_tree().await?;

                // Always do a full reindexing on startup.
                let indexer_svc = IndexerService::try_new(context, args.watch_path.to_path_buf())?;
                println!("Performing full reindexing...");
                indexer_svc
                    .index_files(&files.into_iter().map(|x| x.path()).collect::<Vec<_>>()[..])
                    .await?;
                println!("OK, inserted...");

                let (_debouncer, mut rx) = files_svc.watch(&args.watch_path)?;
                loop {
                    while let Some(res) = rx.next().await {
                        if let Ok(events) = res {
                            let files = events
                                .into_iter()
                                .filter_map(|x| if x.path.is_file() { Some(x.path) } else { None })
                                .collect::<Vec<_>>();

                            if files.is_empty() {
                                continue;
                            }

                            println!("Updating {:?}", files);
                            let _ = indexer_svc.index_files(&files[..]).await;
                        }
                    }
                }
            }

            Commands::FindFiles(ref args) => {
                let svc = FilesService::new(args.path.to_path_buf());
                let files = svc
                    .read_tree()
                    .await?
                    .into_iter()
                    .filter(|x| {
                        if let Some(extension) = x.path().extension().and_then(|ext| ext.to_str()) {
                            args.types.contains(&extension.to_string())
                        } else {
                            false
                        }
                    })
                    .collect::<Vec<_>>();

                let paths = files.into_iter().map(|x| x.path()).collect::<Vec<_>>();
                let files_with_hashes = svc.hash_files(&paths[..]);

                for (path, hash) in files_with_hashes {
                    println!("{}\t{}", path.display(), hex::encode(&hash));
                }

                Ok(())
            }
        }
    }
}
