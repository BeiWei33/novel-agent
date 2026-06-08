use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::ModelError;

pub mod rig_provider;

pub use rig_provider::RigModelClient;

#[async_trait]
pub trait ModelClient: Send + Sync {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    pub system_prompt: Option<String>,
    pub prompt: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub text: String,
    pub raw: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    OpenAi,
}

impl ModelProvider {
    pub fn parse(value: &str) -> Result<Self, ModelError> {
        match value.to_ascii_lowercase().as_str() {
            "openai" => Ok(Self::OpenAi),
            _ => Err(ModelError::UnsupportedProvider(value.to_string())),
        }
    }
}
