use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::domain::NovelId;
use crate::error::AgentError;
use crate::model::{ModelClient, ModelRequest};

pub type ModelHandle = Arc<dyn ModelClient>;
pub type MemoryHandle = Arc<dyn AgentMemory>;
pub type StorageHandle = Arc<dyn AgentStorage>;

#[async_trait]
pub trait NovelAgent: Send + Sync {
    fn role(&self) -> AgentRole;

    async fn run(&self, ctx: AgentContext, input: AgentInput) -> Result<AgentOutput, AgentError>;
}

pub trait AgentMemory: Send + Sync {}
pub trait AgentStorage: Send + Sync {}

pub struct PromptAgent {
    role: AgentRole,
    system_prompt: &'static str,
}

impl PromptAgent {
    pub fn new(role: AgentRole, system_prompt: &'static str) -> Self {
        Self {
            role,
            system_prompt,
        }
    }
}

#[async_trait]
impl NovelAgent for PromptAgent {
    fn role(&self) -> AgentRole {
        self.role
    }

    async fn run(&self, ctx: AgentContext, input: AgentInput) -> Result<AgentOutput, AgentError> {
        let payload = serde_json::to_string_pretty(&input.payload)
            .map_err(|err| AgentError::InvalidOutput(err.to_string()))?;
        let context = serde_json::to_string_pretty(&input.context)
            .map_err(|err| AgentError::InvalidOutput(err.to_string()))?;
        let prompt = format!(
            "任务: {}\n{}\n\n输入 JSON:\n{}\n\n补充上下文 JSON:\n{}",
            input.task.as_str(),
            input.prompt,
            payload,
            context
        );
        let response = ctx
            .model
            .complete(ModelRequest {
                system_prompt: Some(self.system_prompt.to_string()),
                prompt,
                temperature: Some(0.7),
                max_tokens: None,
            })
            .await?;

        Ok(parse_agent_output(self.role, response.text))
    }
}

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
    pub raw_notes: String,
    pub attempt: u32,
    pub will_fallback: bool,
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

impl AgentRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Orchestrator => "orchestrator",
            Self::Market => "market",
            Self::Plot => "plot",
            Self::Character => "character",
            Self::Worldbuilding => "worldbuilding",
            Self::Writer => "writer",
            Self::Continuity => "continuity",
            Self::Style => "style",
            Self::Reviewer => "reviewer",
        }
    }
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

impl AgentTask {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CreateNovel => "create_novel",
            Self::GenerateOutline => "generate_outline",
            Self::GenerateChapter => "generate_chapter",
            Self::ReviewChapter => "review_chapter",
            Self::RewriteChapter => "rewrite_chapter",
            Self::ExtractFacts => "extract_facts",
            Self::PolishStyle => "polish_style",
            Self::CheckContinuity => "check_continuity",
        }
    }
}

fn parse_agent_output(role: AgentRole, raw_text: String) -> AgentOutput {
    let candidate = strip_code_fence(raw_text.trim());
    match serde_json::from_str::<Value>(candidate) {
        Ok(value) => {
            let structured = value
                .get("structured")
                .cloned()
                .unwrap_or_else(|| value.clone());
            let raw_notes = value
                .get("raw_notes")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();

            AgentOutput {
                role,
                structured,
                raw_text,
                parse_error: None,
                raw_notes,
                attempt: 1,
                will_fallback: false,
                artifacts: vec![],
            }
        }
        Err(err) => AgentOutput {
            role,
            structured: json!({}),
            raw_text,
            parse_error: Some(err.to_string()),
            raw_notes: String::new(),
            attempt: 1,
            will_fallback: false,
            artifacts: vec![],
        },
    }
}

fn strip_code_fence(value: &str) -> &str {
    let Some(stripped) = value.strip_prefix("```") else {
        return value;
    };
    let stripped = stripped.strip_prefix("json").unwrap_or(stripped);
    stripped
        .trim_start_matches(|ch| ch == '\r' || ch == '\n')
        .strip_suffix("```")
        .map(str::trim)
        .unwrap_or(value)
}
