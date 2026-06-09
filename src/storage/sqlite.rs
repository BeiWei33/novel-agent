use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

use crate::agents::{AgentOutput, AgentStorage, AgentTask};
use crate::domain::{
    Chapter, ChapterDraft, ChapterId, ChapterStatus, CharacterCard, Fact, FactId, FactTriple,
    Novel, NovelBible, NovelId, NovelStatus, ReviewReport, TargetPlatform,
};
use crate::error::StorageError;

#[derive(Debug, Clone)]
pub struct SqliteStorage {
    pool: SqlitePool,
}

impl AgentStorage for SqliteStorage {}

impl SqliteStorage {
    pub async fn connect(database_url: &str) -> Result<Self, StorageError> {
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), StorageError> {
        for statement in SCHEMA {
            sqlx::query(statement).execute(&self.pool).await?;
        }
        self.ensure_api_jobs_columns().await?;
        self.ensure_api_jobs_indexes().await?;
        Ok(())
    }

    async fn ensure_api_jobs_columns(&self) -> Result<(), StorageError> {
        if !self.api_jobs_has_column("payload").await? {
            sqlx::query("ALTER TABLE api_jobs ADD COLUMN payload TEXT NOT NULL DEFAULT '{}'")
                .execute(&self.pool)
                .await?;
        }
        if !self.api_jobs_has_column("source_job_id").await? {
            sqlx::query("ALTER TABLE api_jobs ADD COLUMN source_job_id TEXT")
                .execute(&self.pool)
                .await?;
        }
        if !self.api_jobs_has_column("progress_current").await? {
            sqlx::query(
                "ALTER TABLE api_jobs ADD COLUMN progress_current INTEGER NOT NULL DEFAULT 0",
            )
            .execute(&self.pool)
            .await?;
        }
        if !self.api_jobs_has_column("progress_total").await? {
            sqlx::query(
                "ALTER TABLE api_jobs ADD COLUMN progress_total INTEGER NOT NULL DEFAULT 1",
            )
            .execute(&self.pool)
            .await?;
        }
        self.backfill_api_jobs_progress().await?;
        Ok(())
    }

    async fn backfill_api_jobs_progress(&self) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE api_jobs
            SET progress_total = 1
            WHERE progress_total < 1
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE api_jobs
            SET progress_current = progress_total
            WHERE status = ?1
              AND progress_current = 0
            "#,
        )
        .bind(JobStatus::Succeeded.as_str())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn api_jobs_has_column(&self, column: &str) -> Result<bool, StorageError> {
        let rows = sqlx::query("PRAGMA table_info(api_jobs)")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .iter()
            .any(|row| row.get::<String, _>("name") == column))
    }

    async fn ensure_api_jobs_indexes(&self) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_api_jobs_source_job_id
            ON api_jobs(source_job_id)
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_api_jobs_novel_id_updated_at
            ON api_jobs(novel_id, updated_at DESC)
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_api_jobs_status_kind_updated_at
            ON api_jobs(status, kind, updated_at DESC)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub fn novels(&self) -> NovelRepository<'_> {
        NovelRepository { pool: &self.pool }
    }

    pub fn chapters(&self) -> ChapterRepository<'_> {
        ChapterRepository { pool: &self.pool }
    }

    pub fn chapter_versions(&self) -> ChapterVersionRepository<'_> {
        ChapterVersionRepository { pool: &self.pool }
    }

    pub fn characters(&self) -> CharacterRepository<'_> {
        CharacterRepository { pool: &self.pool }
    }

    pub fn review_reports(&self) -> ReviewReportRepository<'_> {
        ReviewReportRepository { pool: &self.pool }
    }

    pub fn facts(&self) -> FactRepository<'_> {
        FactRepository { pool: &self.pool }
    }

    pub fn world_settings(&self) -> WorldSettingRepository<'_> {
        WorldSettingRepository { pool: &self.pool }
    }

    pub fn continuity_reports(&self) -> ContinuityReportRepository<'_> {
        ContinuityReportRepository { pool: &self.pool }
    }

    pub fn agent_runs(&self) -> AgentRunRepository<'_> {
        AgentRunRepository { pool: &self.pool }
    }

    pub fn jobs(&self) -> JobRepository<'_> {
        JobRepository { pool: &self.pool }
    }
}

pub struct NovelRepository<'a> {
    pool: &'a SqlitePool,
}

impl NovelRepository<'_> {
    pub async fn insert(&self, novel: &Novel) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO novels (id, title, genre, target_platform, status, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                genre = excluded.genre,
                target_platform = excluded.target_platform,
                status = excluded.status,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(novel.id.as_str())
        .bind(&novel.title)
        .bind(&novel.genre)
        .bind(novel.target_platform.as_str())
        .bind(novel.status.as_str())
        .bind(novel.created_at.to_rfc3339())
        .bind(novel.updated_at.to_rfc3339())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn find(&self, id: &NovelId) -> Result<Option<Novel>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT id, title, genre, target_platform, status, created_at, updated_at
            FROM novels
            WHERE id = ?1
            "#,
        )
        .bind(id.as_str())
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_novel).transpose()
    }

    pub async fn list_recent(&self, limit: u32) -> Result<Vec<Novel>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, genre, target_platform, status, created_at, updated_at
            FROM novels
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )
        .bind(i64::from(limit.max(1)))
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_novel).collect()
    }

    pub async fn save_bible(&self, bible: &NovelBible) -> Result<(), StorageError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO novel_bibles (novel_id, data, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(novel_id) DO UPDATE SET
                data = excluded.data,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(bible.novel_id.as_str())
        .bind(to_json(bible)?)
        .bind(&now)
        .bind(&now)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_bible(&self, novel_id: &NovelId) -> Result<Option<NovelBible>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT data
            FROM novel_bibles
            WHERE novel_id = ?1
            "#,
        )
        .bind(novel_id.as_str())
        .fetch_optional(self.pool)
        .await?;

        row.map(|row| from_json(row.get("data"))).transpose()
    }
}

