use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub model: ModelConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig {
                provider: "openai".to_string(),
                model: "gpt-5".to_string(),
            },
            storage: StorageConfig {
                database_url: "sqlite://novel-agent.db".to_string(),
            },
        }
    }
}

impl AppConfig {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&content)?)
    }
}
