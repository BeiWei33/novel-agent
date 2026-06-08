use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ChapterId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReviewReportId(String);

impl ReviewReportId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ReviewReportId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for ReviewReportId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for ReviewReportId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub id: ReviewReportId,
    pub chapter_id: ChapterId,
    pub total_score: i32,
    pub passed: bool,
    pub scores: ReviewScores,
    pub strengths: Vec<String>,
    pub issues: Vec<ReviewIssue>,
    pub suggestions: Vec<String>,
    pub rewrite_instruction: RewriteDecision,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewScores {
    pub opening_hook_score: i32,
    pub pacing_score: i32,
    pub payoff_score: i32,
    pub character_score: i32,
    pub dialogue_score: i32,
    pub continuity_score: i32,
    pub cliffhanger_score: i32,
    pub platform_fit_score: i32,
}

impl ReviewScores {
    pub fn passes_default_line(&self, total_score: i32) -> bool {
        total_score >= 75
            && self.cliffhanger_score >= 7
            && self.continuity_score >= 8
            && self.pacing_score >= 7
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    pub severity: String,
    pub dimension: String,
    pub location: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteDecision {
    pub needed: bool,
    pub rewrite_type: String,
    pub priority: String,
    pub goals: Vec<String>,
    pub preserve: Vec<String>,
    pub change: Vec<String>,
    pub avoid: Vec<String>,
}

impl RewriteDecision {
    pub fn none() -> Self {
        Self {
            needed: false,
            rewrite_type: "none".to_string(),
            priority: "low".to_string(),
            goals: vec![],
            preserve: vec![],
            change: vec![],
            avoid: vec![],
        }
    }

    pub fn partial(goals: Vec<String>, change: Vec<String>) -> Self {
        Self {
            needed: true,
            rewrite_type: "partial".to_string(),
            priority: "medium".to_string(),
            goals,
            preserve: vec!["保留章节核心事实和主线推进。".to_string()],
            change,
            avoid: vec!["避免只增加解释而不增加行动。".to_string()],
        }
    }
}
