use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::StorageError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NovelId(String);

impl NovelId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for NovelId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for NovelId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for NovelId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl fmt::Display for NovelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Novel {
    pub id: NovelId,
    pub title: String,
    pub genre: String,
    pub target_platform: TargetPlatform,
    pub status: NovelStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Novel {
    pub fn draft(title: impl Into<String>, genre: impl Into<String>, platform: TargetPlatform) -> Self {
        let now = Utc::now();
        Self {
            id: NovelId::new(),
            title: title.into(),
            genre: genre.into(),
            target_platform: platform,
            status: NovelStatus::Draft,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelBible {
    pub novel_id: NovelId,
    pub title_candidates: Vec<TitleCandidate>,
    pub premise: String,
    pub genre: String,
    pub target_platform: TargetPlatform,
    pub target_readers: String,
    pub core_selling_points: Vec<String>,
    pub reader_expectations: Vec<String>,
    pub main_conflict: String,
    pub protagonist_goal: String,
    pub emotional_value: String,
    pub tone: String,
    pub platform_tags: Vec<String>,
    pub world_rules: Vec<String>,
    pub constraints: Vec<String>,
    pub opening_strategy: OpeningStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleCandidate {
    pub title: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpeningStrategy {
    pub first_scene: String,
    pub first_conflict: String,
    pub first_three_chapters_goal: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetPlatform {
    General,
    Qidian,
    Fanqie,
}

impl TargetPlatform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Qidian => "qidian",
            Self::Fanqie => "fanqie",
        }
    }
}

impl fmt::Display for TargetPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TargetPlatform {
    type Err = StorageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "general" | "generic" | "通用" => Ok(Self::General),
            "qidian" | "起点" => Ok(Self::Qidian),
            "fanqie" | "番茄" => Ok(Self::Fanqie),
            _ => Err(StorageError::InvalidEnum {
                kind: "TargetPlatform",
                value: value.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NovelStatus {
    Draft,
    Active,
    Completed,
    Archived,
}

impl NovelStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Archived => "archived",
        }
    }
}

impl fmt::Display for NovelStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NovelStatus {
    type Err = StorageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            "archived" => Ok(Self::Archived),
            _ => Err(StorageError::InvalidEnum {
                kind: "NovelStatus",
                value: value.to_string(),
            }),
        }
    }
}
