use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

use crate::agents::AgentStorage;
use crate::domain::{
    Chapter, ChapterDraft, ChapterId, ChapterStatus, CharacterCard, Novel, NovelBible, NovelId,
    NovelStatus, ReviewReport, TargetPlatform,
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
        Ok(())
    }

    pub fn novels(&self) -> NovelRepository<'_> {
        NovelRepository { pool: &self.pool }
    }

    pub fn chapters(&self) -> ChapterRepository<'_> {
        ChapterRepository { pool: &self.pool }
    }

    pub fn characters(&self) -> CharacterRepository<'_> {
        CharacterRepository { pool: &self.pool }
    }

    pub fn review_reports(&self) -> ReviewReportRepository<'_> {
        ReviewReportRepository { pool: &self.pool }
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
        sqlx::query(
            r#"
            UPDATE chapters
            SET title = ?2,
                content = ?3,
                summary = ?4,
                status = ?5,
                word_count = ?6,
                version = ?7,
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
        .bind(Utc::now().to_rfc3339())
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
}

fn row_to_novel(row: sqlx::sqlite::SqliteRow) -> Result<Novel, StorageError> {
    let target_platform = TargetPlatform::from_str(row.get::<String, _>("target_platform").as_str())?;
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
];
