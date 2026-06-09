use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::domain::NovelId;
use crate::error::{AgentError, ModelError};
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
        let response = tokio::time::timeout(
            model_call_timeout(self.role, input.task),
            ctx.model.complete(ModelRequest {
                system_prompt: Some(self.system_prompt.to_string()),
                prompt,
                temperature: Some(default_temperature(self.role, input.task)),
                max_tokens: Some(default_max_tokens(self.role, input.task)),
            }),
        )
        .await
        .map_err(|_| {
            AgentError::Model(ModelError::Provider {
                provider: "model".to_string(),
                message: format!(
                    "{} {} timed out after {} seconds",
                    self.role.as_str(),
                    input.task.as_str(),
                    model_call_timeout(self.role, input.task).as_secs()
                ),
            })
        })??;

        let mut output = parse_agent_output(self.role, response.text);
        output.token_usage = response.usage;
        Ok(output)
    }
}

fn default_temperature(role: AgentRole, task: AgentTask) -> f32 {
    match (role, task) {
        (AgentRole::Reviewer | AgentRole::Continuity, _) => 0.3,
        (
            AgentRole::Market | AgentRole::Plot | AgentRole::Character | AgentRole::Worldbuilding,
            _,
        ) => 0.6,
        _ => 0.7,
    }
}

fn default_max_tokens(role: AgentRole, task: AgentTask) -> u32 {
    match (role, task) {
        (AgentRole::Market, AgentTask::CreateNovel) => 2_500,
        (AgentRole::Plot, AgentTask::GenerateOutline) => 6_500,
        (AgentRole::Character, AgentTask::CreateNovel) => 3_000,
        (AgentRole::Worldbuilding, AgentTask::CreateNovel) => 2_800,
        (AgentRole::Writer, AgentTask::GenerateChapter | AgentTask::RewriteChapter) => 7_000,
        (AgentRole::Style, AgentTask::PolishStyle) => 7_000,
        (AgentRole::Continuity, _) => 2_500,
        (AgentRole::Reviewer, _) => 3_000,
        _ => 4_000,
    }
}

fn model_call_timeout(role: AgentRole, task: AgentTask) -> Duration {
    let seconds = match (role, task) {
        (AgentRole::Character, AgentTask::CreateNovel) => 360,
        (AgentRole::Plot, AgentTask::GenerateOutline)
        | (AgentRole::Writer, AgentTask::GenerateChapter | AgentTask::RewriteChapter)
        | (AgentRole::Style, AgentTask::PolishStyle) => 300,
        _ => 240,
    };
    Duration::from_secs(seconds)
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
    pub duration_ms: Option<u64>,
    pub token_usage: Option<crate::model::ModelUsage>,
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
    let candidate = extract_json_candidate(raw_text.trim());
    match serde_json::from_str::<Value>(candidate) {
        Ok(value) => match parse_agent_output_envelope(role, &value) {
            Ok((structured, raw_notes)) => AgentOutput {
                role,
                structured,
                raw_text,
                parse_error: None,
                raw_notes,
                attempt: 1,
                will_fallback: false,
                duration_ms: None,
                token_usage: None,
                artifacts: vec![],
            },
            Err(err) => AgentOutput {
                role,
                structured: json!({}),
                raw_text,
                parse_error: Some(err),
                raw_notes: String::new(),
                attempt: 1,
                will_fallback: false,
                duration_ms: None,
                token_usage: None,
                artifacts: vec![],
            },
        },
        Err(err) => AgentOutput {
            role,
            structured: json!({}),
            raw_text,
            parse_error: Some(err.to_string()),
            raw_notes: String::new(),
            attempt: 1,
            will_fallback: false,
            duration_ms: None,
            token_usage: None,
            artifacts: vec![],
        },
    }
}

fn extract_json_candidate(value: &str) -> &str {
    let value = value.trim().trim_start_matches('\u{feff}');
    if let Some(fenced) = fenced_json_candidate(value) {
        return fenced;
    }

    first_balanced_json_object(value).unwrap_or(value)
}

fn parse_agent_output_envelope(
    expected_role: AgentRole,
    value: &Value,
) -> Result<(Value, String), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "AgentOutput envelope must be a JSON object".to_string())?;
    let role = object
        .get("role")
        .and_then(Value::as_str)
        .ok_or_else(|| "AgentOutput envelope missing string field `role`".to_string())?;
    if role != expected_role.as_str() {
        return Err(format!(
            "AgentOutput envelope role `{role}` does not match expected `{}`",
            expected_role.as_str()
        ));
    }

    let structured = object
        .get("structured")
        .ok_or_else(|| "AgentOutput envelope missing field `structured`".to_string())?;
    if !structured.is_object() {
        return Err("AgentOutput envelope field `structured` must be an object".to_string());
    }
    validate_structured_payload(expected_role, structured)?;

    let raw_notes = object
        .get("raw_notes")
        .and_then(Value::as_str)
        .ok_or_else(|| "AgentOutput envelope missing string field `raw_notes`".to_string())?;

    Ok((structured.clone(), raw_notes.to_string()))
}

fn validate_structured_payload(role: AgentRole, structured: &Value) -> Result<(), String> {
    match role {
        AgentRole::Market => {
            require_object_field(structured, "market_analysis")?;
            require_array_field(structured, "title_candidates")?;
            require_object_field(structured, "opening_strategy")?;
            require_object_field(structured, "platform_profile")?;
        }
        AgentRole::Plot => {
            require_object_field(structured, "plot_plan")?;
            require_array_field(structured, "chapter_outlines")?;
        }
        AgentRole::Character => {
            require_array_field(structured, "characters")?;
        }
        AgentRole::Worldbuilding => {
            require_object_field(structured, "world_setting")?;
            require_array_field(structured, "facts_to_seed")?;
        }
        AgentRole::Writer => {
            require_object_field(structured, "chapter_draft")?;
        }
        AgentRole::Continuity => {
            require_object_field(structured, "continuity_report")?;
        }
        AgentRole::Style => {
            require_object_field(structured, "styled_chapter")?;
        }
        AgentRole::Reviewer => {
            require_object_field(structured, "review_report")?;
        }
        AgentRole::Orchestrator => {}
    }

    Ok(())
}

fn require_object_field(value: &Value, key: &str) -> Result<(), String> {
    if value.get(key).is_some_and(Value::is_object) {
        Ok(())
    } else {
        Err(format!(
            "AgentOutput structured missing required object field `{key}`"
        ))
    }
}

fn require_array_field(value: &Value, key: &str) -> Result<(), String> {
    if value.get(key).is_some_and(Value::is_array) {
        Ok(())
    } else {
        Err(format!(
            "AgentOutput structured missing required array field `{key}`"
        ))
    }
}

fn fenced_json_candidate(value: &str) -> Option<&str> {
    let Some(stripped) = value.strip_prefix("```") else {
        return None;
    };
    let stripped = stripped
        .strip_prefix("json")
        .or_else(|| stripped.strip_prefix("JSON"))
        .unwrap_or(stripped)
        .trim_start();
    let Some(fence_end) = stripped.find("```") else {
        return Some(stripped.trim());
    };

    Some(stripped[..fence_end].trim())
}

fn first_balanced_json_object(value: &str) -> Option<&str> {
    let start = value.char_indices().find(|(_, ch)| *ch == '{')?.0;
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in value[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let end = start + offset + ch.len_utf8();
                    return Some(value[start..end].trim());
                }
            }
            _ => {}
        }
    }

    None
}
