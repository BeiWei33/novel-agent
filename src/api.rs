use std::convert::Infallible;
use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post, put},
    Json, Router,
};
use futures_util::{stream, Stream};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

use crate::agents::ModelHandle;
use crate::domain::{
    Chapter, ChapterDraft, ChapterOutline, CharacterCard, Fact, Novel, NovelBible, NovelId,
    ReviewReport, TargetPlatform,
};
use crate::error::{StorageError, WorkflowError};
use crate::storage::{
    AgentRunRecord, AgentRunStatus, AgentRunStatusSummary, JobRecord, JobStatus, SqliteStorage,
};
use crate::workflow::{ChapterGenerationWorkflow, NovelCreationWorkflow};

#[derive(Clone)]
pub struct ApiState {
    storage: SqliteStorage,
    model: ModelHandle,
}

pub fn router(storage: SqliteStorage, model: ModelHandle) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/jobs", get(list_jobs))
        .route("/api/jobs/{job_id}", get(get_job))
        .route("/api/jobs/{job_id}/retry", post(retry_job))
        .route("/api/jobs/{job_id}/cancel", post(cancel_job))
        .route("/api/runs", get(list_all_agent_runs))
        .route("/api/novels", get(list_novels).post(create_novel))
        .route("/api/novels/jobs", post(create_novel_job))
        .route("/api/novels/{novel_id}", get(get_novel))
        .route("/api/novels/{novel_id}/bible", get(get_bible))
        .route("/api/novels/{novel_id}/characters", get(list_characters))
        .route("/api/novels/{novel_id}/facts", get(list_facts))
        .route("/api/novels/{novel_id}/outline", post(generate_outline))
        .route(
            "/api/novels/{novel_id}/world-settings",
            get(get_world_setting),
        )
        .route("/api/novels/{novel_id}/chapters", get(list_chapters))
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}",
            get(get_chapter),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/edit",
            put(save_chapter_edit),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/write",
            post(write_chapter),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/write/jobs",
            post(write_chapter_job),
        )
        .route(
            "/api/novels/{novel_id}/chapters/write/jobs",
            post(write_chapters_job),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/write/stream",
            post(write_chapter_stream),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/review",
            get(get_latest_review).post(review_chapter),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/continuity",
            get(get_latest_continuity),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/review/jobs",
            post(review_chapter_job),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/rewrite",
            post(rewrite_chapter),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/rewrite/jobs",
            post(rewrite_chapter_job),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/rewrite/stream",
            post(rewrite_chapter_stream),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/versions",
            get(list_versions),
        )
        .route(
            "/api/novels/{novel_id}/chapters/{chapter_index}/versions/{version}",
            get(get_version),
        )
        .route(
            "/api/novels/{novel_id}/export/markdown",
            get(export_markdown),
        )
        .route("/api/novels/{novel_id}/runs", get(list_agent_runs))
        .with_state(ApiState { storage, model })
        .layer(CorsLayer::permissive())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

async fn list_jobs(
    State(state): State<ApiState>,
    Query(query): Query<JobsQuery>,
) -> Result<Json<JobsResponse>, ApiError> {
    let status = query
        .status
        .as_deref()
        .map(JobStatus::from_str)
        .transpose()?;
    let kind = query
        .kind
        .as_deref()
        .map(str::trim)
        .filter(|kind| !kind.is_empty());
    let novel_id = query
        .novel_id
        .as_deref()
        .map(str::trim)
        .filter(|novel_id| !novel_id.is_empty());
    let source_job_id = query
        .source_job_id
        .as_deref()
        .map(str::trim)
        .filter(|source_job_id| !source_job_id.is_empty());
    let jobs = state
        .storage
        .jobs()
        .list_recent_filtered(
            capped_limit(query.limit, 50),
            status,
            kind,
            novel_id,
            source_job_id,
        )
        .await?;
    Ok(Json(JobsResponse { jobs }))
}

async fn get_job(
    State(state): State<ApiState>,
    Path(job_id): Path<String>,
) -> Result<Json<JobResponse>, ApiError> {
    let job = state
        .storage
        .jobs()
        .find(&job_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("job `{job_id}` was not found")))?;
    Ok(Json(JobResponse { job }))
}

async fn retry_job(
    State(state): State<ApiState>,
    Path(job_id): Path<String>,
) -> Result<(StatusCode, Json<JobResponse>), ApiError> {
    let source = state
        .storage
        .jobs()
        .find(&job_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("job `{job_id}` was not found")))?;
    if source.status != JobStatus::Failed {
        return Err(ApiError::bad_request(format!(
            "job `{job_id}` cannot be retried unless its status is failed"
        )));
    }

    let job = match source.kind.as_str() {
        "create_novel" => retry_create_novel_job(&state, &source).await?,
        "write_chapter" => retry_chapter_job(&state, &source, "write_chapter").await?,
        "write_chapters" => retry_write_chapters_job(&state, &source).await?,
        "review_chapter" => retry_chapter_job(&state, &source, "review_chapter").await?,
        "rewrite_chapter" => retry_chapter_job(&state, &source, "rewrite_chapter").await?,
        other => {
            return Err(ApiError::bad_request(format!(
                "job kind `{other}` cannot be retried"
            )))
        }
    };

    Ok((StatusCode::ACCEPTED, Json(JobResponse { job })))
}

async fn cancel_job(
    State(state): State<ApiState>,
    Path(job_id): Path<String>,
) -> Result<Json<JobResponse>, ApiError> {
    let job = state
        .storage
        .jobs()
        .find(&job_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("job `{job_id}` was not found")))?;
    if !matches!(job.status, JobStatus::Queued | JobStatus::Running) {
        return Err(ApiError::bad_request(format!(
            "job `{job_id}` cannot be cancelled from status `{}`",
            job.status.as_str()
        )));
    }

    let cancelled = state
        .storage
        .jobs()
        .cancel(&job_id, "job cancelled by user")
        .await?;
    if !cancelled {
        let current = state
            .storage
            .jobs()
            .find(&job_id)
            .await?
            .ok_or_else(|| ApiError::not_found(format!("job `{job_id}` was not found")))?;
        return Err(ApiError::bad_request(format!(
            "job `{job_id}` cannot be cancelled from status `{}`",
            current.status.as_str()
        )));
    }

    let job = state
        .storage
        .jobs()
        .find(&job_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("job `{job_id}` was not found")))?;
    Ok(Json(JobResponse { job }))
}

async fn list_novels(
    State(state): State<ApiState>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<NovelsResponse>, ApiError> {
    let novels = state
        .storage
        .novels()
        .list_recent(capped_limit(query.limit, 50))
        .await?;
    Ok(Json(NovelsResponse { novels }))
}

async fn create_novel(
    State(state): State<ApiState>,
    Json(request): Json<CreateNovelRequest>,
) -> Result<(StatusCode, Json<CreateNovelResponse>), ApiError> {
    let idea = require_non_empty(&request.idea, "idea")?;
    let platform = parse_platform(request.platform.as_deref())?;
    let workflow = NovelCreationWorkflow::new(&state.storage, state.model.clone());
    let result = workflow
        .create_from_idea_with_outline_batch_size(
            idea,
            platform,
            request.chapters.unwrap_or(30).max(1),
            request.outline_batch_size.unwrap_or(5).max(1),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateNovelResponse {
            novel: result.novel,
            bible: result.bible,
            characters: result.characters,
            outlines: result.outlines,
            used_fallback: result.used_fallback,
        }),
    ))
}

async fn create_novel_job(
    State(state): State<ApiState>,
    Json(request): Json<CreateNovelRequest>,
) -> Result<(StatusCode, Json<JobResponse>), ApiError> {
    let idea = require_non_empty(&request.idea, "idea")?.to_string();
    let platform = parse_platform(request.platform.as_deref())?;
    let chapters = request.chapters.unwrap_or(30).max(1);
    let outline_batch_size = request.outline_batch_size.unwrap_or(5).max(1);
    let payload = json!({
        "idea": idea,
        "platform": platform.as_str(),
        "chapters": chapters,
        "outline_batch_size": outline_batch_size
    });
    let job = state
        .storage
        .jobs()
        .create("create_novel", None, None, &payload)
        .await?;
    let job_id = job.id.clone();
    spawn_create_novel_job(
        state.storage.clone(),
        state.model.clone(),
        job_id,
        idea,
        platform,
        chapters,
        outline_batch_size,
    );

    Ok((StatusCode::ACCEPTED, Json(JobResponse { job })))
}

