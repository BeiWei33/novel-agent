use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::NovelId;
use crate::error::StorageError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChapterId(String);

impl ChapterId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ChapterId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for ChapterId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for ChapterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: ChapterId,
    pub novel_id: NovelId,
    pub volume_index: u32,
    pub chapter_index: u32,
    pub title: String,
    pub outline: String,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub status: ChapterStatus,
    pub score: Option<i32>,
    pub word_count: u32,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Chapter {
    pub fn outlined(
        novel_id: NovelId,
        volume_index: u32,
        chapter_index: u32,
        title: impl Into<String>,
        outline: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ChapterId::new(),
            novel_id,
            volume_index,
            chapter_index,
            title: title.into(),
            outline: outline.into(),
            content: None,
            summary: None,
            status: ChapterStatus::Outlined,
            score: None,
            word_count: 0,
            version: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterOutline {
    pub novel_id: NovelId,
    pub volume_index: u32,
    pub chapter_index: u32,
    pub title: String,
    pub pov: String,
    pub goal: String,
    pub conflict: String,
    pub key_events: Vec<String>,
    pub character_changes: Vec<String>,
    pub new_facts: Vec<FactTriple>,
    pub payoff: String,
    pub foreshadowing: Vec<String>,
    pub cliffhanger: String,
    pub estimated_word_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterDraft {
    pub volume_index: u32,
    pub chapter_id: ChapterId,
    pub novel_id: NovelId,
    pub chapter_index: u32,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub word_count: u32,
    pub pov: String,
    pub key_events: Vec<String>,
    pub new_facts: Vec<FactTriple>,
    pub foreshadowing: Vec<Foreshadowing>,
    pub continuity_notes: Vec<String>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteInstruction {
    pub chapter_id: ChapterId,
    pub scope: String,
    pub problems: Vec<String>,
    pub goals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub importance: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Foreshadowing {
    pub seed: String,
    pub status: String,
    pub expected_payoff: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChapterStatus {
    Outlined,
    Drafted,
    Reviewed,
    RewriteNeeded,
    Final,
}

impl ChapterStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Outlined => "outlined",
            Self::Drafted => "drafted",
            Self::Reviewed => "reviewed",
            Self::RewriteNeeded => "rewrite_needed",
            Self::Final => "final",
        }
    }
}

impl fmt::Display for ChapterStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ChapterStatus {
    type Err = StorageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "outlined" => Ok(Self::Outlined),
            "drafted" => Ok(Self::Drafted),
            "reviewed" => Ok(Self::Reviewed),
            "rewrite_needed" => Ok(Self::RewriteNeeded),
            "final" => Ok(Self::Final),
            _ => Err(StorageError::InvalidEnum {
                kind: "ChapterStatus",
                value: value.to_string(),
            }),
        }
    }
}