pub struct ChapterRepository<'a> {
    pool: &'a SqlitePool,
}

impl ChapterRepository<'_> {
    pub async fn upsert_outline(&self, chapter: &Chapter) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO chapters (
                id, novel_id, volume_index, chapter_index, title, outline, content, summary,
                status, score, word_count, version, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(novel_id, chapter_index) DO UPDATE SET
                volume_index = excluded.volume_index,
                title = excluded.title,
                outline = excluded.outline,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(chapter.id.as_str())
        .bind(chapter.novel_id.as_str())
        .bind(i64::from(chapter.volume_index))
        .bind(i64::from(chapter.chapter_index))
        .bind(&chapter.title)
        .bind(&chapter.outline)
        .bind(&chapter.content)
        .bind(&chapter.summary)
        .bind(chapter.status.as_str())
        .bind(chapter.score)
        .bind(i64::from(chapter.word_count))
        .bind(i64::from(chapter.version))
        .bind(chapter.created_at.to_rfc3339())
        .bind(chapter.updated_at.to_rfc3339())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_index(
        &self,
        novel_id: &NovelId,
        chapter_index: u32,
    ) -> Result<Option<Chapter>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT id, novel_id, volume_index, chapter_index, title, outline, content, summary,
                   status, score, word_count, version, created_at, updated_at
            FROM chapters
            WHERE novel_id = ?1 AND chapter_index = ?2
            "#,
        )
        .bind(novel_id.as_str())
        .bind(i64::from(chapter_index))
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_chapter).transpose()
    }

    pub async fn save_draft(&self, draft: &ChapterDraft) -> Result<(), StorageError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE chapters
            SET title = ?2,
                content = ?3,
                summary = ?4,
                status = ?5,
                word_count = ?6,
                version = ?7,
                score = NULL,
                updated_at = ?8
            WHERE id = ?1
            "#,
        )
        .bind(draft.chapter_id.as_str())
        .bind(&draft.title)
        .bind(&draft.content)
        .bind(&draft.summary)
        .bind(ChapterStatus::Drafted.as_str())
        .bind(i64::from(draft.word_count))
        .bind(i64::from(draft.version))
        .bind(&now)
        .execute(self.pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO chapter_versions (
                id, chapter_id, version, title, content, summary, word_count, data, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(chapter_id, version) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                summary = excluded.summary,
                word_count = excluded.word_count,
                data = excluded.data
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(draft.chapter_id.as_str())
        .bind(i64::from(draft.version))
        .bind(&draft.title)
        .bind(&draft.content)
        .bind(&draft.summary)
        .bind(i64::from(draft.word_count))
        .bind(to_json(draft)?)
        .bind(&now)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_reviewed(
        &self,
        chapter_id: &ChapterId,
        score: i32,
        status: ChapterStatus,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            UPDATE chapters
            SET score = ?2, status = ?3, updated_at = ?4
            WHERE id = ?1
            "#,
        )
        .bind(chapter_id.as_str())
        .bind(score)
        .bind(status.as_str())
        .bind(Utc::now().to_rfc3339())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_by_novel(&self, novel_id: &NovelId) -> Result<Vec<Chapter>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT id, novel_id, volume_index, chapter_index, title, outline, content, summary,
                   status, score, word_count, version, created_at, updated_at
            FROM chapters
            WHERE novel_id = ?1
            ORDER BY chapter_index ASC
            "#,
        )
        .bind(novel_id.as_str())
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_chapter).collect()
    }
}

pub struct ChapterVersionRepository<'a> {
    pool: &'a SqlitePool,
}

impl ChapterVersionRepository<'_> {
    pub async fn count_for_chapter(&self, chapter_id: &ChapterId) -> Result<i64, StorageError> {
        let count = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM chapter_versions
            WHERE chapter_id = ?1
            "#,
        )
        .bind(chapter_id.as_str())
        .fetch_one(self.pool)
        .await?;

        Ok(count)
    }

    pub async fn list_version_numbers(
        &self,
        chapter_id: &ChapterId,
    ) -> Result<Vec<u32>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT version
            FROM chapter_versions
            WHERE chapter_id = ?1
            ORDER BY version ASC
            "#,
        )
        .bind(chapter_id.as_str())
        .fetch_all(self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| row.get::<i64, _>("version") as u32)
            .collect())
    }

    pub async fn content_for_version(
        &self,
        chapter_id: &ChapterId,
        version: u32,
    ) -> Result<Option<String>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT content
            FROM chapter_versions
            WHERE chapter_id = ?1 AND version = ?2
            "#,
        )
        .bind(chapter_id.as_str())
        .bind(i64::from(version))
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(|row| row.get("content")))
    }
}

pub struct CharacterRepository<'a> {
    pool: &'a SqlitePool,
}

impl CharacterRepository<'_> {
    pub async fn insert(&self, character: &CharacterCard) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO characters (id, novel_id, name, role, data)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                role = excluded.role,
                data = excluded.data
            "#,
        )
        .bind(character.id.as_str())
        .bind(character.novel_id.as_str())
        .bind(&character.name)
        .bind(&character.role)
        .bind(to_json(character)?)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_by_novel(
        &self,
        novel_id: &NovelId,
    ) -> Result<Vec<CharacterCard>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT data
            FROM characters
            WHERE novel_id = ?1
            ORDER BY role ASC, name ASC
            "#,
        )
        .bind(novel_id.as_str())
        .fetch_all(self.pool)
        .await?;

        rows.into_iter()
            .map(|row| from_json(row.get("data")))
            .collect()
    }
}