async fn get_novel(
    State(state): State<ApiState>,
    Path(novel_id): Path<String>,
) -> Result<Json<NovelDetailResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let novel = state
        .storage
        .novels()
        .find(&novel_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("novel `{novel_id}` was not found")))?;
    let bible = state.storage.novels().find_bible(&novel_id).await?;
    let characters = state.storage.characters().list_by_novel(&novel_id).await?;
    let chapters = state.storage.chapters().list_by_novel(&novel_id).await?;
    let world_setting = state.storage.world_settings().find(&novel_id).await?;
    let facts = state.storage.facts().list_by_novel(&novel_id, 100).await?;

    Ok(Json(NovelDetailResponse {
        novel,
        bible,
        characters,
        chapters,
        world_setting,
        facts,
    }))
}

async fn get_bible(
    State(state): State<ApiState>,
    Path(novel_id): Path<String>,
) -> Result<Json<BibleResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    ensure_novel_exists(&state.storage, &novel_id).await?;
    let bible = state.storage.novels().find_bible(&novel_id).await?;
    Ok(Json(BibleResponse { bible }))
}

async fn list_characters(
    State(state): State<ApiState>,
    Path(novel_id): Path<String>,
) -> Result<Json<CharactersResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    ensure_novel_exists(&state.storage, &novel_id).await?;
    let characters = state.storage.characters().list_by_novel(&novel_id).await?;
    Ok(Json(CharactersResponse { characters }))
}

async fn get_world_setting(
    State(state): State<ApiState>,
    Path(novel_id): Path<String>,
) -> Result<Json<WorldSettingResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    ensure_novel_exists(&state.storage, &novel_id).await?;
    let world_setting = state.storage.world_settings().find(&novel_id).await?;
    Ok(Json(WorldSettingResponse { world_setting }))
}

async fn list_facts(
    State(state): State<ApiState>,
    Path(novel_id): Path<String>,
    Query(query): Query<LimitQuery>,
) -> Result<Json<FactsResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    ensure_novel_exists(&state.storage, &novel_id).await?;
    let facts = state
        .storage
        .facts()
        .list_by_novel(&novel_id, capped_limit(query.limit, 100))
        .await?;
    Ok(Json(FactsResponse { facts }))
}

async fn generate_outline(
    State(state): State<ApiState>,
    Path(novel_id): Path<String>,
    Json(request): Json<OutlineRequest>,
) -> Result<Json<OutlineResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let workflow = NovelCreationWorkflow::new(&state.storage, state.model.clone());
    let outlines = workflow
        .generate_outline_with_batch_size(
            &novel_id,
            request.chapters.unwrap_or(30).max(1),
            request.batch_size.unwrap_or(5).max(1),
        )
        .await?;
    Ok(Json(OutlineResponse { outlines }))
}

async fn list_chapters(
    State(state): State<ApiState>,
    Path(novel_id): Path<String>,
) -> Result<Json<ChaptersResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    ensure_novel_exists(&state.storage, &novel_id).await?;
    let chapters = state.storage.chapters().list_by_novel(&novel_id).await?;
    Ok(Json(ChaptersResponse { chapters }))
}

async fn get_chapter(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<Json<ChapterResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let chapter = find_chapter(&state.storage, &novel_id, chapter_index).await?;
    Ok(Json(ChapterResponse { chapter }))
}

async fn save_chapter_edit(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
    Json(request): Json<ManualEditChapterRequest>,
) -> Result<Json<ChapterDraftResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let workflow = ChapterGenerationWorkflow::new(&state.storage, state.model.clone());
    let draft = workflow
        .save_manual_edit(
            &novel_id,
            chapter_index,
            request.title,
            request.content,
            request.summary,
        )
        .await?;
    Ok(Json(ChapterDraftResponse { draft }))
}

async fn write_chapter(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<Json<ChapterDraftResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let workflow = ChapterGenerationWorkflow::new(&state.storage, state.model.clone());
    let draft = workflow.write_chapter(&novel_id, chapter_index).await?;
    Ok(Json(ChapterDraftResponse { draft }))
}

async fn write_chapter_job(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<(StatusCode, Json<JobResponse>), ApiError> {
    let novel_id = NovelId::from(novel_id);
    ensure_novel_exists(&state.storage, &novel_id).await?;
    let payload = json!({
        "novel_id": novel_id,
        "chapter_index": chapter_index
    });
    let job = state
        .storage
        .jobs()
        .create(
            "write_chapter",
            Some(&novel_id),
            Some(chapter_index),
            &payload,
        )
        .await?;
    let job_id = job.id.clone();
    spawn_write_chapter_job(
        state.storage.clone(),
        state.model.clone(),
        job_id,
        novel_id,
        chapter_index,
    );

    Ok((StatusCode::ACCEPTED, Json(JobResponse { job })))
}

async fn write_chapters_job(
    State(state): State<ApiState>,
    Path(novel_id): Path<String>,
    Json(request): Json<BatchWriteChaptersRequest>,
) -> Result<(StatusCode, Json<JobResponse>), ApiError> {
    let novel_id = NovelId::from(novel_id);
    ensure_novel_exists(&state.storage, &novel_id).await?;
    let chapter_indexes = chapter_range(request.chapter_start, request.chapter_end)?;
    let payload = json!({
        "novel_id": novel_id,
        "chapter_start": request.chapter_start,
        "chapter_end": request.chapter_end,
        "chapter_indexes": chapter_indexes
    });
    let job = state
        .storage
        .jobs()
        .create_with_source_and_progress(
            "write_chapters",
            Some(&novel_id),
            None,
            &payload,
            None,
            0,
            chapter_indexes.len() as u32,
        )
        .await?;
    spawn_write_chapters_job(
        state.storage.clone(),
        state.model.clone(),
        job.id.clone(),
        novel_id,
        request.chapter_start,
        request.chapter_end,
    );

    Ok((StatusCode::ACCEPTED, Json(JobResponse { job })))
}

async fn write_chapter_stream(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let workflow = ChapterGenerationWorkflow::new(&state.storage, state.model.clone());
    let draft = workflow.write_chapter(&novel_id, chapter_index).await?;
    Ok(Sse::new(draft_sse_stream("write", draft)).keep_alive(KeepAlive::default()))
}

async fn review_chapter(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let workflow = ChapterGenerationWorkflow::new(&state.storage, state.model.clone());
    let report = workflow.review_chapter(&novel_id, chapter_index).await?;
    Ok(Json(ReviewResponse { report }))
}

async fn review_chapter_job(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<(StatusCode, Json<JobResponse>), ApiError> {
    let novel_id = NovelId::from(novel_id);
    find_chapter(&state.storage, &novel_id, chapter_index).await?;
    let payload = json!({
        "novel_id": novel_id,
        "chapter_index": chapter_index
    });
    let job = state
        .storage
        .jobs()
        .create(
            "review_chapter",
            Some(&novel_id),
            Some(chapter_index),
            &payload,
        )
        .await?;
    let job_id = job.id.clone();
    spawn_review_chapter_job(
        state.storage.clone(),
        state.model.clone(),
        job_id,
        novel_id,
        chapter_index,
    );

    Ok((StatusCode::ACCEPTED, Json(JobResponse { job })))
}

async fn rewrite_chapter(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<Json<ChapterDraftResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let workflow = ChapterGenerationWorkflow::new(&state.storage, state.model.clone());
    let draft = workflow.rewrite_chapter(&novel_id, chapter_index).await?;
    Ok(Json(ChapterDraftResponse { draft }))
}

async fn rewrite_chapter_job(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<(StatusCode, Json<JobResponse>), ApiError> {
    let novel_id = NovelId::from(novel_id);
    find_chapter(&state.storage, &novel_id, chapter_index).await?;
    let payload = json!({
        "novel_id": novel_id,
        "chapter_index": chapter_index
    });
    let job = state
        .storage
        .jobs()
        .create(
            "rewrite_chapter",
            Some(&novel_id),
            Some(chapter_index),
            &payload,
        )
        .await?;
    let job_id = job.id.clone();
    spawn_rewrite_chapter_job(
        state.storage.clone(),
        state.model.clone(),
        job_id,
        novel_id,
        chapter_index,
    );

    Ok((StatusCode::ACCEPTED, Json(JobResponse { job })))
}

async fn rewrite_chapter_stream(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let workflow = ChapterGenerationWorkflow::new(&state.storage, state.model.clone());
    let draft = workflow.rewrite_chapter(&novel_id, chapter_index).await?;
    Ok(Sse::new(draft_sse_stream("rewrite", draft)).keep_alive(KeepAlive::default()))
}

async fn get_latest_review(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<Json<LatestReviewResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let chapter = find_chapter(&state.storage, &novel_id, chapter_index).await?;
    let report = state
        .storage
        .review_reports()
        .latest_for_chapter(&chapter.id)
        .await?;
    Ok(Json(LatestReviewResponse { chapter, report }))
}

async fn get_latest_continuity(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<Json<LatestContinuityResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let chapter = find_chapter(&state.storage, &novel_id, chapter_index).await?;
    let report = state
        .storage
        .continuity_reports()
        .latest_for_chapter(&chapter.id)
        .await?;
    Ok(Json(LatestContinuityResponse { chapter, report }))
}

async fn list_versions(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index)): Path<(String, u32)>,
) -> Result<Json<ChapterVersionsResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let chapter = find_chapter(&state.storage, &novel_id, chapter_index).await?;
    let versions = state
        .storage
        .chapter_versions()
        .list_version_numbers(&chapter.id)
        .await?;
    Ok(Json(ChapterVersionsResponse {
        novel_id: novel_id.to_string(),
        chapter_id: chapter.id.to_string(),
        chapter_index,
        versions,
    }))
}

