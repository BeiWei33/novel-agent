use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use novel_agent::agents::ModelHandle;
use novel_agent::domain::{ChapterOutline, ChapterStatus, CharacterCard, TargetPlatform};
use novel_agent::error::ModelError;
use novel_agent::model::{
    ModelClient, ModelProvider, ModelRequest, ModelResponse, SmokeModelClient,
};
use novel_agent::storage::SqliteStorage;
use novel_agent::workflow::{ChapterGenerationWorkflow, NovelCreationWorkflow};
use uuid::Uuid;

#[derive(Debug)]
struct InvalidJsonModel;
#[derive(Debug)]
struct MissingEnvelopeModel;
#[derive(Debug)]
struct MissingStructuredFieldModel;
#[derive(Debug)]
struct RetryAwareEnvelopeModel {
    attempts_by_role: Mutex<HashMap<&'static str, u32>>,
}
#[derive(Debug)]
struct FencedJsonModel;
#[derive(Debug)]
struct DirectTrailingJsonModel;
#[derive(Debug)]
struct ValidJsonModel {
    fixture: FixtureKind,
}

#[derive(Debug)]
struct NeedsRewriteModel;

#[derive(Debug, Clone, Copy)]
enum FixtureKind {
    Fantasy,
    Urban,
    Romance,
}

struct FixtureSpec {
    kind: FixtureKind,
    path: &'static str,
    idea: &'static str,
    platform: TargetPlatform,
    expected_title: &'static str,
}

#[async_trait]
impl ModelClient for InvalidJsonModel {
    async fn complete(&self, _request: ModelRequest) -> Result<ModelResponse, ModelError> {
        Ok(ModelResponse {
            text: "not json".to_string(),
            raw: "not json".to_string(),
            usage: None,
        })
    }
}

#[async_trait]
impl ModelClient for MissingEnvelopeModel {
    async fn complete(&self, _request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let text = r#"{"market_analysis":{},"raw_notes":"missing envelope"}"#.to_string();
        Ok(ModelResponse {
            raw: text.clone(),
            text,
            usage: None,
        })
    }
}

#[async_trait]
impl ModelClient for MissingStructuredFieldModel {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let role = role_key_for_system(request.system_prompt.as_deref().unwrap_or_default());
        let text = serde_json::json!({
            "role": role,
            "structured": {},
            "raw_notes": ""
        })
        .to_string();
        Ok(ModelResponse {
            raw: text.clone(),
            text,
            usage: None,
        })
    }
}

#[async_trait]
impl ModelClient for ValidJsonModel {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let system = request.system_prompt.as_deref().unwrap_or_default();
        let text = fixture_response_for_system(self.fixture, system);

        Ok(ModelResponse {
            raw: text.clone(),
            text,
            usage: None,
        })
    }
}

#[async_trait]
impl ModelClient for RetryAwareEnvelopeModel {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let system = request.system_prompt.as_deref().unwrap_or_default();
        let role_key = role_key_for_system(system);
        let attempt = {
            let mut attempts = self.attempts_by_role.lock().unwrap();
            let entry = attempts.entry(role_key).or_default();
            *entry += 1;
            *entry
        };
        let text = if attempt == 1 {
            r#"{"market_analysis":{},"raw_notes":"missing envelope"}"#.to_string()
        } else if request.prompt.contains("上一次解析错误：")
            && request.prompt.contains("AgentOutput envelope")
        {
            fixture_response_for_system(FixtureKind::Fantasy, system)
        } else {
            r#"{"role":"unknown","structured":{},"raw_notes":"retry prompt missed parse error"}"#
                .to_string()
        };

        Ok(ModelResponse {
            raw: text.clone(),
            text,
            usage: None,
        })
    }
}

#[async_trait]
impl ModelClient for FencedJsonModel {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let system = request.system_prompt.as_deref().unwrap_or_default();
        let text = format!(
            "```json\n{}\n```\n以上为结构化 JSON。",
            fixture_response_for_system(FixtureKind::Fantasy, system)
        );

        Ok(ModelResponse {
            raw: text.clone(),
            text,
            usage: None,
        })
    }
}

#[async_trait]
impl ModelClient for DirectTrailingJsonModel {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let system = request.system_prompt.as_deref().unwrap_or_default();
        let text = format!(
            "{}\n以上为结构化 JSON，请按需入库。",
            fixture_response_for_system(FixtureKind::Fantasy, system)
        );

        Ok(ModelResponse {
            raw: text.clone(),
            text,
            usage: None,
        })
    }
}

#[async_trait]
impl ModelClient for NeedsRewriteModel {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let system = request.system_prompt.as_deref().unwrap_or_default();
        let prompt = request.prompt.as_str();
        let text = if system.contains("Market Agent Prompt") {
            market_json()
        } else if system.contains("Plot Agent Prompt") {
            plot_json()
        } else if system.contains("Character Agent Prompt") {
            character_json()
        } else if system.contains("Worldbuilding Agent Prompt") {
            worldbuilding_json()
        } else if system.contains("Chapter Writer Agent Prompt")
            && prompt.contains("rewrite_chapter")
        {
            rewrite_writer_json()
        } else if system.contains("Chapter Writer Agent Prompt") {
            writer_json()
        } else if system.contains("Continuity Agent Prompt") {
            continuity_json()
        } else if system.contains("Style Agent Prompt") && prompt.contains("重写后强化目标")
        {
            rewrite_style_json()
        } else if system.contains("Style Agent Prompt") {
            style_json()
        } else if system.contains("Reviewer Agent Prompt") && prompt.contains("重写后强化目标")
        {
            reviewer_json()
        } else if system.contains("Reviewer Agent Prompt") {
            low_score_reviewer_json()
        } else {
            r#"{"role":"unknown","structured":{},"raw_notes":""}"#.to_string()
        };

        Ok(ModelResponse {
            raw: text.clone(),
            text,
            usage: None,
        })
    }
}

