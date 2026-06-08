use std::path::PathBuf;

use chrono::Utc;

use crate::domain::{
    Chapter, ChapterDraft, ChapterStatus, FactTriple, Foreshadowing, NovelId, ReviewIssue,
    ReviewReport, ReviewReportId, ReviewScores, RewriteDecision,
};
use crate::error::WorkflowError;
use crate::storage::SqliteStorage;

pub struct ChapterGenerationWorkflow<'a> {
    storage: &'a SqliteStorage,
}

impl<'a> ChapterGenerationWorkflow<'a> {
    pub fn new(storage: &'a SqliteStorage) -> Self {
        Self { storage }
    }

    pub async fn write_chapter(
        &self,
        novel_id: &NovelId,
        chapter_index: u32,
    ) -> Result<ChapterDraft, WorkflowError> {
        let novel = self
            .storage
            .novels()
            .find(novel_id)
            .await?
            .ok_or_else(|| WorkflowError::NovelNotFound(novel_id.to_string()))?;
        let chapter = match self
            .storage
            .chapters()
            .find_by_index(novel_id, chapter_index)
            .await?
        {
            Some(chapter) => chapter,
            None => {
                let chapter = Chapter::outlined(
                    novel_id.clone(),
                    1,
                    chapter_index,
                    format!("第{}章 待生成", chapter_index),
                    "临时章节大纲：等待 Plot Agent 补全。".to_string(),
                );
                self.storage.chapters().upsert_outline(&chapter).await?;
                chapter
            }
        };

        let content = render_placeholder_chapter(&novel.title, &chapter);
        let summary = format!(
            "第{}章围绕「{}」推进，完成一次小冲突和一个章尾期待。",
            chapter.chapter_index, chapter.title
        );
        let draft = ChapterDraft {
            volume_index: chapter.volume_index,
            chapter_id: chapter.id.clone(),
            novel_id: chapter.novel_id.clone(),
            chapter_index: chapter.chapter_index,
            title: chapter.title.clone(),
            word_count: count_words(&content),
            content,
            summary,
            pov: "第三人称有限视角".to_string(),
            key_events: vec!["主角确认当前局势并做出主动选择。".to_string()],
            new_facts: vec![FactTriple {
                subject: "林舟".to_string(),
                predicate: "确认".to_string(),
                object: format!("第{}章发生的局势变化", chapter.chapter_index),
                importance: 2,
            }],
            foreshadowing: vec![Foreshadowing {
                seed: "提前出现的人".to_string(),
                status: "planted".to_string(),
                expected_payoff: "下一章揭示其提前出现的原因。".to_string(),
            }],
            continuity_notes: vec!["占位生成版本未调用 Continuity Agent。".to_string()],
            version: chapter.version + 1,
        };

        self.storage.chapters().save_draft(&draft).await?;
        Ok(draft)
    }

    pub async fn review_chapter(
        &self,
        novel_id: &NovelId,
        chapter_index: u32,
    ) -> Result<ReviewReport, WorkflowError> {
        let chapter = self
            .storage
            .chapters()
            .find_by_index(novel_id, chapter_index)
            .await?
            .ok_or_else(|| WorkflowError::ChapterNotFound {
                novel_id: novel_id.to_string(),
                chapter: chapter_index,
            })?;

        let content = chapter.content.clone().unwrap_or_default();
        let (total_score, scores) = score_placeholder_chapter(&content);
        let passed = scores.passes_default_line(total_score);
        let issues = if passed {
            vec![]
        } else {
            vec![ReviewIssue {
                severity: "medium".to_string(),
                dimension: "pacing".to_string(),
                location: "整章".to_string(),
                description: "当前章节还缺少足够明确的情绪回报或章尾推进。".to_string(),
            }]
        };
        let suggestions = if passed {
            vec!["保持当前推进节奏，下一章优先回收本章章尾压力。".to_string()]
        } else {
            vec![
                "强化主角当前目标，让读者知道本章必须解决什么。".to_string(),
                "章尾增加一个新的压力点或信息差。".to_string(),
            ]
        };
        let report = ReviewReport {
            id: ReviewReportId::new(),
            chapter_id: chapter.id.clone(),
            total_score,
            passed,
            scores,
            strengths: if passed {
                vec!["目标、冲突和章尾期待已达到 MVP 发布线。".to_string()]
            } else {
                vec![]
            },
            issues,
            suggestions,
            rewrite_instruction: if passed {
                RewriteDecision::none()
            } else {
                RewriteDecision::partial(
                    vec!["强化本章目标和章尾钩子。".to_string()],
                    vec!["压缩解释性段落，增加主角行动和对手施压。".to_string()],
                )
            },
            created_at: Utc::now(),
        };

        self.storage.review_reports().insert(&report).await?;
        self.storage
            .chapters()
            .mark_reviewed(
                &chapter.id,
                report.total_score,
                if report.passed {
                    ChapterStatus::Final
                } else {
                    ChapterStatus::RewriteNeeded
                },
            )
            .await?;

        Ok(report)
    }