pub struct FactRepository<'a> {
    pool: &'a SqlitePool,
}

impl FactRepository<'_> {
    pub async fn insert_seed_facts(
        &self,
        novel_id: &NovelId,
        facts: &[FactTriple],
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            DELETE FROM facts
            WHERE novel_id = ?1 AND chapter_id IS NULL
            "#,
        )
        .bind(novel_id.as_str())
        .execute(self.pool)
        .await?;

        for fact in facts {
            let now = Utc::now();
            let id = FactId::new();
            sqlx::query(
                r#"
                INSERT INTO facts (
                    id, novel_id, chapter_id, subject, predicate, object, importance, created_at
                )
                VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7)
                "#,
            )
            .bind(id.as_str())
            .bind(novel_id.as_str())
            .bind(&fact.subject)
            .bind(&fact.predicate)
            .bind(&fact.object)
            .bind(fact.importance)
            .bind(now.to_rfc3339())
            .execute(self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn insert_for_chapter(
        &self,
        novel_id: &NovelId,
        chapter_id: &ChapterId,
        facts: &[FactTriple],
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            DELETE FROM facts
            WHERE novel_id = ?1 AND chapter_id = ?2
            "#,
        )
        .bind(novel_id.as_str())
        .bind(chapter_id.as_str())
        .execute(self.pool)
        .await?;

        for fact in facts {
            let now = Utc::now();
            let id = FactId::new();
            sqlx::query(
                r#"
                INSERT INTO facts (
                    id, novel_id, chapter_id, subject, predicate, object, importance, created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
            )
            .bind(id.as_str())
            .bind(novel_id.as_str())
            .bind(chapter_id.as_str())
            .bind(&fact.subject)
            .bind(&fact.predicate)
            .bind(&fact.object)
            .bind(fact.importance)
            .bind(now.to_rfc3339())
            .execute(self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn list_by_novel(
        &self,
        novel_id: &NovelId,
        limit: u32,
    ) -> Result<Vec<Fact>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT id, novel_id, chapter_id, subject, predicate, object, importance, created_at
            FROM facts
            WHERE novel_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )
        .bind(novel_id.as_str())
        .bind(i64::from(limit))
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_fact).collect()
    }
}

pub struct WorldSettingRepository<'a> {
    pool: &'a SqlitePool,
}

impl WorldSettingRepository<'_> {
    pub async fn save(
        &self,
        novel_id: &NovelId,
        data: &serde_json::Value,
    ) -> Result<(), StorageError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO world_settings (novel_id, data, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(novel_id) DO UPDATE SET
                data = excluded.data,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(novel_id.as_str())
        .bind(to_json(data)?)
        .bind(&now)
        .bind(&now)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn find(
        &self,
        novel_id: &NovelId,
    ) -> Result<Option<serde_json::Value>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT data
            FROM world_settings
            WHERE novel_id = ?1
            "#,
        )
        .bind(novel_id.as_str())
        .fetch_optional(self.pool)
        .await?;

        row.map(|row| from_json(row.get("data"))).transpose()
    }
}

pub struct ContinuityReportRepository<'a> {
    pool: &'a SqlitePool,
}

impl ContinuityReportRepository<'_> {
    pub async fn insert(
        &self,
        chapter_id: &ChapterId,
        passed: bool,
        data: &serde_json::Value,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO continuity_reports (id, chapter_id, passed, data, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(chapter_id.as_str())
        .bind(passed)
        .bind(to_json(data)?)
        .bind(Utc::now().to_rfc3339())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn latest_for_chapter(
        &self,
        chapter_id: &ChapterId,
    ) -> Result<Option<serde_json::Value>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT data
            FROM continuity_reports
            WHERE chapter_id = ?1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(chapter_id.as_str())
        .fetch_optional(self.pool)
        .await?;

        row.map(|row| from_json(row.get("data"))).transpose()
    }
}

pub struct ReviewReportRepository<'a> {
    pool: &'a SqlitePool,
}

impl ReviewReportRepository<'_> {
    pub async fn insert(&self, report: &ReviewReport) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO review_reports (id, chapter_id, total_score, passed, data, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(report.id.as_str())
        .bind(report.chapter_id.as_str())
        .bind(report.total_score)
        .bind(report.passed)
        .bind(to_json(report)?)
        .bind(report.created_at.to_rfc3339())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn latest_for_chapter(
        &self,
        chapter_id: &ChapterId,
    ) -> Result<Option<ReviewReport>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT data
            FROM review_reports
            WHERE chapter_id = ?1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(chapter_id.as_str())
        .fetch_optional(self.pool)
        .await?;

        row.map(|row| from_json(row.get("data"))).transpose()
    }
}

pub struct AgentRunRepository<'a> {
    pool: &'a SqlitePool,
}

pub struct JobRepository<'a> {
    pool: &'a SqlitePool,
}

