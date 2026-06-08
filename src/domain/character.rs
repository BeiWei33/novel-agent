use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::NovelId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharacterId(String);

impl CharacterId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CharacterId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for CharacterId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for CharacterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCard {
    pub id: CharacterId,
    pub novel_id: NovelId,
    pub id_hint: String,
    pub name: String,
    pub role: String,
    pub identity: String,
    pub personality: Vec<String>,
    pub desire: String,
    pub motivation: String,
    pub secret: String,
    pub abilities: Vec<String>,
    pub limitations: Vec<String>,
    pub current_state: String,
    pub relationship_map: Vec<CharacterRelationship>,
    pub arc: CharacterArc,
    pub first_appearance_chapter: u32,
    pub chapter_1_to_30_plan: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRelationship {
    pub target: String,
    pub relationship: String,
    pub tension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterArc {
    pub start: String,
    pub turning_points: Vec<String>,
    pub expected_end: String,
}