async fn get_version(
    State(state): State<ApiState>,
    Path((novel_id, chapter_index, version)): Path<(String, u32, u32)>,
) -> Result<Json<ChapterVersionResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let chapter = find_chapter(&state.storage, &novel_id, chapter_index).await?;
    let content = state
        .storage
        .chapter_versions()
        .content_for_version(&chapter.id, version)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(format!(
                "version v{version} for chapter {chapter_index} was not found"
            ))
        })?;

    Ok(Json(ChapterVersionResponse {
        novel_id: novel_id.to_string(),
        chapter_id: chapter.id.to_string(),
        chapter_index,
        version,
        content,
    }))
}

async fn export_markdown(
    State(state): State<ApiState>,
    Path(novel_id): Path<String>,
) -> Result<Json<ExportMarkdownResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    let workflow = ChapterGenerationWorkflow::new(&state.storage, state.model.clone());
    let markdown = workflow.export_markdown_content(&novel_id).await?;
    Ok(Json(ExportMarkdownResponse {
        novel_id: novel_id.to_string(),
        format: "markdown".to_string(),
        filename: format!("{novel_id}.md"),
        markdown,
    }))
}

async fn list_agent_runs(
    State(state): State<ApiState>,
    Path(novel_id): Path<String>,
    Query(query): Query<AgentRunsQuery>,
) -> Result<Json<AgentRunsResponse>, ApiError> {
    let novel_id = NovelId::from(novel_id);
    ensure_novel_exists(&state.storage, &novel_id).await?;
    agent_runs_response(&state, Some(&novel_id), query, 20).await
}

async fn list_all_agent_runs(
    State(state): State<ApiState>,
    Query(query): Query<AgentRunsQuery>,
) -> Result<Json<AgentRunsResponse>, ApiError> {
    let novel_id = query
        .novel_id
        .as_deref()
        .map(str::trim)
        .filter(|novel_id| !novel_id.is_empty())
        .map(NovelId::from);
    if let Some(novel_id) = &novel_id {
        ensure_novel_exists(&state.storage, novel_id).await?;
    }
    agent_runs_response(&state, novel_id.as_ref(), query, 50).await
}

async fn agent_runs_response(
    state: &ApiState,
    novel_id: Option<&NovelId>,
    query: AgentRunsQuery,
    default_limit: u32,
) -> Result<Json<AgentRunsResponse>, ApiError> {
    let limit = capped_limit(query.limit, default_limit);
    let role = query
        .role
        .as_deref()
        .map(str::trim)
        .filter(|role| !role.is_empty());
    let task = query
        .task
        .as_deref()
        .map(str::trim)
        .filter(|task| !task.is_empty());
    let status = query
        .status
        .as_deref()
        .map(parse_agent_run_status)
        .transpose()?;
    let fetch_limit = if status.is_some() { 200 } else { limit };
    let runs = state
        .storage
        .agent_runs()
        .list_recent_filtered(novel_id, fetch_limit, role, task)
        .await?;
    let runs = runs
        .into_iter()
        .filter(|run| status.map(|status| run.status() == status).unwrap_or(true))
        .take(limit as usize)
        .collect::<Vec<_>>();
    let summary = AgentRunStatusSummary::from_runs(&runs);
    let runs = runs.iter().map(AgentRunResponse::from_record).collect();
    Ok(Json(AgentRunsResponse { runs, summary }))
}

async fn ensure_novel_exists(storage: &SqliteStorage, novel_id: &NovelId) -> Result<(), ApiError> {
    storage
        .novels()
        .find(novel_id)
        .await?
        .map(|_| ())
        .ok_or_else(|| ApiError::not_found(format!("novel `{novel_id}` was not found")))
}

async fn find_chapter(
    storage: &SqliteStorage,
    novel_id: &NovelId,
    chapter_index: u32,
) -> Result<Chapter, ApiError> {
    storage
        .chapters()
        .find_by_index(novel_id, chapter_index)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(format!(
                "chapter {chapter_index} for novel `{novel_id}` was not found"
            ))
        })
}

fn capped_limit(limit: Option<u32>, default: u32) -> u32 {
    limit.unwrap_or(default).clamp(1, 200)
}

fn parse_platform(platform: Option<&str>) -> Result<TargetPlatform, ApiError> {
    TargetPlatform::from_str(platform.unwrap_or("general")).map_err(ApiError::from)
}

fn parse_agent_run_status(value: &str) -> Result<AgentRunStatus, ApiError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ok" => Ok(AgentRunStatus::Ok),
        "fallback" => Ok(AgentRunStatus::Fallback),
        "parse_error" => Ok(AgentRunStatus::ParseError),
        _ => Err(ApiError::bad_request(format!(
            "invalid agent run status `{value}`"
        ))),
    }
}

fn require_non_empty<'a>(value: &'a str, field: &'static str) -> Result<&'a str, ApiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(ApiError::bad_request(format!("{field} cannot be empty")))
    } else {
        Ok(trimmed)
    }
}

fn draft_sse_stream(
    operation: &'static str,
    draft: ChapterDraft,
) -> impl Stream<Item = Result<Event, Infallible>> {
    let mut events = Vec::new();
    events.push(sse_json(
        "started",
        json!({
            "operation": operation,
            "chapter_index": draft.chapter_index,
            "version": draft.version
        }),
    ));

    for (index, chunk) in chunk_text(&draft.content, 48).into_iter().enumerate() {
        events.push(sse_json(
            "chapter_chunk",
            json!({
                "operation": operation,
                "chapter_index": draft.chapter_index,
                "chunk_index": index,
                "text": chunk
            }),
        ));
    }

    events.push(sse_json(
        "completed",
        json!({
            "operation": operation,
            "draft": draft
        }),
    ));
    stream::iter(events)
}