    pub async fn rewrite_chapter(
        &self,
        novel_id: &NovelId,
        chapter_index: u32,
    ) -> Result<ChapterDraft, WorkflowError> {
        let chapter = self
            .storage
            .chapters()
            .find_by_index(novel_id, chapter_index)
            .await?
            .ok_or_else(|| WorkflowError::ChapterNotFound {
                novel_id: novel_id.to_string(),
                chapter: chapter_index,
            })?;

        if chapter.content.is_none() {
            return self.write_chapter(novel_id, chapter_index).await;
        }

        let mut content = chapter.content.unwrap_or_default();
        content.push_str("\n\n【重写标记】本版本需要根据审稿意见强化目标、冲突和章尾钩子。");
        let draft = ChapterDraft {
            volume_index: chapter.volume_index,
            chapter_id: chapter.id.clone(),
            novel_id: chapter.novel_id.clone(),
            chapter_index: chapter.chapter_index,
            title: chapter.title,
            summary: format!("第{}章重写版本，强化目标、冲突和钩子。", chapter.chapter_index),
            word_count: count_words(&content),
            content,
            pov: "第三人称有限视角".to_string(),
            key_events: vec!["根据审稿意见强化章节冲突链。".to_string()],
            new_facts: vec![],
            foreshadowing: vec![Foreshadowing {
                seed: "重写版本强化后的章尾钩子".to_string(),
                status: "advanced".to_string(),
                expected_payoff: "下一章兑现或升级压力。".to_string(),
            }],
            continuity_notes: vec!["重写后需要再次执行 Continuity Agent。".to_string()],
            version: chapter.version + 1,
        };

        self.storage.chapters().save_draft(&draft).await?;
        Ok(draft)
    }

    pub async fn export_markdown(
        &self,
        novel_id: &NovelId,
        output: Option<PathBuf>,
    ) -> Result<PathBuf, WorkflowError> {
        let novel = self
            .storage
            .novels()
            .find(novel_id)
            .await?
            .ok_or_else(|| WorkflowError::NovelNotFound(novel_id.to_string()))?;
        let chapters = self.storage.chapters().list_by_novel(novel_id).await?;
        let path = output.unwrap_or_else(|| PathBuf::from(format!("exports/{}.md", novel_id)));

        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut markdown = format!("# {}\n\n", novel.title);
        for chapter in chapters {
            markdown.push_str(&format!("## {}\n\n", chapter.title));
            if let Some(content) = chapter.content {
                markdown.push_str(&content);
            } else {
                markdown.push_str(&chapter.outline);
            }
            markdown.push_str("\n\n");
        }

        tokio::fs::write(&path, markdown).await?;
        Ok(path)
    }
}

fn render_placeholder_chapter(novel_title: &str, chapter: &Chapter) -> String {
    format!(
        "《{}》\n\n{}\n\n{}\n\n林舟站在旧日的路口，终于确认自己真的回到了命运转折之前。\n他没有急着证明什么，而是先把眼前能抓住的资源一项项列出来。\n对手的压力很快落下，熟悉的规则再次试图把他推回原位。\n这一次，他选择先退半步，换一个所有人都看不懂的入口。\n\n章尾，林舟发现原本应该三天后才出现的人，提前出现在门外。",
        novel_title, chapter.title, chapter.outline
    )
}

fn score_placeholder_chapter(content: &str) -> (i32, ReviewScores) {
    let has_content = !content.trim().is_empty();
    let total = if has_content { 78 } else { 45 };

    (
        total,
        ReviewScores {
            opening_hook_score: if has_content { 7 } else { 3 },
            pacing_score: if has_content { 7 } else { 3 },
            payoff_score: if has_content { 7 } else { 3 },
            character_score: if has_content { 7 } else { 3 },
            dialogue_score: if has_content { 6 } else { 2 },
            continuity_score: if has_content { 8 } else { 3 },
            cliffhanger_score: if has_content { 8 } else { 2 },
            platform_fit_score: if has_content { 7 } else { 3 },
        },
    )
}

fn count_words(content: &str) -> u32 {
    content.chars().filter(|ch| !ch.is_whitespace()).count() as u32
}
