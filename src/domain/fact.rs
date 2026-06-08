use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ChapterId, NovelId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactId(String);

impl FactId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for FactId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for FactId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for FactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: FactId,
    pub novel_id: NovelId,
    pub chapter_id: Option<ChapterId>,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub importance: i32,
    pub created_at: DateTime<Utc>,
}
