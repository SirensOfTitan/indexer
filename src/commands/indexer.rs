use std::path::PathBuf;

use async_trait::async_trait;
use clap::{Args, Parser, Subcommand};

use crate::services::files::FilesService;

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
        match self.command {
            Commands::Run(ref args) => {

                let svc = FilesService::new(&args.watch_path);
                let files = svc.read_tree().await?;
                println!("{:?}", files);


                Ok(())
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

                let files_with_hashes = svc.hash_files(&files);

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
