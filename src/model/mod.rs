use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::ModelError;

pub mod rig_provider;
pub mod smoke_provider;

pub use rig_provider::RigModelClient;
pub use smoke_provider::SmokeModelClient;

#[async_trait]
pub trait ModelClient: Send + Sync {
    fn metadata(&self) -> ModelMetadata {
        ModelMetadata::unknown()
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError>;

    async fn complete_stream(
        &self,
        request: ModelRequest,
    ) -> Result<ModelStreamResponse, ModelError> {
        let response = self.complete(request).await?;
        Ok(ModelStreamResponse::from_response(response))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub provider: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

impl ModelMetadata {
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            reasoning_effort: None,
        }
    }

    pub fn with_reasoning_effort(mut self, reasoning_effort: Option<String>) -> Self {
        self.reasoning_effort = reasoning_effort.filter(|value| !value.trim().is_empty());
        self
    }

    pub fn unknown() -> Self {
        Self::new("unknown", "unknown")
    }
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
    pub usage: Option<ModelUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStreamResponse {
    pub chunks: Vec<ModelStreamChunk>,
    pub response: ModelResponse,
}

impl ModelStreamResponse {
    pub fn from_response(response: ModelResponse) -> Self {
        let chunks = chunk_text(&response.text, 96)
            .into_iter()
            .enumerate()
            .map(|(index, text)| ModelStreamChunk {
                index: index as u32,
                is_final: false,
                text,
            })
            .collect::<Vec<_>>();
        let mut chunks = if chunks.is_empty() {
            vec![ModelStreamChunk {
                index: 0,
                text: String::new(),
                is_final: true,
            }]
        } else {
            chunks
        };
        if let Some(last) = chunks.last_mut() {
            last.is_final = true;
        }

        Self { chunks, response }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStreamChunk {
    pub index: u32,
    pub text: String,
    pub is_final: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    OpenAi,
    DeepSeek,
    Smoke,
}

impl ModelProvider {
    pub fn parse(value: &str) -> Result<Self, ModelError> {
        match value.to_ascii_lowercase().as_str() {
            "openai" => Ok(Self::OpenAi),
            "deepseek" => Ok(Self::DeepSeek),
            "smoke" | "local" | "offline" => Ok(Self::Smoke),
            _ => Err(ModelError::UnsupportedProvider(value.to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::DeepSeek => "deepseek",
            Self::Smoke => "smoke",
        }
    }
}

fn chunk_text(text: &str, chunk_chars: usize) -> Vec<String> {
    let chunk_chars = chunk_chars.max(1);
    let mut chunks = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if current.chars().count() >= chunk_chars {
            chunks.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}
