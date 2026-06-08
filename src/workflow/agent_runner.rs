use serde_json::Value;

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
    let mut last_output = None;

    for attempt in 1..=max_attempts {
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
            "上一次输出不是合法 JSON 或不符合 envelope。请只修复 JSON 格式，保持业务内容，不要在 JSON 外追加解释。"
                .to_string()
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
                artifacts: vec![],
            },
        };
        output.attempt = attempt;
        output.will_fallback = output.parse_error.is_some() && attempt == max_attempts;
        storage.agent_runs().insert(novel_id, task, &output).await?;

        if output.parse_error.is_none() {
            return Ok(output);
        }

        last_output = Some(output);
    }

    Ok(last_output.expect("max_attempts is always at least one"))
}
