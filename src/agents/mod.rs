use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::NovelId;
use crate::error::AgentError;
use crate::model::ModelClient;

pub type ModelHandle = Arc<dyn ModelClient>;
pub type MemoryHandle = Arc<dyn AgentMemory>;
pub type StorageHandle = Arc<dyn AgentStorage>;

#[async_trait]
pub trait NovelAgent: Send + Sync {
    fn role(&self) -> AgentRole;

    async fn run(
        &self,
        ctx: AgentContext,
        input: AgentInput,
    ) -> Result<AgentOutput, AgentError>;
}

pub trait AgentMemory: Send + Sync {}
pub trait AgentStorage: Send + Sync {}

#[derive(Clone)]
pub struct AgentContext {
    pub novel_id: Option<NovelId>,
    pub memory: Option<MemoryHandle>,
    pub model: ModelHandle,
    pub storage: Option<StorageHandle>,
    pub constraints: AgentConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConstraints {
    pub target_platform: Option<String>,
    pub max_retries: u32,
    pub require_json: bool,
}

impl Default for AgentConstraints {
    fn default() -> Self {
        Self {
            target_platform: None,
            max_retries: 2,
            require_json: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    pub task: AgentTask,
    pub prompt: String,
    pub payload: Value,
    pub context: Vec<ContextItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub role: AgentRole,
    pub structured: Value,
    pub raw_text: String,
    pub parse_error: Option<String>,
    pub artifacts: Vec<AgentArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub kind: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentArtifact {
    pub kind: String,
    pub name: String,
    pub data: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    Orchestrator,
    Market,
    Plot,
    Character,
    Worldbuilding,
    Writer,
    Continuity,
    Style,
    Reviewer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentTask {
    CreateNovel,
    GenerateOutline,
    GenerateChapter,
    ReviewChapter,
    RewriteChapter,
    ExtractFacts,
    PolishStyle,
    CheckContinuity,
}
