mod chapter;
mod character;
mod fact;
mod novel;
mod review;

pub use chapter::{
    Chapter, ChapterDraft, ChapterId, ChapterOutline, ChapterStatus, FactTriple, Foreshadowing,
    RewriteInstruction,
};
pub use character::{CharacterArc, CharacterCard, CharacterId, CharacterRelationship};
pub use fact::{Fact, FactId};
pub use novel::{
    Novel, NovelBible, NovelId, NovelStatus, OpeningStrategy, PlatformProfile, TargetPlatform,
    TitleCandidate,
};
pub use review::{ReviewIssue, ReviewReport, ReviewReportId, ReviewScores, RewriteDecision};
