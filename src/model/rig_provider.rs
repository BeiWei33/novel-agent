use async_trait::async_trait;
use rig_core::client::{CompletionClient, ProviderClient};
use rig_core::completion::Prompt;
use rig_core::providers::{deepseek, openai};
use serde_json::json;

use super::{ModelClient, ModelProvider, ModelRequest, ModelResponse};
use crate::error::ModelError;

#[derive(Debug, Clone)]
pub struct RigModelClient {
    provider: ModelProvider,
    model: String,
    reasoning_effort: Option<String>,
}

impl RigModelClient {
    pub fn new(provider: ModelProvider, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
            reasoning_effort: None,
        }
    }

    pub fn with_reasoning_effort(mut self, reasoning_effort: Option<String>) -> Self {
        self.reasoning_effort = reasoning_effort.filter(|value| !value.trim().is_empty());
        self
    }
}

#[async_trait]
impl ModelClient for RigModelClient {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        match self.provider {
            ModelProvider::OpenAi => self.complete_openai(request).await,
            ModelProvider::DeepSeek => self.complete_deepseek(request).await,
            ModelProvider::Smoke => unreachable!("smoke provider is handled by SmokeModelClient"),
        }
    }
}

impl RigModelClient {
    async fn complete_openai(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let client = openai::Client::from_env().map_err(|source| ModelError::Provider {
            provider: "openai".to_string(),
            message: source.to_string(),
        })?;

        let mut builder = client.agent(self.model.as_str());
        if let Some(system_prompt) = request.system_prompt {
            builder = builder.preamble(&system_prompt);
        }
        if let Some(temperature) = request.temperature {
            builder = builder.temperature(f64::from(temperature));
        }
        if let Some(max_tokens) = request.max_tokens {
            builder = builder.max_tokens(u64::from(max_tokens));
        }
        if let Some(reasoning_effort) = &self.reasoning_effort {
            builder = builder.additional_params(json!({
                "reasoning_effort": reasoning_effort
            }));
        }

        let agent = builder.build();
        let text = agent
            .prompt(request.prompt.as_str())
            .await
            .map_err(|source| ModelError::Provider {
                provider: "openai".to_string(),
                message: source.to_string(),
            })?;

        Ok(ModelResponse {
            raw: text.clone(),
            text,
            usage: None,
        })
    }

    async fn complete_deepseek(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let client = deepseek::Client::from_env().map_err(|source| ModelError::Provider {
            provider: "deepseek".to_string(),
            message: source.to_string(),
        })?;

        let mut builder = client.agent(self.model.as_str());
        if let Some(system_prompt) = request.system_prompt {
            builder = builder.preamble(&system_prompt);
        }
        if let Some(temperature) = request.temperature {
            builder = builder.temperature(f64::from(temperature));
        }
        if let Some(max_tokens) = request.max_tokens {
            builder = builder.max_tokens(u64::from(max_tokens));
        }

        let agent = builder.build();
        let text = agent
            .prompt(request.prompt.as_str())
            .await
            .map_err(|source| ModelError::Provider {
                provider: "deepseek".to_string(),
                message: source.to_string(),
            })?;

        Ok(ModelResponse {
            raw: text.clone(),
            text,
            usage: None,
        })
    }
}