fn sse_json(event: &'static str, data: Value) -> Result<Event, Infallible> {
    Ok(Event::default().event(event).data(data.to_string()))
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

async fn retry_create_novel_job(
    state: &ApiState,
    source: &JobRecord,
) -> Result<JobRecord, ApiError> {
    let idea = payload_required_str(&source.payload, "idea")?.to_string();
    let platform_name = payload_required_str(&source.payload, "platform")?;
    let platform = parse_platform(Some(platform_name))?;
    let chapters = payload_u32_or(&source.payload, "chapters", 30)?;
    let outline_batch_size = payload_u32_or(&source.payload, "outline_batch_size", 5)?;
    let job = state
        .storage
        .jobs()
        .create_with_source(
            "create_novel",
            None,
            None,
            &source.payload,
            Some(&source.id),
        )
        .await?;

    spawn_create_novel_job(
        state.storage.clone(),
        state.model.clone(),
        job.id.clone(),
        idea,
        platform,
        chapters,
        outline_batch_size,
    );

    Ok(job)
}

async fn retry_chapter_job(
    state: &ApiState,
    source: &JobRecord,
    kind: &'static str,
) -> Result<JobRecord, ApiError> {
    let novel_id = source
        .novel_id
        .clone()
        .or_else(|| {
            source
                .payload
                .get("novel_id")
                .and_then(Value::as_str)
                .map(NovelId::from)
        })
        .ok_or_else(|| ApiError::bad_request("job payload is missing novel_id"))?;
    let chapter_index = source
        .chapter_index
        .map(Ok)
        .unwrap_or_else(|| payload_u32(&source.payload, "chapter_index"))?;

    match kind {
        "write_chapter" => ensure_novel_exists(&state.storage, &novel_id).await?,
        "review_chapter" | "rewrite_chapter" => {
            find_chapter(&state.storage, &novel_id, chapter_index).await?;
        }
        _ => {
            return Err(ApiError::bad_request(format!(
                "job kind `{kind}` cannot be retried"
            )))
        }
    }

    let job = state
        .storage
        .jobs()
        .create_with_source(
            kind,
            Some(&novel_id),
            Some(chapter_index),
            &source.payload,
            Some(&source.id),
        )
        .await?;
    match kind {
        "write_chapter" => spawn_write_chapter_job(
            state.storage.clone(),
            state.model.clone(),
            job.id.clone(),
            novel_id,
            chapter_index,
        ),
        "review_chapter" => spawn_review_chapter_job(
            state.storage.clone(),
            state.model.clone(),
            job.id.clone(),
            novel_id,
            chapter_index,
        ),
        "rewrite_chapter" => spawn_rewrite_chapter_job(
            state.storage.clone(),
            state.model.clone(),
            job.id.clone(),
            novel_id,
            chapter_index,
        ),
        _ => unreachable!("kind was checked before spawning retry job"),
    }

    Ok(job)
}

async fn retry_write_chapters_job(
    state: &ApiState,
    source: &JobRecord,
) -> Result<JobRecord, ApiError> {
    let novel_id = source
        .novel_id
        .clone()
        .or_else(|| {
            source
                .payload
                .get("novel_id")
                .and_then(Value::as_str)
                .map(NovelId::from)
        })
        .ok_or_else(|| ApiError::bad_request("job payload is missing novel_id"))?;
    let chapter_start = payload_u32(&source.payload, "chapter_start")?;
    let chapter_end = payload_u32(&source.payload, "chapter_end")?;
    let chapter_indexes = chapter_range(chapter_start, chapter_end)?;
    ensure_novel_exists(&state.storage, &novel_id).await?;

    let job = state
        .storage
        .jobs()
        .create_with_source_and_progress(
            "write_chapters",
            Some(&novel_id),
            None,
            &source.payload,
            Some(&source.id),
            0,
            chapter_indexes.len() as u32,
        )
        .await?;
    spawn_write_chapters_job(
        state.storage.clone(),
        state.model.clone(),
        job.id.clone(),
        novel_id,
        chapter_start,
        chapter_end,
    );

    Ok(job)
}

fn spawn_create_novel_job(
    storage: SqliteStorage,
    model: ModelHandle,
    job_id: String,
    idea: String,
    platform: TargetPlatform,
    chapters: u32,
    outline_batch_size: u32,
) {
    tokio::spawn(async move {
        if !mark_job_running(&storage, &job_id).await {
            return;
        }
        let workflow = NovelCreationWorkflow::new(&storage, model);
        match workflow
            .create_from_idea_with_outline_batch_size(&idea, platform, chapters, outline_batch_size)
            .await
        {
            Ok(result) => {
                complete_job(
                    &storage,
                    &job_id,
                    json!({
                        "novel": result.novel,
                        "bible": result.bible,
                        "characters": result.characters,
                        "outlines": result.outlines,
                        "used_fallback": result.used_fallback
                    }),
                )
                .await
            }
            Err(error) => fail_job(&storage, &job_id, error.to_string()).await,
        }
    });
}

fn spawn_write_chapters_job(
    storage: SqliteStorage,
    model: ModelHandle,
    job_id: String,
    novel_id: NovelId,
    chapter_start: u32,
    chapter_end: u32,
) {
    tokio::spawn(async move {
        if !mark_job_running(&storage, &job_id).await {
            return;
        }
        let workflow = ChapterGenerationWorkflow::new(&storage, model);
        let mut drafts = Vec::new();
        for chapter_index in chapter_start..=chapter_end {
            if !job_accepts_more_work(&storage, &job_id).await {
                return;
            }
            match workflow.write_chapter(&novel_id, chapter_index).await {
                Ok(draft) => {
                    drafts.push(draft);
                    set_job_progress(&storage, &job_id, drafts.len() as u32).await;
                }
                Err(error) => {
                    fail_job(&storage, &job_id, error.to_string()).await;
                    return;
                }
            }
        }

        complete_job(
            &storage,
            &job_id,
            json!({
                "chapter_start": chapter_start,
                "chapter_end": chapter_end,
                "drafts": drafts
            }),
        )
        .await;
    });
}

fn spawn_write_chapter_job(
    storage: SqliteStorage,
    model: ModelHandle,
    job_id: String,
    novel_id: NovelId,
    chapter_index: u32,
) {
    tokio::spawn(async move {
        if !mark_job_running(&storage, &job_id).await {
            return;
        }
        let workflow = ChapterGenerationWorkflow::new(&storage, model);
        match workflow.write_chapter(&novel_id, chapter_index).await {
            Ok(draft) => complete_job(&storage, &job_id, json!({ "draft": draft })).await,
            Err(error) => fail_job(&storage, &job_id, error.to_string()).await,
        }
    });
}

fn spawn_review_chapter_job(
    storage: SqliteStorage,
    model: ModelHandle,
    job_id: String,
    novel_id: NovelId,
    chapter_index: u32,
) {
    tokio::spawn(async move {
        if !mark_job_running(&storage, &job_id).await {
            return;
        }
        let workflow = ChapterGenerationWorkflow::new(&storage, model);
        match workflow.review_chapter(&novel_id, chapter_index).await {
            Ok(report) => complete_job(&storage, &job_id, json!({ "report": report })).await,
            Err(error) => fail_job(&storage, &job_id, error.to_string()).await,
        }
    });
}

fn spawn_rewrite_chapter_job(
    storage: SqliteStorage,
    model: ModelHandle,
    job_id: String,
    novel_id: NovelId,
    chapter_index: u32,
) {
    tokio::spawn(async move {
        if !mark_job_running(&storage, &job_id).await {
            return;
        }
        let workflow = ChapterGenerationWorkflow::new(&storage, model);
        match workflow.rewrite_chapter(&novel_id, chapter_index).await {
            Ok(draft) => complete_job(&storage, &job_id, json!({ "draft": draft })).await,
            Err(error) => fail_job(&storage, &job_id, error.to_string()).await,
        }
    });
}

fn payload_required_str<'a>(payload: &'a Value, key: &'static str) -> Result<&'a str, ApiError> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ApiError::bad_request(format!("job payload is missing {key}")))
}

fn payload_u32(payload: &Value, key: &'static str) -> Result<u32, ApiError> {
    let value = payload
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| ApiError::bad_request(format!("job payload is missing {key}")))?;
    u32::try_from(value)
        .map_err(|_| ApiError::bad_request(format!("job payload {key} is too large")))
}

fn payload_u32_or(payload: &Value, key: &'static str, default: u32) -> Result<u32, ApiError> {
    if payload.get(key).is_some() {
        payload_u32(payload, key)
    } else {
        Ok(default)
    }
}

const MAX_BATCH_CHAPTERS: u32 = 50;

