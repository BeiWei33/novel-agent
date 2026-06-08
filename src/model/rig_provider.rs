use async_trait::async_trait;
use rig_core::client::{CompletionClient, ProviderClient};
use rig_core::completion::Prompt;
use rig_core::providers::openai;

use super::{ModelClient, ModelProvider, ModelRequest, ModelResponse};
use crate::error::ModelError;

#[derive(Debug, Clone)]
pub struct RigModelClient {
    provider: ModelProvider,
    model: String,
}

impl RigModelClient {
    pub fn new(provider: ModelProvider, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
        }
    }
}

#[async_trait]
impl ModelClient for RigModelClient {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        match self.provider {
            ModelProvider::OpenAi => self.complete_openai(request).await,
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
            builder = builder.preamble(system_prompt);
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
        })
    }
}
