use std::{path::PathBuf};

use async_trait::async_trait;
use clap::{Args, Parser, Subcommand};
use futures::StreamExt;



use crate::{context::Context, entity::types::file::{CreateFileProps, File}, services::files::FilesService};

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
    types: Vec<String>
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

                let svc = FilesService::new(&args.watch_path);

                let files = svc.read_tree().await?;
                let files_with_hashes = svc.hash_files(&files.into_iter().map(|x| x.path()).collect::<Vec<_>>());


                File::create_many(
                    &context,
                    files_with_hashes
                        .into_iter()
                        .map(|x| CreateFileProps::builder().path(x.path).hash(x.hash).build())
                        .collect()
                ).await?;

                println!("OK, inserted");


                let (_debouncer, mut rx) = svc.watch(&args.watch_path)?;
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
                            let files_with_hashes = svc.hash_files(&files);
                            File::create_many(
                                &context,
                                files_with_hashes
                                    .into_iter()
                                    .map(|x| CreateFileProps::builder().path(x.path).hash(x.hash).build())
                                    .collect()
                            ).await?;
                        }
                    }
                }
            }

            Commands::FindFiles(ref args) => {
                let svc = FilesService::new(&args.path);
                let files = svc.read_tree().await?
                    .into_iter()
                    .filter(|x| {
                        if let Some(extension) = x.path().extension().and_then(|ext| ext.to_str()) {
                            args.types.contains(&extension.to_string())
                        } else {
                            false
                        }
                    })
                    .collect::<Vec<_>>();

                let files_with_hashes = svc.hash_files(&files.into_iter().map(|x| x.path()).collect::<Vec<_>>());

                for file in files_with_hashes {
                    if let Some(path) = file.path.to_str() {
                        println!("{}\t{}", path, hex::encode(&file.hash));
                    }
                }

                Ok(())
            }
        }
    }
}
