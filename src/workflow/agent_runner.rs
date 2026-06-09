use serde_json::Value;
use std::time::Instant;

use crate::agents::{
    AgentConstraints, AgentContext, AgentInput, AgentOutput, AgentRole, AgentTask, ModelHandle,
    NovelAgent, PromptAgent,
};
use crate::domain::NovelId;
use crate::error::WorkflowError;
use crate::storage::SqliteStorage;

pub async fn run_prompt_agent(
    storage: &SqliteStorage,
    model: ModelHandle,
    novel_id: Option<&NovelId>,
    role: AgentRole,
    task: AgentTask,
    system_prompt: &'static str,
    payload: Value,
) -> Result<AgentOutput, WorkflowError> {
    let agent = PromptAgent::new(role, system_prompt);
    let constraints = AgentConstraints::default();
    let max_attempts = constraints.max_retries + 1;
    let mut last_output: Option<AgentOutput> = None;

    for attempt in 1..=max_attempts {
        let started_at = Instant::now();
        let ctx = AgentContext {
            novel_id: novel_id.cloned(),
            memory: None,
            model: model.clone(),
            storage: None,
            constraints: constraints.clone(),
        };
        let retry_instruction = if attempt == 1 {
            "严格按系统提示词要求输出 JSON，不要在 JSON 外追加解释。".to_string()
        } else {
            let last_error = last_output
                .as_ref()
                .and_then(|output| output.parse_error.as_deref())
                .unwrap_or("未知解析错误");
            format!(
                "上一次输出不是合法 JSON 或不符合 envelope。\n\
                 上一次解析错误：{last_error}\n\
                 请只修复 JSON 格式和 AgentOutput envelope，保持业务内容，不要在 JSON 外追加解释。"
            )
        };
        let input = AgentInput {
            task,
            prompt: retry_instruction,
            payload: payload.clone(),
            context: vec![],
        };

        let mut output = match agent.run(ctx, input).await {
            Ok(output) => output,
            Err(err) => AgentOutput {
                role,
                structured: serde_json::json!({}),
                raw_text: String::new(),
                parse_error: Some(err.to_string()),
                raw_notes: String::new(),
                attempt,
                will_fallback: false,
                duration_ms: None,
                token_usage: None,
                artifacts: vec![],
            },
        };
        output.attempt = attempt;
        output.will_fallback = output.parse_error.is_some() && attempt == max_attempts;
        output.duration_ms = Some(elapsed_ms(started_at));
        attach_workflow_context(&mut output.structured, &payload);
        storage.agent_runs().insert(novel_id, task, &output).await?;

        if output.parse_error.is_none() {
            return Ok(output);
        }

        last_output = Some(output);
    }

    Ok(last_output.expect("max_attempts is always at least one"))
}

fn elapsed_ms(started_at: Instant) -> u64 {
    let millis = started_at.elapsed().as_millis();
    millis.min(u128::from(u64::MAX)) as u64
}

fn attach_workflow_context(structured: &mut Value, payload: &Value) {
    let Some(chapter_draft) = payload.get("chapter_draft").and_then(Value::as_object) else {
        return;
    };
    let Some(chapter_index) = chapter_draft.get("chapter_index").and_then(Value::as_u64) else {
        return;
    };

    let mut workflow = serde_json::Map::new();
    workflow.insert("chapter_index".to_string(), Value::from(chapter_index));
    if let Some(volume_index) = chapter_draft.get("volume_index").and_then(Value::as_u64) {
        workflow.insert("volume_index".to_string(), Value::from(volume_index));
    }
    if let Some(chapter_id) = chapter_draft.get("chapter_id").and_then(Value::as_str) {
        workflow.insert("chapter_id".to_string(), Value::from(chapter_id));
    }

    if let Some(map) = structured.as_object_mut() {
        map.insert("_workflow".to_string(), Value::Object(workflow));
    }
}
