use crate::platform::home_dir;
use serde::Deserialize;

#[derive(Default, Deserialize, Debug)]
pub struct Config {
    pub huggingface_token: Option<String>,
}

impl Config {
    pub async fn load() -> Self {
        let path = home_dir().join(".indexer.toml");

        if let Ok(file_contents) = tokio::fs::read_to_string(path).await {
           let config = toml::from_str(&file_contents);

            if let Ok(config) = config {
                return config;
            }
        }

        Self::default()
    }
}