fn chapter_range(chapter_start: u32, chapter_end: u32) -> Result<Vec<u32>, ApiError> {
    if chapter_start == 0 {
        return Err(ApiError::bad_request("chapter_start must be at least 1"));
    }
    if chapter_end < chapter_start {
        return Err(ApiError::bad_request(
            "chapter_end must be greater than or equal to chapter_start",
        ));
    }
    let count = chapter_end - chapter_start + 1;
    if count > MAX_BATCH_CHAPTERS {
        return Err(ApiError::bad_request(format!(
            "batch write jobs can include at most {MAX_BATCH_CHAPTERS} chapters"
        )));
    }
    Ok((chapter_start..=chapter_end).collect())
}

async fn job_accepts_more_work(storage: &SqliteStorage, job_id: &str) -> bool {
    match storage.jobs().find(job_id).await {
        Ok(Some(job)) if job.status == JobStatus::Running => true,
        Ok(Some(job)) => {
            tracing::info!(
                job_id = %job_id,
                status = job.status.as_str(),
                "API batch job stopped before next chapter"
            );
            false
        }
        Ok(None) => {
            tracing::warn!(job_id = %job_id, "API batch job disappeared before next chapter");
            false
        }
        Err(error) => {
            tracing::error!(job_id = %job_id, error = %error, "failed to read API batch job status");
            false
        }
    }
}

async fn set_job_progress(storage: &SqliteStorage, job_id: &str, progress_current: u32) {
    match storage.jobs().set_progress(job_id, progress_current).await {
        Ok(true) => {}
        Ok(false) => {
            tracing::info!(job_id = %job_id, "API job no longer accepts progress update");
        }
        Err(error) => {
            tracing::error!(job_id = %job_id, error = %error, "failed to update API job progress");
        }
    }
}

async fn mark_job_running(storage: &SqliteStorage, job_id: &str) -> bool {
    match storage.jobs().set_running(job_id).await {
        Ok(true) => true,
        Ok(false) => {
            tracing::info!(job_id = %job_id, "API job was not queued; skipping background work");
            false
        }
        Err(error) => {
            tracing::error!(job_id = %job_id, error = %error, "failed to mark API job running");
            false
        }
    }
}

async fn complete_job(storage: &SqliteStorage, job_id: &str, result: Value) {
    match storage.jobs().complete(job_id, &result).await {
        Ok(true) => {}
        Ok(false) => {
            tracing::info!(job_id = %job_id, "API job no longer accepts completion update");
        }
        Err(error) => {
            tracing::error!(job_id = %job_id, error = %error, "failed to complete API job");
        }
    }
}

async fn fail_job(storage: &SqliteStorage, job_id: &str, message: String) {
    match storage.jobs().fail(job_id, &message).await {
        Ok(true) => {}
        Ok(false) => {
            tracing::info!(job_id = %job_id, "API job no longer accepts failure update");
        }
        Err(error) => {
            tracing::error!(job_id = %job_id, error = %error, "failed to fail API job");
        }
    }
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request",
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: message.into(),
        }
    }
}

impl From<StorageError> for ApiError {
    fn from(error: StorageError) -> Self {
        match error {
            StorageError::InvalidEnum { .. } => Self::bad_request(error.to_string()),
            _ => Self::internal(error.to_string()),
        }
    }
}

impl From<WorkflowError> for ApiError {
    fn from(error: WorkflowError) -> Self {
        match error {
            WorkflowError::NovelNotFound(_) | WorkflowError::ChapterNotFound { .. } => {
                Self::not_found(error.to_string())
            }
            WorkflowError::InvalidInput(_) => Self::bad_request(error.to_string()),
            _ => Self::internal(error.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({
                "error": {
                    "code": self.code,
                    "message": self.message
                }
            })),
        )
            .into_response()
    }
}

#[derive(Debug, Deserialize)]
struct LimitQuery {
    limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct JobsQuery {
    limit: Option<u32>,
    status: Option<String>,
    kind: Option<String>,
    novel_id: Option<String>,
    source_job_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AgentRunsQuery {
    limit: Option<u32>,
    novel_id: Option<String>,
    role: Option<String>,
    task: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateNovelRequest {
    idea: String,
    platform: Option<String>,
    chapters: Option<u32>,
    outline_batch_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OutlineRequest {
    chapters: Option<u32>,
    batch_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ManualEditChapterRequest {
    title: Option<String>,
    content: String,
    summary: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BatchWriteChaptersRequest {
    chapter_start: u32,
    chapter_end: u32,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(Debug, Serialize)]
struct JobsResponse {
    jobs: Vec<JobRecord>,
}

#[derive(Debug, Serialize)]
struct JobResponse {
    job: JobRecord,
}

#[derive(Debug, Serialize)]
struct NovelsResponse {
    novels: Vec<Novel>,
}

#[derive(Debug, Serialize)]
struct CreateNovelResponse {
    novel: Novel,
    bible: NovelBible,
    characters: Vec<CharacterCard>,
    outlines: Vec<ChapterOutline>,
    used_fallback: bool,
}

#[derive(Debug, Serialize)]
struct NovelDetailResponse {
    novel: Novel,
    bible: Option<NovelBible>,
    characters: Vec<CharacterCard>,
    chapters: Vec<Chapter>,
    world_setting: Option<Value>,
    facts: Vec<Fact>,
}

#[derive(Debug, Serialize)]
struct BibleResponse {
    bible: Option<NovelBible>,
}

#[derive(Debug, Serialize)]
struct CharactersResponse {
    characters: Vec<CharacterCard>,
}

#[derive(Debug, Serialize)]
struct WorldSettingResponse {
    world_setting: Option<Value>,
}

#[derive(Debug, Serialize)]
struct FactsResponse {
    facts: Vec<Fact>,
}

#[derive(Debug, Serialize)]
struct OutlineResponse {
    outlines: Vec<ChapterOutline>,
}

#[derive(Debug, Serialize)]
struct ChaptersResponse {
    chapters: Vec<Chapter>,
}

#[derive(Debug, Serialize)]
struct ChapterResponse {
    chapter: Chapter,
}

#[derive(Debug, Serialize)]
struct ChapterDraftResponse {
    draft: ChapterDraft,
}

#[derive(Debug, Serialize)]
struct ReviewResponse {
    report: ReviewReport,
}

#[derive(Debug, Serialize)]
struct LatestReviewResponse {
    chapter: Chapter,
    report: Option<ReviewReport>,
}

#[derive(Debug, Serialize)]
struct LatestContinuityResponse {
    chapter: Chapter,
    report: Option<Value>,
}

#[derive(Debug, Serialize)]
struct ChapterVersionsResponse {
    novel_id: String,
    chapter_id: String,
    chapter_index: u32,
    versions: Vec<u32>,
}

#[derive(Debug, Serialize)]
struct ChapterVersionResponse {
    novel_id: String,
    chapter_id: String,
    chapter_index: u32,
    version: u32,
    content: String,
}

#[derive(Debug, Serialize)]
struct ExportMarkdownResponse {
    novel_id: String,
    format: String,
    filename: String,
    markdown: String,
}

#[derive(Debug, Serialize)]
struct AgentRunsResponse {
    runs: Vec<AgentRunResponse>,
    summary: AgentRunStatusSummary,
}

#[derive(Debug, Serialize)]
struct AgentRunResponse {
    id: String,
    novel_id: Option<String>,
    role: String,
    task: String,
    status: String,
    attempt: Option<u64>,
    duration_ms: Option<u64>,
    total_tokens: Option<u64>,
    structured: Value,
    raw_text: String,
    raw_notes: String,
    parse_error: Option<String>,
    created_at: String,
}

impl AgentRunResponse {
    fn from_record(run: &AgentRunRecord) -> Self {
        Self {
            id: run.id.clone(),
            novel_id: run.novel_id.as_ref().map(ToString::to_string),
            role: run.role.clone(),
            task: run.task.clone(),
            status: run.status().as_str().to_string(),
            attempt: run.attempt(),
            duration_ms: run.duration_ms(),
            total_tokens: run.total_tokens(),
            structured: run.structured.clone(),
            raw_text: run.raw_text.clone(),
            raw_notes: run.raw_notes.clone(),
            parse_error: run.parse_error.clone(),
            created_at: run.created_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;
    use crate::model::SmokeModelClient;

    #[tokio::test]
    async fn api_can_create_and_read_smoke_project() {
        let db_path = std::env::temp_dir().join(format!("novel-agent-api-{}.db", Uuid::new_v4()));
        let database_url = format!(
            "sqlite://{}",
            db_path.display().to_string().replace('\\', "/")
        );
        let storage = SqliteStorage::connect(&database_url).await.unwrap();
        storage.migrate().await.unwrap();
        let app = router(
            storage.clone(),
            Arc::new(SmokeModelClient::new("smoke".to_string())),
        );

        let cors_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/api/novels")
                    .header("origin", "http://localhost:5173")
                    .header("access-control-request-method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cors_response.status(), StatusCode::OK);
        assert!(cors_response
            .headers()
            .get("access-control-allow-origin")
            .is_some());

        let create_response = app
            .clone()
            .oneshot(json_request(
                "POST",
                "/api/novels",
                json!({
                    "idea": "都市重生商业文，主角回到十年前从外卖站逆袭",
                    "platform": "fanqie",
                    "chapters": 3,
                    "outline_batch_size": 2
                }),
            ))
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);
        let create_json = response_json(create_response).await;
        let novel_id = create_json["novel"]["id"].as_str().unwrap();
        assert_eq!(create_json["outlines"].as_array().unwrap().len(), 3);

        let list_response = app
            .clone()
            .oneshot(empty_request("GET", "/api/novels"))
            .await
            .unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);
        let list_json = response_json(list_response).await;
        assert_eq!(list_json["novels"].as_array().unwrap().len(), 1);

        let bible_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!("/api/novels/{novel_id}/bible"),
            ))
            .await
            .unwrap();
        assert_eq!(bible_response.status(), StatusCode::OK);
        let bible_json = response_json(bible_response).await;
        assert_eq!(bible_json["bible"]["novel_id"].as_str(), Some(novel_id));

        let characters_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!("/api/novels/{novel_id}/characters"),
            ))
            .await
            .unwrap();
        assert_eq!(characters_response.status(), StatusCode::OK);
        let characters_json = response_json(characters_response).await;
        assert!(!characters_json["characters"].as_array().unwrap().is_empty());
        assert!(characters_json["characters"]
            .as_array()
            .unwrap()
            .iter()
            .all(|character| character["novel_id"].as_str() == Some(novel_id)));

        let world_setting_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!("/api/novels/{novel_id}/world-settings"),
            ))
            .await
            .unwrap();
        assert_eq!(world_setting_response.status(), StatusCode::OK);
        let world_setting_json = response_json(world_setting_response).await;
        assert!(world_setting_json["world_setting"].is_object());

        let facts_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!("/api/novels/{novel_id}/facts?limit=10"),
            ))
            .await
            .unwrap();
        assert_eq!(facts_response.status(), StatusCode::OK);
        let facts_json = response_json(facts_response).await;
        let facts = facts_json["facts"].as_array().unwrap();
        assert!(!facts.is_empty());
        assert!(facts
            .iter()
            .all(|fact| fact["novel_id"].as_str() == Some(novel_id)));