#[derive(Debug, Clone)]
pub struct AgentRunRecord {
    pub id: String,
    pub novel_id: Option<NovelId>,
    pub role: String,
    pub task: String,
    pub structured: serde_json::Value,
    pub raw_text: String,
    pub raw_notes: String,
    pub parse_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobRecord {
    pub id: String,
    pub kind: String,
    pub status: JobStatus,
    pub novel_id: Option<NovelId>,
    pub chapter_index: Option<u32>,
    pub source_job_id: Option<String>,
    pub progress_current: u32,
    pub progress_total: u32,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunStatus {
    Ok,
    Fallback,
    ParseError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl FromStr for JobStatus {
    type Err = StorageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(StorageError::InvalidEnum {
                kind: "JobStatus",
                value: value.to_string(),
            }),
        }
    }
}

impl AgentRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Fallback => "fallback",
            Self::ParseError => "parse_error",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct AgentRunStatusSummary {
    pub total: usize,
    pub ok: usize,
    pub fallback: usize,
    pub parse_error: usize,
    pub duration_ms_total: u64,
    pub tokenized_runs: usize,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

impl AgentRunRecord {
    pub fn status(&self) -> AgentRunStatus {
        if self.parse_error.is_some() {
            AgentRunStatus::ParseError
        } else if self
            .structured
            .get("_engineering")
            .and_then(|value| value.get("will_fallback"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            AgentRunStatus::Fallback
        } else {
            AgentRunStatus::Ok
        }
    }

    pub fn attempt(&self) -> Option<u64> {
        self.engineering_u64("attempt")
    }

    pub fn duration_ms(&self) -> Option<u64> {
        self.engineering_u64("duration_ms")
    }

    pub fn prompt_tokens(&self) -> Option<u64> {
        self.token_usage_u64("prompt_tokens")
    }

    pub fn completion_tokens(&self) -> Option<u64> {
        self.token_usage_u64("completion_tokens")
    }

    pub fn total_tokens(&self) -> Option<u64> {
        self.token_usage_u64("total_tokens")
    }

    fn engineering_u64(&self, key: &str) -> Option<u64> {
        self.structured
            .get("_engineering")
            .and_then(|value| value.get(key))
            .and_then(serde_json::Value::as_u64)
    }

    fn token_usage_u64(&self, key: &str) -> Option<u64> {
        self.structured
            .get("_engineering")
            .and_then(|value| value.get("token_usage"))
            .and_then(|value| value.get(key))
            .and_then(serde_json::Value::as_u64)
    }
}

impl AgentRunStatusSummary {
    pub fn from_runs<'a>(runs: impl IntoIterator<Item = &'a AgentRunRecord>) -> Self {
        let mut summary = Self::default();
        for run in runs {
            summary.total += 1;
            match run.status() {
                AgentRunStatus::Ok => summary.ok += 1,
                AgentRunStatus::Fallback => summary.fallback += 1,
                AgentRunStatus::ParseError => summary.parse_error += 1,
            }
            summary.duration_ms_total += run.duration_ms().unwrap_or(0);
            if run.total_tokens().is_some() {
                summary.tokenized_runs += 1;
            }
            summary.prompt_tokens += run.prompt_tokens().unwrap_or(0);
            summary.completion_tokens += run.completion_tokens().unwrap_or(0);
            summary.total_tokens += run.total_tokens().unwrap_or(0);
        }
        summary
    }

    pub fn has_bad_status(self) -> bool {
        self.fallback > 0 || self.parse_error > 0
    }
}

impl AgentRunRepository<'_> {
    pub async fn insert(
        &self,
        novel_id: Option<&NovelId>,
        task: AgentTask,
        output: &AgentOutput,
    ) -> Result<(), StorageError> {
        let novel_id = novel_id.map(ToString::to_string);
        sqlx::query(
            r#"
            INSERT INTO agent_runs (
                id, novel_id, role, task, structured, raw_text, raw_notes, parse_error, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(novel_id)
        .bind(output.role.as_str())
        .bind(task.as_str())
        .bind(to_json(&structured_with_engineering(output))?)
        .bind(&output.raw_text)
        .bind(&output.raw_notes)
        .bind(&output.parse_error)
        .bind(Utc::now().to_rfc3339())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_recent(
        &self,
        novel_id: Option<&NovelId>,
        limit: u32,
    ) -> Result<Vec<AgentRunRecord>, StorageError> {
        let novel_id = novel_id.map(ToString::to_string);
        let rows = sqlx::query(
            r#"
            SELECT id, novel_id, role, task, structured, raw_text, raw_notes, parse_error, created_at
            FROM agent_runs
            WHERE (?1 IS NULL OR novel_id = ?1)
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )
        .bind(novel_id)
        .bind(i64::from(limit.max(1)))
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_agent_run_record).collect()
    }
}

impl JobRepository<'_> {
    pub async fn create(
        &self,
        kind: impl Into<String>,
        novel_id: Option<&NovelId>,
        chapter_index: Option<u32>,
        payload: &serde_json::Value,
    ) -> Result<JobRecord, StorageError> {
        self.create_with_source(kind, novel_id, chapter_index, payload, None)
            .await
    }

    pub async fn create_with_source(
        &self,
        kind: impl Into<String>,
        novel_id: Option<&NovelId>,
        chapter_index: Option<u32>,
        payload: &serde_json::Value,
        source_job_id: Option<&str>,
    ) -> Result<JobRecord, StorageError> {
        self.create_with_source_and_progress(
            kind,
            novel_id,
            chapter_index,
            payload,
            source_job_id,
            0,
            1,
        )
        .await
    }

    pub async fn create_with_source_and_progress(
        &self,
        kind: impl Into<String>,
        novel_id: Option<&NovelId>,
        chapter_index: Option<u32>,
        payload: &serde_json::Value,
        source_job_id: Option<&str>,
        progress_current: u32,
        progress_total: u32,
    ) -> Result<JobRecord, StorageError> {
        let now = Utc::now();
        let progress_total = progress_total.max(1);
        let progress_current = progress_current.min(progress_total);
        let job = JobRecord {
            id: uuid::Uuid::new_v4().to_string(),
            kind: kind.into(),
            status: JobStatus::Queued,
            novel_id: novel_id.cloned(),
            chapter_index,
            source_job_id: source_job_id.map(str::to_string),
            progress_current,
            progress_total,
            payload: payload.clone(),
            result: None,
            error: None,
            created_at: now,
            updated_at: now,
        };

        sqlx::query(
            r#"
            INSERT INTO api_jobs (
                id, kind, status, novel_id, chapter_index, source_job_id, progress_current, progress_total, payload, result, error, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
        )
        .bind(&job.id)
        .bind(&job.kind)
        .bind(job.status.as_str())
        .bind(job.novel_id.as_ref().map(ToString::to_string))
        .bind(job.chapter_index.map(i64::from))
        .bind(&job.source_job_id)
        .bind(i64::from(job.progress_current))
        .bind(i64::from(job.progress_total))
        .bind(to_json(&job.payload)?)
        .bind(Option::<String>::None)
        .bind(Option::<String>::None)
        .bind(job.created_at.to_rfc3339())
        .bind(job.updated_at.to_rfc3339())
        .execute(self.pool)
        .await?;

        Ok(job)
    }

    pub async fn find(&self, job_id: &str) -> Result<Option<JobRecord>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT id, kind, status, novel_id, chapter_index, source_job_id, progress_current, progress_total, payload, result, error, created_at, updated_at
            FROM api_jobs
            WHERE id = ?1
            "#,
        )
        .bind(job_id)
        .fetch_optional(self.pool)
        .await?;

        row.map(row_to_job_record).transpose()
    }

    pub async fn list_recent(&self, limit: u32) -> Result<Vec<JobRecord>, StorageError> {
        self.list_recent_filtered(limit, None, None, None, None)
            .await
    }

    pub async fn list_recent_filtered(
        &self,
        limit: u32,
        status: Option<JobStatus>,
        kind: Option<&str>,
        novel_id: Option<&str>,
        source_job_id: Option<&str>,
    ) -> Result<Vec<JobRecord>, StorageError> {
        let status = status.map(|status| status.as_str().to_string());
        let kind = kind.map(str::to_string);
        let novel_id = novel_id.map(str::to_string);
        let source_job_id = source_job_id.map(str::to_string);
        let rows = sqlx::query(
            r#"
            SELECT id, kind, status, novel_id, chapter_index, source_job_id, progress_current, progress_total, payload, result, error, created_at, updated_at
            FROM api_jobs
            WHERE (?2 IS NULL OR status = ?2)
              AND (?3 IS NULL OR kind = ?3)
              AND (?4 IS NULL OR novel_id = ?4)
              AND (?5 IS NULL OR source_job_id = ?5)
            ORDER BY updated_at DESC
            LIMIT ?1
            "#,
        )
        .bind(i64::from(limit.max(1)))
        .bind(status)
        .bind(kind)
        .bind(novel_id)
        .bind(source_job_id)
        .fetch_all(self.pool)
        .await?;

        rows.into_iter().map(row_to_job_record).collect()
    }

    pub async fn set_running(&self, job_id: &str) -> Result<bool, StorageError> {
        self.update_incomplete_status(job_id, JobStatus::Running, None, None, &[JobStatus::Queued])
            .await
    }

    pub async fn complete(
        &self,
        job_id: &str,
        result: &serde_json::Value,
    ) -> Result<bool, StorageError> {
        self.update_incomplete_status(
            job_id,
            JobStatus::Succeeded,
            Some(result),
            None,
            &[JobStatus::Queued, JobStatus::Running],
        )
        .await
    }

    pub async fn fail(&self, job_id: &str, error: &str) -> Result<bool, StorageError> {
        self.update_incomplete_status(
            job_id,
            JobStatus::Failed,
            None,
            Some(error),
            &[JobStatus::Queued, JobStatus::Running],
        )
        .await
    }

    pub async fn cancel(&self, job_id: &str, error: &str) -> Result<bool, StorageError> {
        self.update_incomplete_status(
            job_id,
            JobStatus::Cancelled,
            None,
            Some(error),
            &[JobStatus::Queued, JobStatus::Running],
        )
        .await
    }

    pub async fn set_progress(
        &self,
        job_id: &str,
        progress_current: u32,
    ) -> Result<bool, StorageError> {
        let result = sqlx::query(
            r#"
            UPDATE api_jobs
            SET progress_current = CASE
                    WHEN ?2 > progress_total THEN progress_total
                    ELSE ?2
                END,
                updated_at = ?3
            WHERE id = ?1
              AND status = ?4
            "#,
        )
        .bind(job_id)
        .bind(i64::from(progress_current))
        .bind(Utc::now().to_rfc3339())
        .bind(JobStatus::Running.as_str())
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn fail_incomplete(&self, error: &str) -> Result<u64, StorageError> {
        let result = sqlx::query(
            r#"
            UPDATE api_jobs
            SET status = ?1,
                result = NULL,
                error = ?2,
                updated_at = ?3
            WHERE status IN (?4, ?5)
            "#,
        )
        .bind(JobStatus::Failed.as_str())
        .bind(error)
        .bind(Utc::now().to_rfc3339())
        .bind(JobStatus::Queued.as_str())
        .bind(JobStatus::Running.as_str())
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn update_incomplete_status(
        &self,
        job_id: &str,
        status: JobStatus,
        result: Option<&serde_json::Value>,
        error: Option<&str>,
        allowed_statuses: &[JobStatus],
    ) -> Result<bool, StorageError> {
        debug_assert!(!allowed_statuses.is_empty());
        let result = result.map(to_json).transpose()?;
        let first_allowed = allowed_statuses[0].as_str();
        let second_allowed = allowed_statuses
            .get(1)
            .unwrap_or(&allowed_statuses[0])
            .as_str();
        let result = sqlx::query(
            r#"
            UPDATE api_jobs
            SET status = ?2,
                result = ?3,
                error = ?4,
                updated_at = ?5,
                progress_current = CASE
                    WHEN ?2 = ?8 THEN progress_total
                    ELSE progress_current
                END
            WHERE id = ?1
              AND status IN (?6, ?7)
            "#,
        )
        .bind(job_id)
        .bind(status.as_str())
        .bind(result)
        .bind(error)
        .bind(Utc::now().to_rfc3339())
        .bind(first_allowed)
        .bind(second_allowed)
        .bind(JobStatus::Succeeded.as_str())
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

fn row_to_novel(row: sqlx::sqlite::SqliteRow) -> Result<Novel, StorageError> {
    let target_platform =
        TargetPlatform::from_str(row.get::<String, _>("target_platform").as_str())?;
    let status = NovelStatus::from_str(row.get::<String, _>("status").as_str())?;

    Ok(Novel {
        id: NovelId::from(row.get::<String, _>("id")),
        title: row.get("title"),
        genre: row.get("genre"),
        target_platform,
        status,
        created_at: parse_datetime(row.get("created_at"))?,
        updated_at: parse_datetime(row.get("updated_at"))?,
    })
}

fn row_to_chapter(row: sqlx::sqlite::SqliteRow) -> Result<Chapter, StorageError> {
    let status = ChapterStatus::from_str(row.get::<String, _>("status").as_str())?;

    Ok(Chapter {
        id: ChapterId::from(row.get::<String, _>("id")),
        novel_id: NovelId::from(row.get::<String, _>("novel_id")),
        volume_index: row.get::<i64, _>("volume_index") as u32,
        chapter_index: row.get::<i64, _>("chapter_index") as u32,
        title: row.get("title"),
        outline: row.get("outline"),
        content: row.get("content"),
        summary: row.get("summary"),
        status,
        score: row.get::<Option<i64>, _>("score").map(|score| score as i32),
        word_count: row.get::<i64, _>("word_count") as u32,
        version: row.get::<i64, _>("version") as u32,
        created_at: parse_datetime(row.get("created_at"))?,
        updated_at: parse_datetime(row.get("updated_at"))?,
    })
}

fn row_to_fact(row: sqlx::sqlite::SqliteRow) -> Result<Fact, StorageError> {
    Ok(Fact {
        id: FactId::from(row.get::<String, _>("id")),
        novel_id: NovelId::from(row.get::<String, _>("novel_id")),
        chapter_id: row
            .get::<Option<String>, _>("chapter_id")
            .map(ChapterId::from),
        subject: row.get("subject"),
        predicate: row.get("predicate"),
        object: row.get("object"),
        importance: row.get::<i64, _>("importance") as i32,
        created_at: parse_datetime(row.get("created_at"))?,
    })
}

fn row_to_agent_run_record(row: sqlx::sqlite::SqliteRow) -> Result<AgentRunRecord, StorageError> {
    Ok(AgentRunRecord {
        id: row.get("id"),
        novel_id: row.get::<Option<String>, _>("novel_id").map(NovelId::from),
        role: row.get("role"),
        task: row.get("task"),
        structured: from_json(row.get("structured"))?,
        raw_text: row.get("raw_text"),
        raw_notes: row.get("raw_notes"),
        parse_error: row.get("parse_error"),
        created_at: parse_datetime(row.get("created_at"))?,
    })
}

fn row_to_job_record(row: sqlx::sqlite::SqliteRow) -> Result<JobRecord, StorageError> {
    let status = JobStatus::from_str(row.get::<String, _>("status").as_str())?;
    let result = row
        .get::<Option<String>, _>("result")
        .map(from_json)
        .transpose()?;

    Ok(JobRecord {
        id: row.get("id"),
        kind: row.get("kind"),
        status,
        novel_id: row.get::<Option<String>, _>("novel_id").map(NovelId::from),
        chapter_index: row
            .get::<Option<i64>, _>("chapter_index")
            .map(|chapter_index| chapter_index as u32),
        source_job_id: row.get("source_job_id"),
        progress_current: row.get::<i64, _>("progress_current") as u32,
        progress_total: row.get::<i64, _>("progress_total") as u32,
        payload: from_json(row.get("payload"))?,
        result,
        error: row.get("error"),
        created_at: parse_datetime(row.get("created_at"))?,
        updated_at: parse_datetime(row.get("updated_at"))?,
    })
}

fn parse_datetime(value: String) -> Result<DateTime<Utc>, StorageError> {
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| StorageError::InvalidTimestamp(value))
}

fn to_json<T: Serialize>(value: &T) -> Result<String, StorageError> {
    Ok(serde_json::to_string(value)?)
}

fn from_json<T: DeserializeOwned>(value: String) -> Result<T, StorageError> {
    Ok(serde_json::from_str(&value)?)
}

fn structured_with_engineering(output: &AgentOutput) -> serde_json::Value {
    let mut structured = output.structured.clone();
    let metadata = serde_json::json!({
        "attempt": output.attempt,
        "will_fallback": output.will_fallback,
        "parse_error": output.parse_error.as_deref(),
        "duration_ms": output.duration_ms,
        "token_usage": output.token_usage,
    });

    match &mut structured {
        serde_json::Value::Object(map) => {
            map.insert("_engineering".to_string(), metadata);
            structured
        }
        _ => serde_json::json!({
            "value": structured,
            "_engineering": metadata
        }),
    }
}

const SCHEMA: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS novels (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        genre TEXT NOT NULL,
        target_platform TEXT NOT NULL,
        status TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS novel_bibles (
        novel_id TEXT PRIMARY KEY,
        data TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY (novel_id) REFERENCES novels(id)
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS characters (
        id TEXT PRIMARY KEY,
        novel_id TEXT NOT NULL,
        name TEXT NOT NULL,
        role TEXT NOT NULL,
        data TEXT NOT NULL,
        FOREIGN KEY (novel_id) REFERENCES novels(id)
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS chapters (
        id TEXT PRIMARY KEY,
        novel_id TEXT NOT NULL,
        volume_index INTEGER NOT NULL,
        chapter_index INTEGER NOT NULL,
        title TEXT NOT NULL,
        outline TEXT NOT NULL,
        content TEXT,
        summary TEXT,
        status TEXT NOT NULL,
        score INTEGER,
        word_count INTEGER NOT NULL DEFAULT 0,
        version INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        UNIQUE (novel_id, chapter_index),
        FOREIGN KEY (novel_id) REFERENCES novels(id)
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS chapter_versions (
        id TEXT PRIMARY KEY,
        chapter_id TEXT NOT NULL,
        version INTEGER NOT NULL,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        summary TEXT NOT NULL,
        word_count INTEGER NOT NULL,
        data TEXT NOT NULL,
        created_at TEXT NOT NULL,
        UNIQUE (chapter_id, version),
        FOREIGN KEY (chapter_id) REFERENCES chapters(id)
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS facts (
        id TEXT PRIMARY KEY,
        novel_id TEXT NOT NULL,
        chapter_id TEXT,
        subject TEXT NOT NULL,
        predicate TEXT NOT NULL,
        object TEXT NOT NULL,
        importance INTEGER NOT NULL,
        created_at TEXT NOT NULL,
        FOREIGN KEY (novel_id) REFERENCES novels(id),
        FOREIGN KEY (chapter_id) REFERENCES chapters(id)
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS world_settings (
        novel_id TEXT PRIMARY KEY,
        data TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY (novel_id) REFERENCES novels(id)
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS continuity_reports (
        id TEXT PRIMARY KEY,
        chapter_id TEXT NOT NULL,
        passed INTEGER NOT NULL,
        data TEXT NOT NULL,
        created_at TEXT NOT NULL,
        FOREIGN KEY (chapter_id) REFERENCES chapters(id)
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS review_reports (
        id TEXT PRIMARY KEY,
        chapter_id TEXT NOT NULL,
        total_score INTEGER NOT NULL,
        passed INTEGER NOT NULL,
        data TEXT NOT NULL,
        created_at TEXT NOT NULL,
        FOREIGN KEY (chapter_id) REFERENCES chapters(id)
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS agent_runs (
        id TEXT PRIMARY KEY,
        novel_id TEXT,
        role TEXT NOT NULL,
        task TEXT NOT NULL,
        structured TEXT NOT NULL,
        raw_text TEXT NOT NULL,
        raw_notes TEXT NOT NULL,
        parse_error TEXT,
        created_at TEXT NOT NULL,
        FOREIGN KEY (novel_id) REFERENCES novels(id)
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS api_jobs (
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL,
        status TEXT NOT NULL,
        novel_id TEXT,
        chapter_index INTEGER,
        source_job_id TEXT,
        progress_current INTEGER NOT NULL DEFAULT 0,
        progress_total INTEGER NOT NULL DEFAULT 1,
        payload TEXT NOT NULL DEFAULT '{}',
        result TEXT,
        error TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY (novel_id) REFERENCES novels(id),
        FOREIGN KEY (source_job_id) REFERENCES api_jobs(id)
    )
    "#,
    r#"
    CREATE INDEX IF NOT EXISTS idx_api_jobs_updated_at
    ON api_jobs(updated_at DESC)
    "#,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_run_status_summary_counts_bad_statuses() {
        let ok = agent_run_record(false, None);
        let fallback = agent_run_record(true, None);
        let parse_error = agent_run_record(true, Some("invalid json"));

        assert_eq!(ok.status(), AgentRunStatus::Ok);
        assert_eq!(fallback.status(), AgentRunStatus::Fallback);
        assert_eq!(parse_error.status(), AgentRunStatus::ParseError);

        let summary = AgentRunStatusSummary::from_runs([&ok, &fallback, &parse_error]);
        assert_eq!(
            summary,
            AgentRunStatusSummary {
                total: 3,
                ok: 1,
                fallback: 1,
                parse_error: 1,
                duration_ms_total: 45,
                tokenized_runs: 3,
                prompt_tokens: 30,
                completion_tokens: 15,
                total_tokens: 45
            }
        );
        assert!(summary.has_bad_status());
    }

    #[tokio::test]
    async fn api_jobs_persist_payload_and_fail_incomplete_jobs() {
        let db_path =
            std::env::temp_dir().join(format!("novel-agent-jobs-{}.db", uuid::Uuid::new_v4()));
        let database_url = format!(
            "sqlite://{}",
            db_path.display().to_string().replace('\\', "/")
        );
        let storage = SqliteStorage::connect(&database_url).await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE api_jobs (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                status TEXT NOT NULL,
                novel_id TEXT,
                chapter_index INTEGER,
                result TEXT,
                error TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&storage.pool)
        .await
        .unwrap();

        storage.migrate().await.unwrap();
        let payload = serde_json::json!({
            "novel_id": "novel-job-test",
            "chapter_index": 7
        });
        let novel_id = NovelId::from("novel-job-test");
        let job = storage
            .jobs()
            .create("write_chapter", Some(&novel_id), Some(7), &payload)
            .await
            .unwrap();
        assert_eq!(job.payload, payload);
        assert!(job.source_job_id.is_none());
        assert_eq!(job.progress_current, 0);
        assert_eq!(job.progress_total, 1);

        let retry_job = storage
            .jobs()
            .create_with_source(
                "write_chapter",
                Some(&novel_id),
                Some(7),
                &payload,
                Some(&job.id),
            )
            .await
            .unwrap();
        assert_eq!(retry_job.source_job_id.as_deref(), Some(job.id.as_str()));
        assert_eq!(retry_job.progress_current, 0);
        assert_eq!(retry_job.progress_total, 1);

        let batch_job = storage
            .jobs()
            .create_with_source_and_progress(
                "write_chapters",
                Some(&novel_id),
                None,
                &serde_json::json!({
                    "novel_id": "novel-job-test",
                    "chapter_start": 8,
                    "chapter_end": 10
                }),
                None,
                0,
                3,
            )
            .await
            .unwrap();
        assert_eq!(batch_job.progress_current, 0);
        assert_eq!(batch_job.progress_total, 3);
        assert!(storage.jobs().set_running(&batch_job.id).await.unwrap());
        assert!(storage.jobs().set_progress(&batch_job.id, 2).await.unwrap());
        let running_batch = storage.jobs().find(&batch_job.id).await.unwrap().unwrap();
        assert_eq!(running_batch.progress_current, 2);
        assert_eq!(running_batch.progress_total, 3);
        assert!(storage
            .jobs()
            .complete(&batch_job.id, &serde_json::json!({ "drafts": [] }))
            .await
            .unwrap());
        let completed_batch = storage.jobs().find(&batch_job.id).await.unwrap().unwrap();
        assert_eq!(completed_batch.status, JobStatus::Succeeded);
        assert_eq!(completed_batch.progress_current, 3);
        assert_eq!(completed_batch.progress_total, 3);
        let filtered_batches = storage
            .jobs()
            .list_recent_filtered(
                10,
                Some(JobStatus::Succeeded),
                Some("write_chapters"),
                Some(novel_id.as_str()),
                None,
            )
            .await
            .unwrap();
        assert_eq!(filtered_batches.len(), 1);
        assert_eq!(filtered_batches[0].id, batch_job.id);

        let novel_jobs = storage
            .jobs()
            .list_recent_filtered(10, None, None, Some(novel_id.as_str()), None)
            .await
            .unwrap();
        assert_eq!(novel_jobs.len(), 3);
        assert!(novel_jobs
            .iter()
            .all(|job| job.novel_id.as_ref() == Some(&novel_id)));

        let source_jobs = storage
            .jobs()
            .list_recent_filtered(10, None, None, None, Some(&job.id))
            .await
            .unwrap();
        assert_eq!(source_jobs.len(), 1);
        assert_eq!(source_jobs[0].id, retry_job.id);
        assert_eq!(
            source_jobs[0].source_job_id.as_deref(),
            Some(job.id.as_str())
        );

        storage.jobs().set_running(&job.id).await.unwrap();
        let changed = storage
            .jobs()
            .fail_incomplete("server restarted before completion")
            .await
            .unwrap();
        assert_eq!(changed, 2);

        let failed = storage.jobs().find(&job.id).await.unwrap().unwrap();
        assert_eq!(failed.status, JobStatus::Failed);
        assert_eq!(
            failed.error.as_deref(),
            Some("server restarted before completion")
        );
        assert_eq!(failed.payload["chapter_index"].as_u64(), Some(7));
        assert!(failed.result.is_none());
        assert_eq!(failed.progress_current, 0);
        assert_eq!(failed.progress_total, 1);

        let failed_retry = storage.jobs().find(&retry_job.id).await.unwrap().unwrap();
        assert_eq!(failed_retry.status, JobStatus::Failed);
        assert_eq!(
            failed_retry.error.as_deref(),
            Some("server restarted before completion")
        );
        assert_eq!(failed_retry.source_job_id.as_deref(), Some(job.id.as_str()));
        assert_eq!(failed_retry.progress_current, 0);
        assert_eq!(failed_retry.progress_total, 1);

        let cancel_job = storage
            .jobs()
            .create("write_chapter", Some(&novel_id), Some(8), &payload)
            .await
            .unwrap();
        assert!(storage
            .jobs()
            .cancel(&cancel_job.id, "job cancelled by user")
            .await
            .unwrap());
        let cancelled = storage.jobs().find(&cancel_job.id).await.unwrap().unwrap();
        assert_eq!(cancelled.status, JobStatus::Cancelled);
        assert_eq!(cancelled.error.as_deref(), Some("job cancelled by user"));
        assert!(cancelled.result.is_none());
        assert_eq!(cancelled.progress_current, 0);
        assert_eq!(cancelled.progress_total, 1);

        assert!(!storage.jobs().set_running(&cancel_job.id).await.unwrap());
        assert!(!storage
            .jobs()
            .complete(&cancel_job.id, &serde_json::json!({ "ok": true }))
            .await
            .unwrap());
        let still_cancelled = storage.jobs().find(&cancel_job.id).await.unwrap().unwrap();
        assert_eq!(still_cancelled.status, JobStatus::Cancelled);
        assert!(still_cancelled.result.is_none());
        assert_eq!(still_cancelled.progress_current, 0);
        assert_eq!(still_cancelled.progress_total, 1);

        let _ = std::fs::remove_file(db_path);
    }

    fn agent_run_record(will_fallback: bool, parse_error: Option<&str>) -> AgentRunRecord {
        AgentRunRecord {
            id: uuid::Uuid::new_v4().to_string(),
            novel_id: Some(NovelId::from("novel-test")),
            role: "writer".to_string(),
            task: "chapter_draft".to_string(),
            structured: serde_json::json!({
                "_engineering": {
                    "will_fallback": will_fallback,
                    "duration_ms": 15,
                    "token_usage": {
                        "prompt_tokens": 10,
                        "completion_tokens": 5,
                        "total_tokens": 15
                    }
                }
            }),
            raw_text: "{}".to_string(),
            raw_notes: String::new(),
            parse_error: parse_error.map(str::to_string),
            created_at: Utc::now(),
        }
    }
}