#[tokio::test]
async fn fenced_agent_output_json_with_trailing_text_is_parsed() {
    let db_path = std::env::temp_dir().join(format!("novel-agent-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    );
    let storage = SqliteStorage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let model: ModelHandle = Arc::new(FencedJsonModel);

    let creation = NovelCreationWorkflow::new(&storage, model);
    let result = creation
        .create_from_idea(
            "玄幻升级文，边城少年继承一座会记录因果债的古塔",
            TargetPlatform::Qidian,
        )
        .await
        .unwrap();
    assert!(!result.used_fallback);
    assert_eq!(result.novel.title, "因果塔债");

    let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
    let parse_errors: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_runs WHERE parse_error IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(parse_errors, 0);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn direct_agent_output_json_with_trailing_text_is_parsed() {
    let db_path = std::env::temp_dir().join(format!("novel-agent-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    );
    let storage = SqliteStorage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let model: ModelHandle = Arc::new(DirectTrailingJsonModel);

    let creation = NovelCreationWorkflow::new(&storage, model);
    let result = creation
        .create_from_idea(
            "玄幻升级文，边城少年继承一座会记录因果债的古塔",
            TargetPlatform::Qidian,
        )
        .await
        .unwrap();
    assert!(!result.used_fallback);
    assert_eq!(result.novel.title, "因果塔债");

    let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
    let parse_errors: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_runs WHERE parse_error IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(parse_errors, 0);

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn model_provider_parse_supports_local_smoke_aliases() {
    assert_eq!(ModelProvider::parse("smoke").unwrap(), ModelProvider::Smoke);
    assert_eq!(ModelProvider::parse("local").unwrap(), ModelProvider::Smoke);
    assert_eq!(
        ModelProvider::parse("offline").unwrap(),
        ModelProvider::Smoke
    );
    assert_eq!(
        ModelProvider::parse("openai").unwrap(),
        ModelProvider::OpenAi
    );
    assert_eq!(
        ModelProvider::parse("deepseek").unwrap(),
        ModelProvider::DeepSeek
    );
}

#[tokio::test]
async fn json_without_agent_output_envelope_is_marked_parse_error() {
    let db_path = std::env::temp_dir().join(format!("novel-agent-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    );
    let storage = SqliteStorage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let model: ModelHandle = Arc::new(MissingEnvelopeModel);

    let creation = NovelCreationWorkflow::new(&storage, model);
    let result = creation
        .create_from_idea(
            "都市重生商业文，主角回到十年前，从外卖站开始逆袭",
            TargetPlatform::Fanqie,
        )
        .await
        .unwrap();
    assert!(result.used_fallback);

    let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
    let parse_error: String = sqlx::query_scalar(
        "SELECT parse_error FROM agent_runs WHERE parse_error IS NOT NULL LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(parse_error.contains("AgentOutput envelope"));

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn structured_missing_required_agent_fields_is_marked_parse_error() {
    let db_path = std::env::temp_dir().join(format!("novel-agent-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    );
    let storage = SqliteStorage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let model: ModelHandle = Arc::new(MissingStructuredFieldModel);

    let creation = NovelCreationWorkflow::new(&storage, model);
    let result = creation
        .create_from_idea(
            "都市重生商业文，主角回到十年前，从外卖站开始逆袭",
            TargetPlatform::Fanqie,
        )
        .await
        .unwrap();
    assert!(result.used_fallback);

    let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
    let missing_market_analysis_errors: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_runs WHERE parse_error LIKE '%market_analysis%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(missing_market_analysis_errors > 0);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn retry_prompt_includes_previous_parse_error() {
    let db_path = std::env::temp_dir().join(format!("novel-agent-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    );
    let storage = SqliteStorage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let model = Arc::new(RetryAwareEnvelopeModel {
        attempts_by_role: Mutex::new(HashMap::new()),
    });
    let model_handle: ModelHandle = model.clone();

    let creation = NovelCreationWorkflow::new(&storage, model_handle);
    let result = creation
        .create_from_idea_with_outline_batch_size(
            "玄幻升级文，边城少年继承一座会记录因果债的古塔",
            TargetPlatform::Qidian,
            6,
            10,
        )
        .await
        .unwrap();
    assert!(!result.used_fallback);
    assert_eq!(result.novel.title, "因果塔债");

    let attempts = model.attempts_by_role.lock().unwrap();
    for role in ["market", "plot", "character", "worldbuilding"] {
        assert_eq!(attempts.get(role), Some(&2), "{role} should retry once");
    }
    drop(attempts);

    let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
    let parse_errors: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_runs WHERE parse_error LIKE '%AgentOutput envelope%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(parse_errors, 4);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn local_smoke_provider_drives_workflows_without_fallback() {
    let db_path = std::env::temp_dir().join(format!("novel-agent-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    );
    let storage = SqliteStorage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let model: ModelHandle = Arc::new(SmokeModelClient::new("smoke"));

    let creation = NovelCreationWorkflow::new(&storage, model.clone());
    let result = creation
        .create_from_idea(
            "都市重生商业文，主角回到十年前，从外卖站开始逆袭",
            TargetPlatform::Fanqie,
        )
        .await
        .unwrap();
    assert!(!result.used_fallback);
    assert_eq!(result.novel.title, "重回外卖站");
    assert_eq!(result.outlines.len(), 30);
    assert_eq!(result.outlines.first().unwrap().chapter_index, 1);
    assert_eq!(result.outlines.last().unwrap().chapter_index, 30);

    let chapters = ChapterGenerationWorkflow::new(&storage, model);
    let draft = chapters.write_chapter(&result.novel.id, 1).await.unwrap();
    assert!(!draft.content.trim().is_empty());
    assert!(!draft
        .continuity_notes
        .iter()
        .any(|note| note.to_ascii_lowercase().contains("fallback")));

    let report = chapters.review_chapter(&result.novel.id, 1).await.unwrap();
    assert!(report.passed);
    assert_eq!(report.total_score, 82);

    let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
    let parse_errors: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_runs WHERE parse_error IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(parse_errors, 0);

    let structured: String =
        sqlx::query_scalar("SELECT structured FROM agent_runs WHERE role = 'market' LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    let structured: serde_json::Value = serde_json::from_str(&structured).unwrap();
    let total_tokens = structured["_engineering"]["token_usage"]["total_tokens"]
        .as_u64()
        .unwrap_or_default();
    assert!(total_tokens > 0);
    let plot_runs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_runs WHERE role = 'plot' AND parse_error IS NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(plot_runs, 6);
    let runs = storage
        .agent_runs()
        .list_recent(Some(&result.novel.id), 20)
        .await
        .unwrap();
    assert!(!runs.is_empty());
    assert!(runs
        .iter()
        .all(|run| run.novel_id.as_ref() == Some(&result.novel.id)));
    assert!(runs.iter().any(|run| run.role == "market"));
    assert!(runs.iter().any(|run| run.role == "reviewer"));
    assert!(runs
        .iter()
        .any(|run| run.structured.get("_engineering").is_some()));

    let export_path = std::env::temp_dir().join(format!("novel-agent-{}.md", Uuid::new_v4()));
    let exported = chapters
        .export_markdown(&result.novel.id, Some(export_path.clone()))
        .await
        .unwrap();
    assert_eq!(exported, export_path);

    let _ = std::fs::remove_file(db_path);
    let _ = std::fs::remove_file(export_path);
}

#[tokio::test]
async fn create_novel_can_limit_initial_outline_chapters() {
    let db_path = std::env::temp_dir().join(format!("novel-agent-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    );
    let storage = SqliteStorage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let model: ModelHandle = Arc::new(SmokeModelClient::new("smoke"));

    let creation = NovelCreationWorkflow::new(&storage, model);
    let result = creation
        .create_from_idea_with_outline_batch_size(
            "都市重生商业文，主角回到十年前，从外卖站开始逆袭",
            TargetPlatform::Fanqie,
            6,
            10,
        )
        .await
        .unwrap();
    assert!(!result.used_fallback);
    assert_eq!(result.outlines.len(), 6);

    let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
    let saved_chapters: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chapters WHERE novel_id = ?")
            .bind(result.novel.id.to_string())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(saved_chapters, 6);
    let plot_runs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM agent_runs WHERE role = 'plot' AND parse_error IS NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(plot_runs, 1);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn create_novel_can_merge_batched_initial_outlines() {
    let db_path = std::env::temp_dir().join(format!("novel-agent-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    );
    let storage = SqliteStorage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let model: ModelHandle = Arc::new(SmokeModelClient::new("smoke"));

    let creation = NovelCreationWorkflow::new(&storage, model);
    let result = creation
        .create_from_idea_with_outline_batch_size(
            "都市重生商业文，主角回到十年前，从外卖站开始逆袭",
            TargetPlatform::Fanqie,
            13,
            6,
        )
        .await
        .unwrap();
    assert!(!result.used_fallback);
    assert_eq!(result.outlines.len(), 13);
    assert_eq!(
        result
            .outlines
            .iter()
            .map(|outline| outline.chapter_index)
            .collect::<Vec<_>>(),
        (1..=13).collect::<Vec<_>>()
    );

    let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
    let saved_chapters: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chapters WHERE novel_id = ?")
            .bind(result.novel.id.to_string())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(saved_chapters, 13);
    let plot_runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_runs WHERE role = 'plot'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(plot_runs, 3);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn local_smoke_provider_supports_streamed_completion_chunks() {
    let client = SmokeModelClient::new("smoke");
    let response = client
        .complete_stream(ModelRequest {
            system_prompt: Some("Market Agent Prompt".to_string()),
            prompt: r#"{"target_platform":"fanqie","idea":"都市重生商业文，主角回到十年前，从外卖站开始逆袭"}"#.repeat(4),
            temperature: Some(0.7),
            max_tokens: None,
        })
        .await
        .unwrap();

    assert!(response.chunks.len() > 1);
    assert!(response.chunks.last().unwrap().is_final);
    let stitched = response
        .chunks
        .iter()
        .map(|chunk| chunk.text.as_str())
        .collect::<String>();
    assert_eq!(stitched, response.response.text);
    assert!(
        response
            .response
            .usage
            .and_then(|usage| usage.total_tokens)
            .unwrap_or_default()
            > 0
    );
}

#[tokio::test]
async fn smoke_flow_falls_back_when_model_output_is_invalid() {
    let db_path = std::env::temp_dir().join(format!("novel-agent-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    );
    let storage = SqliteStorage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let model: ModelHandle = Arc::new(InvalidJsonModel);

    let creation = NovelCreationWorkflow::new(&storage, model.clone());
    let result = creation
        .create_from_idea(
            "都市重生商业文，主角回到十年前，从外卖站开始逆袭",
            TargetPlatform::Fanqie,
        )
        .await
        .unwrap();

    assert_eq!(result.outlines.len(), 30);
    assert!(result.used_fallback);

    let chapters = ChapterGenerationWorkflow::new(&storage, model);
    let draft = chapters.write_chapter(&result.novel.id, 1).await.unwrap();
    assert!(!draft.content.trim().is_empty());
    assert!(!draft.new_facts.is_empty());
    assert!(draft
        .continuity_notes
        .iter()
        .any(|note| note.contains("fallback")));

    let report = chapters.review_chapter(&result.novel.id, 1).await.unwrap();
    assert!(report.total_score >= 75);
    assert!(report
        .suggestions
        .iter()
        .any(|suggestion| suggestion.contains("fallback")));

    let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
    let parse_errors: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_runs WHERE parse_error IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(parse_errors > 0);
    let structured: String = sqlx::query_scalar("SELECT structured FROM agent_runs LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    let structured: serde_json::Value = serde_json::from_str(&structured).unwrap();
    let engineering = structured
        .get("_engineering")
        .expect("agent run should include engineering metadata");
    assert!(engineering
        .get("duration_ms")
        .and_then(serde_json::Value::as_u64)
        .is_some());
    assert!(engineering.get("token_usage").is_some());

    let export_path = std::env::temp_dir().join(format!("novel-agent-{}.md", Uuid::new_v4()));
    let exported = chapters
        .export_markdown(&result.novel.id, Some(export_path.clone()))
        .await
        .unwrap();
    assert_eq!(exported, export_path);

    let _ = std::fs::remove_file(db_path);
    let _ = std::fs::remove_file(export_path);
}

#[tokio::test]
async fn valid_json_model_outputs_match_fixture_expected_checks() {
    for spec in [
        FixtureSpec {
            kind: FixtureKind::Urban,
            path: "examples/urban_rebirth.md",
            idea: "都市重生商业文，主角回到十年前，从外卖站开始逆袭。",
            platform: TargetPlatform::Fanqie,
            expected_title: "重回外卖站",
        },
        FixtureSpec {
            kind: FixtureKind::Fantasy,
            path: "examples/fantasy_upgrade.md",
            idea: "玄幻升级文，边城少年继承一座会记录因果债的古塔",
            platform: TargetPlatform::Qidian,
            expected_title: "因果塔债",
        },
        FixtureSpec {
            kind: FixtureKind::Romance,
            path: "examples/romance_comeback.md",
            idea:
                "现代女性向逆袭复仇，女主被未婚夫和闺蜜联手夺走公司后，回到签署股权转让协议前一天。",
            platform: TargetPlatform::Fanqie,
            expected_title: "签约前夜",
        },
    ] {
        assert_fixture_output_matches_expected_checks(spec).await;
    }
}

async fn assert_fixture_output_matches_expected_checks(spec: FixtureSpec) {
    let db_path = std::env::temp_dir().join(format!("novel-agent-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    );
    let storage = SqliteStorage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let model: ModelHandle = Arc::new(ValidJsonModel { fixture: spec.kind });

    let creation = NovelCreationWorkflow::new(&storage, model.clone());
    let result = creation
        .create_from_idea(spec.idea, spec.platform)
        .await
        .unwrap();

    assert!(!result.used_fallback);
    assert_eq!(result.novel.title, spec.expected_title);
    let fixture = load_fixture_checks(spec.path);
    let checks = &fixture["expected_checks"];

    assert!(
        result.bible.title_candidates.len()
            >= checks["market"]["min_title_candidates"].as_u64().unwrap() as usize
    );
    assert_contains_all(
        &result.bible.platform_tags.join(" "),
        &checks["market"]["required_tags"],
    );
    assert_contains_all(
        &result.bible.core_selling_points.join(" "),
        &checks["market"]["must_include_selling_points"],
    );
    assert!(
        result.outlines.len() >= checks["plot"]["min_chapter_outlines"].as_u64().unwrap() as usize
    );
    assert_chapter_outline_matches(&result.outlines[0], &checks["plot"]["chapter_1"]);
    assert_roles_include(&result.characters, &checks["character"]["required_roles"]);
    assert_protagonist_matches(
        &result.characters,
        &checks["character"]["protagonist_must_have"],
    );
    assert!(result.bible.platform_profile.is_some());

    let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
    if let Some(expected_hook) = checks["plot"]["must_include_long_term_hook"].as_str() {
        let plot_structured = latest_agent_structured(&pool, "plot").await;
        assert_contains_alternative(
            plot_structured["plot_plan"]["long_term_hook"]
                .as_str()
                .unwrap_or_default(),
            expected_hook,
        );
    }

    let world_setting = storage
        .world_settings()
        .find(&result.novel.id)
        .await
        .unwrap()
        .unwrap();
    assert_contains_all_value(
        &world_setting,
        &checks["worldbuilding"]["required_world_elements"],
    );
    assert_contains_all_value(
        &world_setting,
        &checks["worldbuilding"]["required_hard_rules"],
    );
    let worldbuilding_structured = latest_agent_structured(&pool, "worldbuilding").await;
    assert_contains_all_value(
        &worldbuilding_structured["facts_to_seed"],
        &checks["worldbuilding"]["required_seed_facts"],
    );

    let chapters = ChapterGenerationWorkflow::new(&storage, model);
    let draft = chapters.write_chapter(&result.novel.id, 1).await.unwrap();
    assert!(!draft.title.trim().is_empty());
    assert!(
        draft.word_count
            >= checks["writer"]["chapter_1_min_word_count"]
                .as_u64()
                .unwrap() as u32
    );
    assert_contains_all(&draft.content, &checks["writer"]["must_include"]);
    assert_contains_none(&draft.content, &checks["writer"]["forbidden"]);
    assert!(!draft
        .continuity_notes
        .iter()
        .any(|note| note.contains("fallback")));
    let continuity_report = storage
        .continuity_reports()
        .latest_for_chapter(&draft.chapter_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(continuity_report["passed"], true);
    assert_contains_all_value(
        &continuity_report,
        &checks["continuity"]["must_track_facts"],
    );
    assert!(!continuity_report["character_state_updates"]
        .as_array()
        .unwrap_or(&vec![])
        .is_empty());
    let high_severity_count = continuity_report["issues"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter(|issue| issue["severity"].as_str() == Some("high"))
        .count() as i64;
    assert!(
        high_severity_count
            <= checks["continuity"]["max_high_severity_issues"]
                .as_i64()
                .unwrap()
    );
    assert_contains_all(&draft.content, &checks["style"]["must_preserve"]);
    assert_contains_none(&draft.content, &checks["style"]["forbidden"]);
    let facts = storage
        .facts()
        .list_by_novel(&result.novel.id, 20)
        .await
        .unwrap();
    let facts_text = facts
        .iter()
        .map(|fact| format!("{} {} {}", fact.subject, fact.predicate, fact.object))
        .collect::<Vec<_>>()
        .join(" ");
    assert_contains_all(&facts_text, &checks["continuity"]["must_track_facts"]);

    let report = chapters.review_chapter(&result.novel.id, 1).await.unwrap();
    assert!(report.passed);
    assert_eq!(report.scores.cliffhanger_score, 8);
    assert!(
        report.total_score
            >= checks["review"]["pass_line"]["total_score"]
                .as_i64()
                .unwrap() as i32
    );
    assert!(
        report.scores.pacing_score
            >= checks["review"]["pass_line"]["pacing_score"]
                .as_i64()
                .unwrap() as i32
    );
    assert!(
        report.scores.continuity_score
            >= checks["review"]["pass_line"]["continuity_score"]
                .as_i64()
                .unwrap() as i32
    );
    assert!(
        report.scores.cliffhanger_score
            >= checks["review"]["pass_line"]["cliffhanger_score"]
                .as_i64()
                .unwrap() as i32
    );

    let parse_errors: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM agent_runs WHERE parse_error IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(parse_errors, 0);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn low_score_review_can_rewrite_and_record_versions() {
    let db_path = std::env::temp_dir().join(format!("novel-agent-{}.db", Uuid::new_v4()));
    let database_url = format!(
        "sqlite://{}",
        db_path.display().to_string().replace('\\', "/")
    );
    let storage = SqliteStorage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let model: ModelHandle = Arc::new(NeedsRewriteModel);

    let creation = NovelCreationWorkflow::new(&storage, model.clone());
    let result = creation
        .create_from_idea(
            "玄幻升级文，边城少年继承一座会记录因果债的古塔",
            TargetPlatform::Qidian,
        )
        .await
        .unwrap();

    let chapters = ChapterGenerationWorkflow::new(&storage, model);
    let first_draft = chapters.write_chapter(&result.novel.id, 1).await.unwrap();
    assert_eq!(first_draft.version, 1);
    assert_eq!(
        storage
            .chapter_versions()
            .list_version_numbers(&first_draft.chapter_id)
            .await
            .unwrap(),
        vec![1]
    );

    let low_report = chapters.review_chapter(&result.novel.id, 1).await.unwrap();
    assert!(!low_report.passed);
    assert!(low_report.rewrite_instruction.needed);
    let reviewed_chapter = storage
        .chapters()
        .find_by_index(&result.novel.id, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reviewed_chapter.status, ChapterStatus::RewriteNeeded);

    let rewritten = chapters.rewrite_chapter(&result.novel.id, 1).await.unwrap();
    assert_eq!(rewritten.version, 2);
    assert!(rewritten.content.contains("重写后强化目标"));
    assert_eq!(
        storage
            .chapter_versions()
            .count_for_chapter(&rewritten.chapter_id)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        storage
            .chapter_versions()
            .list_version_numbers(&rewritten.chapter_id)
            .await
            .unwrap(),
        vec![1, 2]
    );
    assert!(storage
        .chapter_versions()
        .content_for_version(&rewritten.chapter_id, 1)
        .await
        .unwrap()
        .unwrap()
        .contains("能力代价"));
    assert!(storage
        .chapter_versions()
        .content_for_version(&rewritten.chapter_id, 2)
        .await
        .unwrap()
        .unwrap()
        .contains("重写后强化目标"));

    let final_chapter = storage
        .chapters()
        .find_by_index(&result.novel.id, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(final_chapter.version, 2);
    assert_eq!(final_chapter.status, ChapterStatus::Final);

    let manual_content = format!(
        "{}\n人工编辑新增：沈砚主动记录第二笔因果债，把下一章目标钉得更清楚。",
        rewritten.content
    );
    let manual = chapters
        .save_manual_edit(
            &result.novel.id,
            1,
            Some("第一章 债印初醒（人工修订）".to_string()),
            manual_content.clone(),
            Some("人工编辑后补强第二笔因果债和下一章目标。".to_string()),
        )
        .await
        .unwrap();
    assert_eq!(manual.version, 3);
    assert_eq!(manual.title, "第一章 债印初醒（人工修订）");
    assert_eq!(manual.word_count, count_chars(&manual_content));
    assert!(manual.content.contains("人工编辑新增"));
    assert_eq!(
        storage
            .chapter_versions()
            .list_version_numbers(&manual.chapter_id)
            .await
            .unwrap(),
        vec![1, 2, 3]
    );
    assert!(storage
        .chapter_versions()
        .content_for_version(&manual.chapter_id, 3)
        .await
        .unwrap()
        .unwrap()
        .contains("人工编辑新增"));

    let edited_chapter = storage
        .chapters()
        .find_by_index(&result.novel.id, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(edited_chapter.version, 3);
    assert_eq!(edited_chapter.status, ChapterStatus::Drafted);
    assert_eq!(edited_chapter.score, None);

    let manual_report = chapters.review_chapter(&result.novel.id, 1).await.unwrap();
    assert!(manual_report.passed);
    let reviewed_manual = storage
        .chapters()
        .find_by_index(&result.novel.id, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reviewed_manual.version, 3);
    assert_eq!(reviewed_manual.status, ChapterStatus::Final);

    let review_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM review_reports
        WHERE chapter_id = ?1
        "#,
    )
    .bind(rewritten.chapter_id.as_str())
    .fetch_one(&sqlx::SqlitePool::connect(&database_url).await.unwrap())
    .await
    .unwrap();
    assert_eq!(review_count, 3);

    let _ = std::fs::remove_file(db_path);
}

#[test]
fn examples_expose_expected_checks_json() {
    for path in [
        "examples/urban_rebirth.md",
        "examples/fantasy_upgrade.md",
        "examples/romance_comeback.md",
    ] {
        let content = std::fs::read_to_string(path).unwrap();
        let json = extract_expected_checks_json(&content).unwrap();
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(value.get("fixture_id").is_some());
        assert!(value.get("input").is_some());
        assert!(value.get("expected_checks").is_some());
        let checks = &value["expected_checks"];
        assert!(checks["plot"]["min_chapter_outlines"].as_i64().unwrap() >= 30);
        assert_non_empty_array(&checks["worldbuilding"]["required_world_elements"]);
        assert_non_empty_array(&checks["worldbuilding"]["required_hard_rules"]);
        assert_non_empty_array(&checks["worldbuilding"]["required_seed_facts"]);
        assert_non_empty_array(&checks["continuity"]["must_track_facts"]);
        assert_eq!(
            checks["continuity"]["require_character_state_updates"].as_bool(),
            Some(true)
        );
        assert_eq!(
            checks["continuity"]["max_high_severity_issues"].as_i64(),
            Some(0)
        );
        assert_non_empty_array(&checks["style"]["must_preserve"]);
        assert_non_empty_array(&checks["style"]["must_improve"]);
        assert_non_empty_array(&checks["style"]["forbidden"]);
    }
}

fn assert_non_empty_array(value: &serde_json::Value) {
    assert!(value.as_array().is_some_and(|items| !items.is_empty()));
}

fn load_fixture_checks(path: &str) -> serde_json::Value {
    let content = std::fs::read_to_string(path).unwrap();
    let json = extract_expected_checks_json(&content).unwrap();
    serde_json::from_str(json).unwrap()
}

fn assert_contains_all(haystack: &str, expected: &serde_json::Value) {
    for item in expected.as_array().unwrap() {
        let needle = item.as_str().unwrap();
        assert!(
            haystack.contains(needle),
            "expected text to contain '{needle}', got: {haystack}"
        );
    }
}

fn assert_contains_none(haystack: &str, forbidden: &serde_json::Value) {
    for item in forbidden.as_array().unwrap() {
        let needle = item.as_str().unwrap();
        assert!(
            !haystack.contains(needle),
            "expected text not to contain '{needle}', got: {haystack}"
        );
    }
}

fn assert_contains_all_value(haystack: &serde_json::Value, expected: &serde_json::Value) {
    let text = serde_json::to_string(haystack).unwrap();
    assert_contains_all(&text, expected);
}

fn assert_contains_alternative(haystack: &str, expected: &str) {
    let alternatives = expected
        .split('或')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    assert!(
        alternatives.iter().any(|needle| haystack.contains(needle)),
        "expected text to contain one of {:?}, got: {}",
        alternatives,
        haystack
    );
}

fn assert_chapter_outline_matches(outline: &ChapterOutline, checks: &serde_json::Value) {
    if checks["must_have_goal"].as_bool().unwrap_or(false) {
        assert!(!outline.goal.trim().is_empty());
    }
    if checks["must_have_conflict"].as_bool().unwrap_or(false) {
        assert!(!outline.conflict.trim().is_empty());
    }
    if checks["must_have_cliffhanger"].as_bool().unwrap_or(false) {
        assert!(!outline.cliffhanger.trim().is_empty());
    }

    let outline_text = serde_json::to_string(outline).unwrap();
    assert_contains_all(&outline_text, &checks["required_events"]);
}

fn assert_roles_include(characters: &[CharacterCard], expected: &serde_json::Value) {
    let roles = characters
        .iter()
        .map(|character| character.role.as_str())
        .collect::<Vec<_>>();
    for item in expected.as_array().unwrap() {
        let role = item.as_str().unwrap();
        assert!(
            roles.contains(&role),
            "expected roles to contain '{role}', got: {:?}",
            roles
        );
    }
}

fn assert_protagonist_matches(characters: &[CharacterCard], expected: &serde_json::Value) {
    let protagonist = characters
        .iter()
        .find(|character| character.role == "protagonist")
        .expect("protagonist exists");
    let text = serde_json::to_string(protagonist).unwrap();
    assert_contains_all(&text, expected);
}

async fn latest_agent_structured(pool: &sqlx::SqlitePool, role: &str) -> serde_json::Value {
    let data: String = sqlx::query_scalar(
        r#"
        SELECT structured
        FROM agent_runs
        WHERE role = ?1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(role)
    .fetch_one(pool)
    .await
    .unwrap();

    serde_json::from_str(&data).unwrap()
}

fn extract_expected_checks_json(content: &str) -> Option<&str> {
    let marker = "## 回归验收 JSON";
    let start = content.find(marker)?;
    let after_marker = &content[start..];
    let fence_start = after_marker.find("```json")? + "```json".len();
    let after_fence = &after_marker[fence_start..];
    let fence_end = after_fence.find("```")?;
    Some(after_fence[..fence_end].trim())
}

fn fixture_response_for_system(fixture: FixtureKind, system: &str) -> String {
    if system.contains("Market Agent Prompt") {
        fixture.market_json()
    } else if system.contains("Plot Agent Prompt") {
        fixture.plot_json()
    } else if system.contains("Character Agent Prompt") {
        fixture.character_json()
    } else if system.contains("Worldbuilding Agent Prompt") {
        fixture.worldbuilding_json()
    } else if system.contains("Chapter Writer Agent Prompt") {
        fixture.writer_json()
    } else if system.contains("Continuity Agent Prompt") {
        fixture.continuity_json()
    } else if system.contains("Style Agent Prompt") {
        fixture.style_json()
    } else if system.contains("Reviewer Agent Prompt") {
        reviewer_json()
    } else {
        r#"{"role":"unknown","structured":{},"raw_notes":""}"#.to_string()
    }
}

fn role_key_for_system(system: &str) -> &'static str {
    if system.contains("Market Agent Prompt") {
        "market"
    } else if system.contains("Plot Agent Prompt") {
        "plot"
    } else if system.contains("Character Agent Prompt") {
        "character"
    } else if system.contains("Worldbuilding Agent Prompt") {
        "worldbuilding"
    } else if system.contains("Chapter Writer Agent Prompt") {
        "writer"
    } else if system.contains("Continuity Agent Prompt") {
        "continuity"
    } else if system.contains("Style Agent Prompt") {
        "style"
    } else if system.contains("Reviewer Agent Prompt") {
        "reviewer"
    } else {
        "unknown"
    }
}

impl FixtureKind {
    fn market_json(self) -> String {
        match self {
            Self::Fantasy => market_json(),
            Self::Urban | Self::Romance => fixture_market_json(self),
        }
    }

    fn plot_json(self) -> String {
        match self {
            Self::Fantasy => plot_json(),
            Self::Urban | Self::Romance => fixture_plot_json(self),
        }
    }

    fn character_json(self) -> String {
        match self {
            Self::Fantasy => character_json(),
            Self::Urban => urban_character_json(),
            Self::Romance => romance_character_json(),
        }
    }

    fn worldbuilding_json(self) -> String {
        match self {
            Self::Fantasy => worldbuilding_json(),
            Self::Urban | Self::Romance => fixture_worldbuilding_json(self),
        }
    }

    fn writer_json(self) -> String {
        match self {
            Self::Fantasy => writer_json(),
            Self::Urban | Self::Romance => fixture_writer_json(self),
        }
    }

    fn continuity_json(self) -> String {
        match self {
            Self::Fantasy => continuity_json(),
            Self::Urban | Self::Romance => fixture_continuity_json(self),
        }
    }

    fn style_json(self) -> String {
        match self {
            Self::Fantasy => style_json(),
            Self::Urban | Self::Romance => fixture_style_json(self),
        }
    }
}

struct FixtureTemplate {
    target_platform: &'static str,
    title: &'static str,
    genre: &'static str,
    tags: &'static [&'static str],
    selling_points: &'static [&'static str],
    reader_expectations: &'static [&'static str],
    emotional_hooks: &'static [&'static str],
    first_scene: &'static str,
    first_conflict: &'static str,
    first_three_chapters_goal: &'static str,
    chapter_title: &'static str,
    plot_goal: &'static str,
    plot_conflict: &'static str,
    key_events: &'static [&'static str],
    main_conflict: &'static str,
    protagonist_goal: &'static str,
    antagonistic_force: &'static str,
    long_term_hook: &'static str,
    protagonist_name: &'static str,
    world_overview: &'static str,
    world_name: &'static str,
    world_levels: &'static [&'static str],
    world_rules: &'static [&'static str],
    world_costs: &'static [&'static str],
    hard_rules: &'static [&'static str],
    seed_facts: &'static [(&'static str, &'static str, &'static str, i32)],
    body_paragraph: &'static str,
    summary: &'static str,
    writer_key_events: &'static [&'static str],
    continuity_facts: &'static [(&'static str, &'static str, &'static str, i32)],
    style_note: &'static str,
}

fn fixture_template(kind: FixtureKind) -> FixtureTemplate {
    match kind {
        FixtureKind::Urban => FixtureTemplate {
            target_platform: "fanqie",
            title: "重回外卖站",
            genre: "都市",
            tags: &["都市", "重生", "商业", "逆袭"],
            selling_points: &["未来行业节点", "底层逆袭", "本地生活"],
            reader_expectations: &["快速开篇", "商业反击", "底层逆袭"],
            emotional_hooks: &["修复家庭遗憾", "从外卖站翻身"],
            first_scene: "暴雨夜外卖站甩锅",
            first_conflict: "站点责任即将落到主角身上",
            first_three_chapters_goal: "避开事故，取得团队信任，发现本地试点机会",
            chapter_title: "第一章 暴雨回站",
            plot_goal: "重生后立刻处理外卖站危机，证明主角主动目标",
            plot_conflict: "外卖站危机与站点责任压到林舟身上",
            key_events: &["重生", "外卖站危机", "避开事故"],
            main_conflict: "林舟利用未来经验抓住本地生活行业节点，但资金和人脉约束不断放大风险",
            protagonist_goal: "从外卖站起步，建立自己的本地生活业务主动权",
            antagonistic_force: "周启明的甩锅和陈岳的资本包装",
            long_term_hook: "未来巨头本地试点机会",
            protagonist_name: "林舟",
            world_overview: "外卖站规则、本地生活行业节点、资金和人脉约束共同限制主角的商业选择。",
            world_name: "本地生活创业线",
            world_levels: &["外卖站", "片区调度", "社区团购", "本地生活平台"],
            world_rules: &["外卖站规则必须影响调度和责任划分", "本地生活行业节点只能通过具体行动兑现"],
            world_costs: &["资金和人脉约束会限制扩张速度", "未来信息不能直接替代执行成本"],
            hard_rules: &["未来信息不能无代价碾压", "商业决策必须受资源限制"],
            seed_facts: &[
                ("林舟", "掌握", "林舟掌握未来行业节点", 5),
                ("外卖站", "存在", "外卖站存在即时危机", 5),
            ],
            body_paragraph: "暴雨或配送压力把外卖站堵成一团，林舟醒来就看见站长要把站点责任推到他身上。重生没有给他直接胜利，他用主角主动决策重排骑手路线，先避开事故，再让监控和签收记录证明责任归属。商业决策逻辑来自未来经验，也受资金和人脉约束限制；主角主动性体现在他当场拉许蔓核对本地生活行业节点。章尾钩子是陈岳提前注意到这套调度方案，关键人物关系变化随之出现。\n",
            summary: "林舟重生回外卖站暴雨夜，主动调整调度避开事故。",
            writer_key_events: &["重生", "暴雨调度", "避开事故"],
            continuity_facts: &[
                ("林舟", "使用", "主角使用未来经验", 5),
                ("外卖站", "确认", "站点责任归属", 5),
                ("林舟与许蔓", "发生", "关键人物关系变化", 4),
            ],
            style_note: "保留商业决策逻辑、主角主动性和章尾钩子",
        },
        FixtureKind::Romance => FixtureTemplate {
            target_platform: "fanqie",
            title: "签约前夜",
            genre: "现代女性向",
            tags: &["重生", "复仇", "逆袭", "女性向"],
            selling_points: &["背叛节点", "证据链", "事业反击"],
            reader_expectations: &["情绪爽点", "清醒反制", "事业重建"],
            emotional_hooks: &["背叛后的反击", "重新掌控公司"],
            first_scene: "股权转让协议前夜",
            first_conflict: "未婚夫已经布好局逼女主签字",
            first_three_chapters_goal: "稳住对手，暗中取证，签约现场反击",
            chapter_title: "第一章 签字前夜",
            plot_goal: "回到签约前，确认股权转让危机并决定反制",
            plot_conflict: "未婚夫布局和闺蜜背叛同时压迫女主",
            key_events: &["回到签约前", "未婚夫布局", "女主决定反制"],
            main_conflict: "姜晚要依靠证据链夺回股权和舆论主动权",
            protagonist_goal: "保住公司控制权并筛掉背叛关系",
            antagonistic_force: "陆承泽和宋晴的股权局",
            long_term_hook: "幕后资金方真实目的",
            protagonist_name: "姜晚",
            world_overview: "公司股权结构、证据链来源、舆论和资金压力决定复仇节奏。",
            world_name: "股权复仇线",
            world_levels: &["协议前夜", "财务异常", "签约现场", "资金方露面"],
            world_rules: &["公司股权结构必须清晰", "证据链来源必须能被追溯", "舆论和资金压力会反过来影响谈判"],
            world_costs: &["股权和资金操作不能随意跳步", "每次情绪反击都要消耗证据或信任筹码"],
            hard_rules: &["复仇必须依赖证据链", "股权和资金操作不能随意跳步"],
            seed_facts: &[
                ("姜晚", "回到", "姜晚回到签约前一天", 5),
                ("陆承泽", "布置", "陆承泽已经布好股权局", 5),
            ],
            body_paragraph: "姜晚回到签约前夜，桌上的股权转让危机像一把刀压在协议边缘。陆承泽温声催她休息，宋晴却提前拿走了财务权限，未婚夫和闺蜜的关系在细节里露出裂缝。姜晚没有当场翻脸，而是把隐藏证据从旧邮箱、财务备份和聊天记录里一项项扣出来。女主清醒反制依靠证据链逻辑推进，情绪反击落在她重新设定签约议程的那一刻，形成第一处情绪爽点；章尾，顾行川发来消息，提醒她资金方已经入场。\n",
            summary: "姜晚回到签约前夜，发现未婚夫已布好股权局并开始暗中取证。",
            writer_key_events: &["回到签约前", "暗中取证", "决定反制"],
            continuity_facts: &[
                ("姜晚", "确认", "股权转让时间点", 5),
                ("姜晚", "保留", "证据取得方式", 5),
                ("陆承泽与宋晴", "存在", "未婚夫和闺蜜的关系", 4),
            ],
            style_note: "保留女主清醒反制、证据链逻辑和情绪爽点",
        },
        FixtureKind::Fantasy => unreachable!("fantasy uses dedicated fixture functions"),
    }
}

fn fixture_market_json(kind: FixtureKind) -> String {
    let fixture = fixture_template(kind);
    let title_candidates = [
        fixture.title,
        match kind {
            FixtureKind::Urban => "十年前的配送单",
            FixtureKind::Romance => "她在签字前醒来",
            FixtureKind::Fantasy => unreachable!(),
        },
        match kind {
            FixtureKind::Urban => "本地生活之王",
            FixtureKind::Romance => "夺回股权后",
            FixtureKind::Fantasy => unreachable!(),
        },
    ]
    .into_iter()
    .map(|title| serde_json::json!({"title": title, "reason": "贴合样例核心卖点"}))
    .collect::<Vec<_>>();

    serde_json::json!({
        "role": "market",
        "structured": {
            "market_analysis": {
                "target_platform": fixture.target_platform,
                "genre": fixture.genre,
                "sub_genres": fixture.tags,
                "target_readers": "偏好强冲突、强目标和短周期回报的读者",
                "reader_expectations": fixture.reader_expectations,
                "core_selling_points": fixture.selling_points,
                "emotional_hooks": fixture.emotional_hooks,
                "platform_tags": fixture.tags,
                "risk_notes": []
            },
            "title_candidates": title_candidates,
            "intro_candidates": [],
            "opening_strategy": {
                "first_scene": fixture.first_scene,
                "first_conflict": fixture.first_conflict,
                "first_three_chapters_goal": fixture.first_three_chapters_goal,
                "avoid": []
            },
            "platform_profile": {
                "target_platform": fixture.target_platform,
                "opening_speed": "fast",
                "setup_ratio": 0.18,
                "dialogue_ratio": 0.42,
                "payoff_frequency": "every_chapter",
                "cliffhanger_strength": "high",
                "review_bias": {"opening_hook_score": 2, "pacing_score": 2}
            }
        },
        "raw_notes": ""
    })
    .to_string()
}

fn fixture_plot_json(kind: FixtureKind) -> String {
    let fixture = fixture_template(kind);
    let outlines = (1..=30)
        .map(|index| {
            serde_json::json!({
                "volume_index": 1,
                "chapter_index": index,
                "title": if index == 1 {
                    fixture.chapter_title.to_string()
                } else {
                    format!("第{index}章 主线推进")
                },
                "pov": "第三人称有限视角",
                "goal": fixture.plot_goal,
                "conflict": fixture.plot_conflict,
                "key_events": fixture.key_events,
                "character_changes": ["主角目标更主动，外部压力升级"],
                "new_facts": [
                    {
                        "subject": fixture.protagonist_name,
                        "predicate": "推进",
                        "object": format!("第{index}章关键事实"),
                        "importance": 2
                    }
                ],
                "foreshadowing": [fixture.long_term_hook],
                "payoff": "完成一次阶段性小回报",
                "cliffhanger": format!("{}带来新的压力", fixture.long_term_hook),
                "estimated_word_count": 2200
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "role": "plot",
        "structured": {
            "plot_plan": {
                "main_conflict": fixture.main_conflict,
                "protagonist_goal": fixture.protagonist_goal,
                "antagonistic_force": fixture.antagonistic_force,
                "long_term_hook": fixture.long_term_hook,
                "volume_plan": [],
                "foreshadowing": []
            },
            "chapter_outlines": outlines
        },
        "raw_notes": ""
    })
    .to_string()
}

fn urban_character_json() -> String {
    serde_json::json!({
        "role": "character",
        "structured": {
            "characters": [
                {
                    "id_hint": "protagonist",
                    "name": "林舟",
                    "role": "protagonist",
                    "identity": "重生回十年前的外卖站骑手",
                    "personality": ["冷静", "行动快"],
                    "desire": "从外卖站开始逆袭，建立本地生活业务",
                    "motivation": "创业失败遗憾让他必须抓住未来经验",
                    "secret": "掌握未来经验和行业节点",
                    "abilities": ["未来经验", "调度复盘", "行业判断"],
                    "limitations": ["资金不足", "人脉有限"],
                    "current_state": "刚重生，主动目标是避开事故并拿回责任主动权",
                    "relationship_map": [],
                    "arc": {
                        "start": "被站点甩锅",
                        "turning_points": ["主动调整调度", "获得许蔓信任"],
                        "expected_end": "建立第一条本地生活业务线"
                    },
                    "first_appearance_chapter": 1,
                    "chapter_1_to_30_plan": ["前三章确立主动目标", "十章内拿到试点机会"]
                },
                {
                    "id_hint": "antagonist_primary",
                    "name": "陈岳",
                    "role": "antagonist",
                    "identity": "未来竞争对手",
                    "personality": ["擅长包装", "逐利"],
                    "desire": "抢占本地生活试点",
                    "motivation": "利用资本资源复制林舟方案",
                    "secret": "提前接触未来巨头渠道",
                    "abilities": ["资本包装", "关系运作"],
                    "limitations": ["不了解底层站点真实规则"],
                    "current_state": "注意到林舟的异常调度",
                    "relationship_map": [],
                    "arc": {
                        "start": "暗中观察",
                        "turning_points": ["抢同一条试点线"],
                        "expected_end": "成为商业主线对手"
                    },
                    "first_appearance_chapter": 3,
                    "chapter_1_to_30_plan": ["前三章露出关注", "中段正面争夺试点"]
                }
            ],
            "relationship_overview": "",
            "consistency_rules": [],
            "risk_notes": []
        },
        "raw_notes": ""
    })
    .to_string()
}

fn romance_character_json() -> String {
    serde_json::json!({
        "role": "character",
        "structured": {
            "characters": [
                {
                    "id_hint": "protagonist",
                    "name": "姜晚",
                    "role": "protagonist",
                    "identity": "重生回签约前一天的公司创始人",
                    "personality": ["清醒", "克制", "果断"],
                    "desire": "保住公司控制权并完成事业目标",
                    "motivation": "主动取证后清醒反制背叛者",
                    "secret": "保留前世被夺公司的记忆",
                    "abilities": ["主动取证", "舆论判断", "谈判"],
                    "limitations": ["股权局已成形", "资金压力逼近"],
                    "current_state": "回到签约前，事业目标和反制计划已确立",
                    "relationship_map": [],
                    "arc": {
                        "start": "被背叛后重来",
                        "turning_points": ["拿到第一份证据", "签约现场反击"],
                        "expected_end": "重建事业和信任边界"
                    },
                    "first_appearance_chapter": 1,
                    "chapter_1_to_30_plan": ["前三章完成第一轮证据反击", "中段锁定幕后资金方"]
                },
                {
                    "id_hint": "antagonist_fiance",
                    "name": "陆承泽",
                    "role": "antagonist",
                    "identity": "未婚夫",
                    "personality": ["温和表象", "擅长操控"],
                    "desire": "夺走姜晚公司控制权",
                    "motivation": "通过股权转让拿到资本筹码",
                    "secret": "与宋晴联手隐藏资金方条件",
                    "abilities": ["情感操控", "资本包装"],
                    "limitations": ["证据链一旦公开会失去谈判主动"],
                    "current_state": "已经布好股权局",
                    "relationship_map": [],
                    "arc": {
                        "start": "自以为掌控局面",
                        "turning_points": ["签约现场被证据打乱"],
                        "expected_end": "暴露幕后合作"
                    },
                    "first_appearance_chapter": 1,
                    "chapter_1_to_30_plan": ["前三章持续压迫签约", "中段被迫应对证据链"]
                },
                {
                    "id_hint": "ally_investor",
                    "name": "顾行川",
                    "role": "ally",
                    "identity": "投资人",
                    "personality": ["审慎", "利益清楚"],
                    "desire": "找到可靠合作方",
                    "motivation": "看重姜晚的反制能力和公司价值",
                    "secret": "知道幕后资金方部分动向",
                    "abilities": ["资金判断", "信息渠道"],
                    "limitations": ["不会无条件站队"],
                    "current_state": "观察姜晚如何处理签约局",
                    "relationship_map": [],
                    "arc": {
                        "start": "旁观",
                        "turning_points": ["提供资金方线索"],
                        "expected_end": "成为事业合作盟友"
                    },
                    "first_appearance_chapter": 1,
                    "chapter_1_to_30_plan": ["前三章递出线索", "中段建立合作边界"]
                }
            ],
            "relationship_overview": "",
            "consistency_rules": [],
            "risk_notes": []
        },
        "raw_notes": ""
    })
    .to_string()
}

fn fixture_worldbuilding_json(kind: FixtureKind) -> String {
    let fixture = fixture_template(kind);
    serde_json::json!({
        "role": "worldbuilding",
        "structured": {
            "world_setting": {
                "genre_type": fixture.genre,
                "overview": fixture.world_overview,
                "power_system": {
                    "name": fixture.world_name,
                    "levels": fixture.world_levels,
                    "rules": fixture.world_rules,
                    "costs": fixture.world_costs,
                    "limits": fixture.hard_rules
                },
                "organizations": [
                    {
                        "name": fixture.world_name,
                        "role": "提供外部规则和资源压力",
                        "resources": fixture.world_levels,
                        "conflicts": fixture.hard_rules
                    }
                ],
                "locations": [
                    {
                        "name": fixture.first_scene,
                        "description": fixture.first_conflict,
                        "story_use": "第一章压迫场景"
                    }
                ],
                "taboos": fixture.hard_rules,
                "hard_rules": fixture.hard_rules
            },
            "facts_to_seed": fact_values(fixture.seed_facts),
            "risk_notes": []
        },
        "raw_notes": ""
    })
    .to_string()
}

fn fixture_writer_json(kind: FixtureKind) -> String {
    let fixture = fixture_template(kind);
    let content = fixture.body_paragraph.repeat(12);
    let word_count = count_chars(&content);
    serde_json::json!({
        "role": "writer",
        "structured": {
            "chapter_draft": {
                "volume_index": 1,
                "chapter_index": 1,
                "title": fixture.chapter_title,
                "content": content,
                "summary": fixture.summary,
                "word_count": word_count,
                "pov": "第三人称有限视角",
                "key_events": fixture.writer_key_events,
                "new_facts": fact_values(fixture.continuity_facts),
                "foreshadowing": [
                    {"seed": fixture.long_term_hook, "status": "planted", "expected_payoff": "第一卷后段揭示"}
                ],
                "continuity_notes": ["样例事实需要持续追踪"]
            }
        },
        "raw_notes": ""
    })
    .to_string()
}

fn fixture_continuity_json(kind: FixtureKind) -> String {
    let fixture = fixture_template(kind);
    serde_json::json!({
        "role": "continuity",
        "structured": {
            "continuity_report": {
                "passed": true,
                "issues": [],
                "new_facts": fact_values(fixture.continuity_facts),
                "character_state_updates": [
                    {
                        "character": fixture.protagonist_name,
                        "before": "被动承压",
                        "after": "主动制定第一步反击",
                        "reason": fixture.plot_goal
                    }
                ],
                "foreshadowing_updates": [
                    {
                        "seed": fixture.long_term_hook,
                        "status": "planted",
                        "note": "后续章节回收"
                    }
                ]
            }
        },
        "raw_notes": ""
    })
    .to_string()
}

fn fixture_style_json(kind: FixtureKind) -> String {
    let fixture = fixture_template(kind);
    let content = fixture.body_paragraph.repeat(12);
    serde_json::json!({
        "role": "style",
        "structured": {
            "styled_chapter": {
                "title": fixture.chapter_title,
                "content": content,
                "summary": fixture.summary,
                "changes": [
                    {
                        "type": "pacing",
                        "description": "压缩解释，强化行动、对白和场景压力。"
                    }
                ],
                "preserved_facts": fixture.writer_key_events,
                "style_notes": [fixture.style_note]
            }
        },
        "raw_notes": ""
    })
    .to_string()
}

fn fact_values(
    facts: &[(&'static str, &'static str, &'static str, i32)],
) -> Vec<serde_json::Value> {
    facts
        .iter()
        .map(|(subject, predicate, object, importance)| {
            serde_json::json!({
                "subject": subject,
                "predicate": predicate,
                "object": object,
                "importance": importance
            })
        })
        .collect()
}

fn market_json() -> String {
    r#"{
      "role": "market",
      "structured": {
        "market_analysis": {
          "target_platform": "qidian",
          "genre": "玄幻",
          "sub_genres": ["升级"],
          "target_readers": "偏好体系感的读者",
          "reader_expectations": ["能力代价", "长期伏笔"],
          "core_selling_points": ["能力代价", "长期伏笔", "阶段闭环"],
          "emotional_hooks": ["救亲人后的代价"],
          "platform_tags": ["玄幻", "升级", "因果", "古塔"],
          "risk_notes": []
        },
        "title_candidates": [
          {"title": "因果塔债", "reason": "突出核心设定"},
          {"title": "债印登天", "reason": "突出升级"},
          {"title": "古塔问因果", "reason": "突出长线悬念"}
        ],
        "intro_candidates": [],
        "opening_strategy": {
          "first_scene": "妖潮压城",
          "first_conflict": "救妹妹必须背债",
          "first_three_chapters_goal": "确立古塔规则",
          "avoid": []
        },
        "platform_profile": {
          "target_platform": "qidian",
          "opening_speed": "layered",
          "setup_ratio": 0.35,
          "dialogue_ratio": 0.3,
          "payoff_frequency": "every_2_chapters",
          "cliffhanger_strength": "medium",
          "review_bias": {"continuity_score": 2}
        }
      },
      "raw_notes": ""
    }"#
    .to_string()
}

fn plot_json() -> String {
    let outlines = (1..=30)
        .map(|index| {
            format!(
                r#"{{
                  "volume_index": 1,
                  "chapter_index": {index},
                  "title": "第{index}章 因果推进",
                  "pov": "第三人称有限视角",
                  "goal": "救妹妹并确认第一笔因果债的偿还目标",
                  "conflict": "妖潮压城，能力代价压迫主角",
                  "key_events": ["妖潮围城", "救妹妹", "触发古塔", "背上第一笔因果债"],
                  "character_changes": ["主角更清楚代价"],
                  "new_facts": [
                    {{"subject":"沈砚","predicate":"背负","object":"第{index}笔因果债","importance":2}}
                  ],
                  "foreshadowing": ["古塔主人线索"],
                  "payoff": "获得阶段信息",
                  "cliffhanger": "债印出现异常",
                  "estimated_word_count": 2500
                }}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r#"{{
          "role": "plot",
          "structured": {{
            "plot_plan": {{
              "main_conflict": "因果债与守城压力",
              "protagonist_goal": "救下妹妹并查清古塔",
              "antagonistic_force": "妖潮与宗门试探",
              "long_term_hook": "古塔真正主人",
              "volume_plan": [],
              "foreshadowing": []
            }},
            "chapter_outlines": [{outlines}]
          }},
          "raw_notes": ""
        }}"#
    )
}

fn character_json() -> String {
    r#"{
      "role": "character",
      "structured": {
        "characters": [
          {
            "id_hint": "protagonist",
            "name": "沈砚",
            "role": "protagonist",
            "identity": "边城少年",
            "personality": ["谨慎", "重情", "有底线"],
            "desire": "救下妹妹并查清古塔真正主人",
            "motivation": "救亲人动机明确，守住家人",
            "secret": "与古塔有关",
            "abilities": ["因果债感知"],
            "limitations": ["能力限制明确，借力必须承担因果债"],
            "current_state": "刚触发古塔",
            "relationship_map": [],
            "arc": {
              "start": "被动守城",
              "turning_points": ["背负第一笔债"],
              "expected_end": "主动承担因果"
            },
            "first_appearance_chapter": 1,
            "chapter_1_to_30_plan": ["确立规则"]
          },
          {
            "id_hint": "antagonist_primary",
            "name": "赤鳞妖王",
            "role": "antagonist",
            "identity": "妖潮首领",
            "personality": ["残忍", "狡诈"],
            "desire": "攻破边城",
            "motivation": "夺取古塔碎片线索",
            "secret": "受未知势力驱使",
            "abilities": ["统御妖潮"],
            "limitations": ["无法直接进入古塔结界"],
            "current_state": "第一卷外部压力来源",
            "relationship_map": [],
            "arc": {
              "start": "压迫边城",
              "turning_points": ["发现沈砚能借古塔之力"],
              "expected_end": "被迫暴露幕后线索"
            },
            "first_appearance_chapter": 1,
            "chapter_1_to_30_plan": ["持续制造妖潮压力"]
          },
          {
            "id_hint": "supporting_sister",
            "name": "沈青禾",
            "role": "supporting",
            "identity": "沈砚妹妹",
            "personality": ["坚韧", "信任兄长"],
            "desire": "活下去并帮助沈砚",
            "motivation": "不拖累家人",
            "secret": "体内藏有古塔碎片线索",
            "abilities": ["感知古塔碎片"],
            "limitations": ["当前状态虚弱"],
            "current_state": "被妖潮重伤，需要救治",
            "relationship_map": [],
            "arc": {
              "start": "被保护者",
              "turning_points": ["显露碎片线索"],
              "expected_end": "成为解开古塔秘密的关键"
            },
            "first_appearance_chapter": 1,
            "chapter_1_to_30_plan": ["推动古塔碎片线索"]
          }
        ],
        "relationship_overview": "",
        "consistency_rules": [],
        "risk_notes": []
      },
      "raw_notes": ""
    }"#
    .to_string()
}

fn worldbuilding_json() -> String {
    r#"{
      "role": "worldbuilding",
      "structured": {
        "world_setting": {
          "genre_type": "玄幻",
          "overview": "边城妖潮压力下，古塔层级以因果债规则驱动，借力者必须偿还代价。",
          "power_system": {
            "name": "因果债",
            "levels": ["债印初醒", "借力还债"],
            "rules": ["借力必须偿还因果", "每次借用古塔力量都必须形成可追踪债务"],
            "costs": ["能力突破必须有代价", "债务积累会带来反噬"],
            "limits": ["不能无代价改写已发生的因果"]
          },
          "organizations": [
            {
              "name": "边城守备营",
              "role": "提供外部压力和秩序约束",
              "resources": ["城防", "巡逻队"],
              "conflicts": ["妖潮压城", "资源不足"]
            }
          ],
          "locations": [
            {
              "name": "边城北墙",
              "description": "妖潮最先冲击的城防缺口",
              "story_use": "开篇压迫场景"
            }
          ],
          "taboos": ["不可逃避已确认因果债"],
          "hard_rules": ["借力必须偿还因果", "能力突破必须有代价", "借力必须留下债务记录"]
        },
        "facts_to_seed": [
          {"subject":"古塔","predicate":"记录","object":"古塔记录因果债","importance":5},
          {"subject":"沈砚","predicate":"第一次借力","object":"沈砚第一次借力会留下债印","importance":5}
        ],
        "risk_notes": []
      },
      "raw_notes": ""
    }"#.to_string()
}

fn writer_json() -> String {
    let content = fantasy_chapter_body();
    let word_count = count_chars(&content);
    serde_json::json!({
        "role": "writer",
        "structured": {
            "chapter_draft": {
                "volume_index": 1,
                "chapter_index": 1,
                "title": "第一章 债印初醒",
                "content": content,
                "summary": "沈砚为救妹妹触发古塔，背上第一笔因果债。",
                "word_count": word_count,
                "pov": "第三人称有限视角",
                "key_events": ["妖潮围城", "触发古塔", "战斗策略", "背上第一笔因果债"],
                "new_facts": [
                    {"subject":"沈砚","predicate":"背负","object":"第一笔因果债","importance":5},
                    {"subject":"古塔","predicate":"要求","object":"古塔借力代价","importance":5},
                    {"subject":"沈青禾","predicate":"状态","object":"妹妹状态危急但稳定","importance":4}
                ],
                "foreshadowing": [
                    {"seed":"古塔主人长期伏笔","status":"planted","expected_payoff":"第一卷后段揭示"}
                ],
                "continuity_notes": ["第一笔因果债需要持续追踪"]
            }
        },
        "raw_notes": ""
    })
    .to_string()
}

fn continuity_json() -> String {
    r#"{
      "role": "continuity",
      "structured": {
        "continuity_report": {
          "passed": true,
          "issues": [],
          "new_facts": [
            {"subject":"沈砚","predicate":"背负","object":"第一笔因果债","importance":5},
            {"subject":"古塔","predicate":"要求","object":"古塔借力代价","importance":5},
            {"subject":"沈青禾","predicate":"状态","object":"妹妹状态危急但稳定","importance":4}
          ],
          "character_state_updates": [
            {
              "character": "沈砚",
              "before": "刚触发古塔",
              "after": "背负第一笔因果债",
              "reason": "为救妹妹借用古塔力量"
            }
          ],
          "foreshadowing_updates": [
            {
              "seed": "古塔主人",
              "status": "planted",
              "note": "第一卷后段揭示"
            }
          ]
        }
      },
      "raw_notes": ""
    }"#
    .to_string()
}

fn style_json() -> String {
    let content = fantasy_chapter_body();
    serde_json::json!({
        "role": "style",
        "structured": {
            "styled_chapter": {
                "title": "第一章 债印初醒",
                "content": content,
                "summary": "沈砚为救妹妹触发古塔，背上第一笔因果债。",
                "changes": [
                    {
                        "type": "pacing",
                        "description": "减少设定堆叠，强化战斗画面和能力代价。"
                    }
                ],
                "preserved_facts": ["因果债规则", "战斗局势变化", "长期伏笔"],
                "style_notes": ["保留因果代价的压迫感"]
            }
        },
        "raw_notes": ""
    })
    .to_string()
}

fn fantasy_chapter_body() -> String {
    let paragraph = "妖潮撞上边城北墙时，沈砚先看风向、缺口和守军退路，用战斗策略把赤鳞妖群引到塌陷的箭楼下。沈青禾的妹妹状态仍然危险，古塔在他掌心亮起债印，因果债规则清楚提醒他：每一次借力都有能力代价。沈砚借来短暂力量救妹妹，也把第一笔因果债刻进掌心。战斗局势变化后，城墙暂时守住，但古塔主人长期伏笔浮出水面，说明还有更大因果问题在等他偿还。\n";
    paragraph.repeat(18)
}

fn count_chars(content: &str) -> u32 {
    content.chars().filter(|ch| !ch.is_whitespace()).count() as u32
}

fn rewrite_writer_json() -> String {
    let content = format!(
        "{}\n重写后强化目标：沈砚把救妹妹、偿还第一笔因果债和查清古塔主人三件事压成同一条行动线，章尾钩子更明确。",
        fantasy_chapter_body()
    );
    let word_count = count_chars(&content);
    serde_json::json!({
        "role": "writer",
        "structured": {
            "chapter_draft": {
                "volume_index": 1,
                "chapter_index": 1,
                "title": "第一章 债印初醒",
                "content": content,
                "summary": "沈砚在重写稿中强化目标、冲突和章尾钩子。",
                "word_count": word_count,
                "pov": "第三人称有限视角",
                "key_events": ["重写后强化目标", "背上第一笔因果债"],
                "new_facts": [
                    {"subject":"沈砚","predicate":"强化","object":"重写后强化目标","importance":4}
                ],
                "foreshadowing": [
                    {"seed":"古塔主人长期伏笔","status":"advanced","expected_payoff":"第一卷后段揭示"}
                ],
                "continuity_notes": ["重写后继续追踪第一笔因果债"]
            }
        },
        "raw_notes": ""
    })
    .to_string()
}

fn rewrite_style_json() -> String {
    let content = format!(
        "{}\n重写后强化目标：沈砚的行动线更集中，章尾直接指向下一笔因果债。",
        fantasy_chapter_body()
    );
    serde_json::json!({
        "role": "style",
        "structured": {
            "styled_chapter": {
                "title": "第一章 债印初醒",
                "content": content,
                "summary": "沈砚在重写稿中强化目标、冲突和章尾钩子。",
                "changes": [
                    {
                        "type": "clarity",
                        "description": "强化目标线和章尾压力。"
                    }
                ],
                "preserved_facts": ["重写后强化目标", "第一笔因果债"],
                "style_notes": ["保留因果债事实"]
            }
        },
        "raw_notes": ""
    })
    .to_string()
}

fn low_score_reviewer_json() -> String {
    r#"{
      "role": "reviewer",
      "structured": {
        "review_report": {
          "total_score": 68,
          "passed": false,
          "scores": {
            "opening_hook_score": 7,
            "pacing_score": 6,
            "payoff_score": 7,
            "character_score": 7,
            "dialogue_score": 6,
            "continuity_score": 7,
            "cliffhanger_score": 6,
            "platform_fit_score": 7
          },
          "strengths": ["设定清楚"],
          "issues": [
            {
              "severity": "medium",
              "dimension": "pacing",
              "location": "整章",
              "description": "目标线和章尾钩子仍需强化。"
            }
          ],
          "suggestions": ["强化目标、冲突和章尾钩子"],
          "rewrite_instruction": {
            "needed": true,
            "rewrite_type": "partial",
            "priority": "medium",
            "goals": ["强化目标、冲突和章尾钩子"],
            "preserve": ["第一笔因果债", "古塔主人长期伏笔"],
            "change": ["压缩解释性段落", "增加主角主动行动"],
            "avoid": ["只增加设定解释"]
          }
        }
      },
      "raw_notes": ""
    }"#
    .to_string()
}

fn reviewer_json() -> String {
    r#"{
      "role": "reviewer",
      "structured": {
        "review_report": {
          "total_score": 82,
          "passed": true,
          "scores": {
            "opening_hook_score": 8,
            "pacing_score": 8,
            "payoff_score": 8,
            "character_score": 8,
            "dialogue_score": 7,
            "continuity_score": 8,
            "cliffhanger_score": 8,
            "platform_fit_score": 8
          },
          "strengths": ["目标清楚"],
          "issues": [],
          "suggestions": ["下一章继续强化代价"],
          "rewrite_instruction": {
            "needed": false,
            "rewrite_type": "none",
            "priority": "low",
            "goals": [],
            "preserve": [],
            "change": [],
            "avoid": []
          }
        }
      },
      "raw_notes": ""
    }"#
    .to_string()
}