        let write_response = app
            .clone()
            .oneshot(empty_request(
                "POST",
                &format!("/api/novels/{novel_id}/chapters/1/write"),
            ))
            .await
            .unwrap();
        assert_eq!(write_response.status(), StatusCode::OK);

        let continuity_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!("/api/novels/{novel_id}/chapters/1/continuity"),
            ))
            .await
            .unwrap();
        assert_eq!(continuity_response.status(), StatusCode::OK);
        let continuity_json = response_json(continuity_response).await;
        assert_eq!(
            continuity_json["chapter"]["chapter_index"].as_u64(),
            Some(1)
        );
        assert!(continuity_json["report"]["issues"].is_array());
        assert!(continuity_json["report"]["new_facts"].is_array());

        let review_response = app
            .clone()
            .oneshot(empty_request(
                "POST",
                &format!("/api/novels/{novel_id}/chapters/1/review"),
            ))
            .await
            .unwrap();
        assert_eq!(review_response.status(), StatusCode::OK);

        let manual_content = "人工编辑新增：主角在雨夜重新校准目标，并把旧债线索压进下一章。";
        let manual_edit_response = app
            .clone()
            .oneshot(json_request(
                "PUT",
                &format!("/api/novels/{novel_id}/chapters/1/edit"),
                json!({
                    "title": "第一章 人工修订",
                    "content": manual_content,
                    "summary": "人工保存后补强目标和伏笔"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(manual_edit_response.status(), StatusCode::OK);
        let manual_edit_json = response_json(manual_edit_response).await;
        assert_eq!(
            manual_edit_json["draft"]["title"].as_str(),
            Some("第一章 人工修订")
        );
        assert_eq!(
            manual_edit_json["draft"]["content"].as_str(),
            Some(manual_content)
        );
        assert_eq!(manual_edit_json["draft"]["version"].as_u64(), Some(2));

        let edited_chapter_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!("/api/novels/{novel_id}/chapters/1"),
            ))
            .await
            .unwrap();
        assert_eq!(edited_chapter_response.status(), StatusCode::OK);
        let edited_chapter_json = response_json(edited_chapter_response).await;
        assert_eq!(edited_chapter_json["chapter"]["version"].as_u64(), Some(2));
        assert_eq!(
            edited_chapter_json["chapter"]["content"].as_str(),
            Some(manual_content)
        );

        let versions_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!("/api/novels/{novel_id}/chapters/1/versions"),
            ))
            .await
            .unwrap();
        assert_eq!(versions_response.status(), StatusCode::OK);
        let versions_json = response_json(versions_response).await;
        assert!(versions_json["versions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|version| version.as_u64() == Some(2)));

        let empty_manual_edit_response = app
            .clone()
            .oneshot(json_request(
                "PUT",
                &format!("/api/novels/{novel_id}/chapters/1/edit"),
                json!({
                    "content": "   "
                }),
            ))
            .await
            .unwrap();
        assert_eq!(empty_manual_edit_response.status(), StatusCode::BAD_REQUEST);

        let stream_response = app
            .clone()
            .oneshot(empty_request(
                "POST",
                &format!("/api/novels/{novel_id}/chapters/2/write/stream"),
            ))
            .await
            .unwrap();
        assert_eq!(stream_response.status(), StatusCode::OK);
        assert!(stream_response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("text/event-stream"));
        let stream_text = response_text(stream_response).await;
        assert!(stream_text.contains("event: started"));
        assert!(stream_text.contains("\"chapter_index\":2"));
        assert!(stream_text.contains("event: chapter_chunk"));
        assert!(stream_text.contains("event: completed"));

        let job_response = app
            .clone()
            .oneshot(empty_request(
                "POST",
                &format!("/api/novels/{novel_id}/chapters/3/write/jobs"),
            ))
            .await
            .unwrap();
        assert_eq!(job_response.status(), StatusCode::ACCEPTED);
        let job_json = response_json(job_response).await;
        let job_id = job_json["job"]["id"].as_str().unwrap();
        assert_eq!(job_json["job"]["kind"].as_str(), Some("write_chapter"));
        assert_eq!(job_json["job"]["status"].as_str(), Some("queued"));
        assert_eq!(
            job_json["job"]["payload"]["chapter_index"].as_u64(),
            Some(3)
        );
        assert_eq!(job_json["job"]["progress_current"].as_u64(), Some(0));
        assert_eq!(job_json["job"]["progress_total"].as_u64(), Some(1));

        let mut completed_job_json = Value::Null;
        for _ in 0..20 {
            let poll_response = app
                .clone()
                .oneshot(empty_request("GET", &format!("/api/jobs/{job_id}")))
                .await
                .unwrap();
            assert_eq!(poll_response.status(), StatusCode::OK);
            let poll_json = response_json(poll_response).await;
            match poll_json["job"]["status"].as_str() {
                Some("succeeded") | Some("failed") => {
                    completed_job_json = poll_json;
                    break;
                }
                _ => tokio::time::sleep(Duration::from_millis(50)).await,
            }
        }
        assert_eq!(
            completed_job_json["job"]["status"].as_str(),
            Some("succeeded")
        );
        assert_eq!(
            completed_job_json["job"]["result"]["draft"]["chapter_index"].as_u64(),
            Some(3)
        );
        assert_eq!(
            completed_job_json["job"]["progress_current"].as_u64(),
            Some(1)
        );
        assert_eq!(
            completed_job_json["job"]["progress_total"].as_u64(),
            Some(1)
        );

        let jobs_response = app
            .clone()
            .oneshot(empty_request("GET", "/api/jobs?limit=10"))
            .await
            .unwrap();
        assert_eq!(jobs_response.status(), StatusCode::OK);
        let jobs_json = response_json(jobs_response).await;
        assert!(jobs_json["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|job| job["id"].as_str() == Some(job_id)));

        let novel_jobs_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!("/api/jobs?limit=10&novel_id={novel_id}"),
            ))
            .await
            .unwrap();
        assert_eq!(novel_jobs_response.status(), StatusCode::OK);
        let novel_jobs_json = response_json(novel_jobs_response).await;
        assert!(novel_jobs_json["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|job| job["id"].as_str() == Some(job_id)));
        assert!(novel_jobs_json["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|job| job["novel_id"].as_str() == Some(novel_id)));

        let restarted_app = router(
            storage.clone(),
            Arc::new(SmokeModelClient::new("smoke".to_string())),
        );
        let persisted_job_response = restarted_app
            .oneshot(empty_request("GET", &format!("/api/jobs/{job_id}")))
            .await
            .unwrap();
        assert_eq!(persisted_job_response.status(), StatusCode::OK);
        let persisted_job_json = response_json(persisted_job_response).await;
        assert_eq!(
            persisted_job_json["job"]["status"].as_str(),
            Some("succeeded")
        );
        assert_eq!(
            persisted_job_json["job"]["result"]["draft"]["chapter_index"].as_u64(),
            Some(3)
        );
        assert_eq!(
            persisted_job_json["job"]["payload"]["chapter_index"].as_u64(),
            Some(3)
        );
        assert_eq!(
            persisted_job_json["job"]["progress_current"].as_u64(),
            Some(1)
        );
        assert_eq!(
            persisted_job_json["job"]["progress_total"].as_u64(),
            Some(1)
        );

        let batch_job_response = app
            .clone()
            .oneshot(json_request(
                "POST",
                &format!("/api/novels/{novel_id}/chapters/write/jobs"),
                json!({
                    "chapter_start": 4,
                    "chapter_end": 5
                }),
            ))
            .await
            .unwrap();
        assert_eq!(batch_job_response.status(), StatusCode::ACCEPTED);
        let batch_job_json = response_json(batch_job_response).await;
        let batch_job_id = batch_job_json["job"]["id"].as_str().unwrap();
        assert_eq!(
            batch_job_json["job"]["kind"].as_str(),
            Some("write_chapters")
        );
        assert_eq!(batch_job_json["job"]["chapter_index"], Value::Null);
        assert_eq!(
            batch_job_json["job"]["payload"]["chapter_start"].as_u64(),
            Some(4)
        );
        assert_eq!(
            batch_job_json["job"]["payload"]["chapter_end"].as_u64(),
            Some(5)
        );
        assert_eq!(
            batch_job_json["job"]["payload"]["chapter_indexes"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(batch_job_json["job"]["progress_current"].as_u64(), Some(0));
        assert_eq!(batch_job_json["job"]["progress_total"].as_u64(), Some(2));

        let mut completed_batch_job_json = Value::Null;
        for _ in 0..30 {
            let poll_response = app
                .clone()
                .oneshot(empty_request("GET", &format!("/api/jobs/{batch_job_id}")))
                .await
                .unwrap();
            assert_eq!(poll_response.status(), StatusCode::OK);
            let poll_json = response_json(poll_response).await;
            match poll_json["job"]["status"].as_str() {
                Some("succeeded") | Some("failed") => {
                    completed_batch_job_json = poll_json;
                    break;
                }
                _ => tokio::time::sleep(Duration::from_millis(50)).await,
            }
        }
        assert_eq!(
            completed_batch_job_json["job"]["status"].as_str(),
            Some("succeeded")
        );
        let batch_drafts = completed_batch_job_json["job"]["result"]["drafts"]
            .as_array()
            .unwrap();
        assert_eq!(batch_drafts.len(), 2);
        assert_eq!(batch_drafts[0]["chapter_index"].as_u64(), Some(4));
        assert_eq!(batch_drafts[1]["chapter_index"].as_u64(), Some(5));
        assert_eq!(
            completed_batch_job_json["job"]["progress_current"].as_u64(),
            Some(2)
        );
        assert_eq!(
            completed_batch_job_json["job"]["progress_total"].as_u64(),
            Some(2)
        );

        let filtered_jobs_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                "/api/jobs?limit=10&status=succeeded&kind=write_chapters",
            ))
            .await
            .unwrap();
        assert_eq!(filtered_jobs_response.status(), StatusCode::OK);
        let filtered_jobs_json = response_json(filtered_jobs_response).await;
        assert!(filtered_jobs_json["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|job| job["id"].as_str() == Some(batch_job_id)));
        assert!(filtered_jobs_json["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|job| {
                job["status"].as_str() == Some("succeeded")
                    && job["kind"].as_str() == Some("write_chapters")
            }));

        let invalid_status_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                "/api/jobs?status=definitely_not_a_status",
            ))
            .await
            .unwrap();
        assert_eq!(invalid_status_response.status(), StatusCode::BAD_REQUEST);

        let completed_retry_response = app
            .clone()
            .oneshot(empty_request("POST", &format!("/api/jobs/{job_id}/retry")))
            .await
            .unwrap();
        assert_eq!(completed_retry_response.status(), StatusCode::BAD_REQUEST);

        let cancel_payload = json!({
            "novel_id": novel_id,
            "chapter_index": 4
        });
        let queued_cancel_job = storage
            .jobs()
            .create(
                "write_chapter",
                Some(&NovelId::from(novel_id)),
                Some(4),
                &cancel_payload,
            )
            .await
            .unwrap();
        let cancel_response = app
            .clone()
            .oneshot(empty_request(
                "POST",
                &format!("/api/jobs/{}/cancel", queued_cancel_job.id),
            ))
            .await
            .unwrap();
        assert_eq!(cancel_response.status(), StatusCode::OK);
        let cancel_json = response_json(cancel_response).await;
        assert_eq!(cancel_json["job"]["status"].as_str(), Some("cancelled"));
        assert_eq!(
            cancel_json["job"]["error"].as_str(),
            Some("job cancelled by user")
        );
        assert!(cancel_json["job"]["result"].is_null());
        assert_eq!(cancel_json["job"]["progress_current"].as_u64(), Some(0));
        assert_eq!(cancel_json["job"]["progress_total"].as_u64(), Some(1));

        let cancel_again_response = app
            .clone()
            .oneshot(empty_request(
                "POST",
                &format!("/api/jobs/{}/cancel", queued_cancel_job.id),
            ))
            .await
            .unwrap();
        assert_eq!(cancel_again_response.status(), StatusCode::BAD_REQUEST);

        let completed_cancel_response = app
            .clone()
            .oneshot(empty_request("POST", &format!("/api/jobs/{job_id}/cancel")))
            .await
            .unwrap();
        assert_eq!(completed_cancel_response.status(), StatusCode::BAD_REQUEST);

        let retry_payload = json!({
            "novel_id": novel_id,
            "chapter_index": 3
        });
        let failed_source_job = storage
            .jobs()
            .create(
                "write_chapter",
                Some(&NovelId::from(novel_id)),
                Some(3),
                &retry_payload,
            )
            .await
            .unwrap();
        storage
            .jobs()
            .fail(&failed_source_job.id, "transient test failure")
            .await
            .unwrap();

        let retry_response = app
            .clone()
            .oneshot(empty_request(
                "POST",
                &format!("/api/jobs/{}/retry", failed_source_job.id),
            ))
            .await
            .unwrap();
        assert_eq!(retry_response.status(), StatusCode::ACCEPTED);
        let retry_json = response_json(retry_response).await;
        let retried_job_id = retry_json["job"]["id"].as_str().unwrap();
        assert_ne!(retried_job_id, failed_source_job.id);
        assert_eq!(retry_json["job"]["kind"].as_str(), Some("write_chapter"));
        assert_eq!(
            retry_json["job"]["source_job_id"].as_str(),
            Some(failed_source_job.id.as_str())
        );
        assert_eq!(
            retry_json["job"]["payload"]["chapter_index"].as_u64(),
            Some(3)
        );
        assert_eq!(retry_json["job"]["progress_current"].as_u64(), Some(0));
        assert_eq!(retry_json["job"]["progress_total"].as_u64(), Some(1));

        let source_jobs_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!(
                    "/api/jobs?limit=10&novel_id={novel_id}&source_job_id={}",
                    failed_source_job.id
                ),
            ))
            .await
            .unwrap();
        assert_eq!(source_jobs_response.status(), StatusCode::OK);
        let source_jobs_json = response_json(source_jobs_response).await;
        assert!(source_jobs_json["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|job| job["id"].as_str() == Some(retried_job_id)));
        assert!(source_jobs_json["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|job| {
                job["source_job_id"].as_str() == Some(failed_source_job.id.as_str())
                    && job["novel_id"].as_str() == Some(novel_id)
            }));

        let mut retried_job_json = Value::Null;
        for _ in 0..20 {
            let poll_response = app
                .clone()
                .oneshot(empty_request("GET", &format!("/api/jobs/{retried_job_id}")))
                .await
                .unwrap();
            assert_eq!(poll_response.status(), StatusCode::OK);
            let poll_json = response_json(poll_response).await;
            match poll_json["job"]["status"].as_str() {
                Some("succeeded") | Some("failed") => {
                    retried_job_json = poll_json;
                    break;
                }
                _ => tokio::time::sleep(Duration::from_millis(50)).await,
            }
        }
        assert_eq!(
            retried_job_json["job"]["status"].as_str(),
            Some("succeeded")
        );
        assert_eq!(
            retried_job_json["job"]["result"]["draft"]["chapter_index"].as_u64(),
            Some(3)
        );
        assert_eq!(
            retried_job_json["job"]["progress_current"].as_u64(),
            Some(1)
        );
        assert_eq!(retried_job_json["job"]["progress_total"].as_u64(), Some(1));

        let retry_batch_payload = json!({
            "novel_id": novel_id,
            "chapter_start": 6,
            "chapter_end": 7,
            "chapter_indexes": [6, 7]
        });
        let failed_batch_source_job = storage
            .jobs()
            .create_with_source_and_progress(
                "write_chapters",
                Some(&NovelId::from(novel_id)),
                None,
                &retry_batch_payload,
                None,
                0,
                2,
            )
            .await
            .unwrap();
        storage
            .jobs()
            .fail(&failed_batch_source_job.id, "batch transient test failure")
            .await
            .unwrap();

        let batch_retry_response = app
            .clone()
            .oneshot(empty_request(
                "POST",
                &format!("/api/jobs/{}/retry", failed_batch_source_job.id),
            ))
            .await
            .unwrap();
        assert_eq!(batch_retry_response.status(), StatusCode::ACCEPTED);
        let batch_retry_json = response_json(batch_retry_response).await;
        let retried_batch_job_id = batch_retry_json["job"]["id"].as_str().unwrap();
        assert_eq!(
            batch_retry_json["job"]["kind"].as_str(),
            Some("write_chapters")
        );
        assert_eq!(
            batch_retry_json["job"]["source_job_id"].as_str(),
            Some(failed_batch_source_job.id.as_str())
        );
        assert_eq!(
            batch_retry_json["job"]["payload"]["chapter_end"].as_u64(),
            Some(7)
        );
        assert_eq!(
            batch_retry_json["job"]["progress_current"].as_u64(),
            Some(0)
        );
        assert_eq!(batch_retry_json["job"]["progress_total"].as_u64(), Some(2));

        let mut retried_batch_job_json = Value::Null;
        for _ in 0..30 {
            let poll_response = app
                .clone()
                .oneshot(empty_request(
                    "GET",
                    &format!("/api/jobs/{retried_batch_job_id}"),
                ))
                .await
                .unwrap();
            assert_eq!(poll_response.status(), StatusCode::OK);
            let poll_json = response_json(poll_response).await;
            match poll_json["job"]["status"].as_str() {
                Some("succeeded") | Some("failed") => {
                    retried_batch_job_json = poll_json;
                    break;
                }
                _ => tokio::time::sleep(Duration::from_millis(50)).await,
            }
        }
        assert_eq!(
            retried_batch_job_json["job"]["status"].as_str(),
            Some("succeeded")
        );
        let retried_batch_drafts = retried_batch_job_json["job"]["result"]["drafts"]
            .as_array()
            .unwrap();
        assert_eq!(retried_batch_drafts.len(), 2);
        assert_eq!(retried_batch_drafts[0]["chapter_index"].as_u64(), Some(6));
        assert_eq!(retried_batch_drafts[1]["chapter_index"].as_u64(), Some(7));
        assert_eq!(
            retried_batch_job_json["job"]["progress_current"].as_u64(),
            Some(2)
        );
        assert_eq!(
            retried_batch_job_json["job"]["progress_total"].as_u64(),
            Some(2)
        );

        let export_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!("/api/novels/{novel_id}/export/markdown"),
            ))
            .await
            .unwrap();
        assert_eq!(export_response.status(), StatusCode::OK);
        let export_json = response_json(export_response).await;
        assert_eq!(export_json["format"].as_str(), Some("markdown"));
        let markdown = export_json["markdown"].as_str().unwrap();
        assert!(markdown.contains("# 重回外卖站"));
        assert!(markdown.contains("## 第1章"));

        let runs_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!("/api/novels/{novel_id}/runs?limit=50"),
            ))
            .await
            .unwrap();
        assert_eq!(runs_response.status(), StatusCode::OK);
        let runs_json = response_json(runs_response).await;
        assert!(runs_json["summary"]["total"].as_u64().unwrap() > 0);
        assert_eq!(runs_json["summary"]["fallback"].as_u64(), Some(0));
        assert_eq!(runs_json["summary"]["parse_error"].as_u64(), Some(0));
        assert!(runs_json["summary"]["tokenized_runs"].as_u64().unwrap() > 0);
        assert!(runs_json["summary"]["total_tokens"].as_u64().unwrap() > 0);

        let filtered_runs_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!(
                    "/api/novels/{novel_id}/runs?limit=20&role=writer&task=generate_chapter&status=ok"
                ),
            ))
            .await
            .unwrap();
        assert_eq!(filtered_runs_response.status(), StatusCode::OK);
        let filtered_runs_json = response_json(filtered_runs_response).await;
        let filtered_runs = filtered_runs_json["runs"].as_array().unwrap();
        assert!(!filtered_runs.is_empty());
        assert!(filtered_runs.iter().all(|run| {
            run["role"].as_str() == Some("writer")
                && run["task"].as_str() == Some("generate_chapter")
                && run["status"].as_str() == Some("ok")
        }));
        assert_eq!(
            filtered_runs_json["summary"]["total"].as_u64(),
            Some(filtered_runs.len() as u64)
        );

        let global_runs_response = app
            .clone()
            .oneshot(empty_request(
                "GET",
                &format!(
                    "/api/runs?limit=20&novel_id={novel_id}&role=writer&task=generate_chapter&status=ok"
                ),
            ))
            .await
            .unwrap();
        assert_eq!(global_runs_response.status(), StatusCode::OK);
        let global_runs_json = response_json(global_runs_response).await;
        let global_runs = global_runs_json["runs"].as_array().unwrap();
        assert_eq!(global_runs.len(), filtered_runs.len());
        assert!(global_runs.iter().all(|run| {
            run["novel_id"].as_str() == Some(novel_id)
                && run["role"].as_str() == Some("writer")
                && run["task"].as_str() == Some("generate_chapter")
                && run["status"].as_str() == Some("ok")
        }));

        let invalid_run_status_response = app
            .oneshot(empty_request(
                "GET",
                &format!("/api/novels/{novel_id}/runs?status=definitely_not_a_status"),
            ))
            .await
            .unwrap();
        assert_eq!(
            invalid_run_status_response.status(),
            StatusCode::BAD_REQUEST
        );

        let _ = std::fs::remove_file(db_path);
    }

    fn empty_request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    async fn response_json(response: axum::response::Response) -> Value {
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    async fn response_text(response: axum::response::Response) -> String {
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}
