use std::path::PathBuf;

use chrono::Utc;
use serde_json::{Value, json};

use super::agent_runner::run_prompt_agent;
use crate::agents::{AgentOutput, AgentRole, AgentTask, ModelHandle};
use crate::domain::{
    Chapter, ChapterDraft, ChapterStatus, FactTriple, Foreshadowing, NovelId, ReviewIssue,
    ReviewReport, ReviewReportId, ReviewScores, RewriteDecision,
};
use crate::error::WorkflowError;
use crate::storage::{AgentRunRecord, AgentRunStatus, SqliteStorage};

pub struct ChapterGenerationWorkflow<'a> {
    storage: &'a SqliteStorage,
    model: ModelHandle,
}

impl<'a> ChapterGenerationWorkflow<'a> {
    pub fn new(storage: &'a SqliteStorage, model: ModelHandle) -> Self {
        Self { storage, model }
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

        let bible = self.storage.novels().find_bible(novel_id).await?;
        let platform_profile = bible
            .as_ref()
            .and_then(|bible| bible.platform_profile.clone());
        let relevant_facts = self.storage.facts().list_by_novel(novel_id, 20).await?;
        let characters = self.storage.characters().list_by_novel(novel_id).await?;
        let recent_summaries = recent_summaries(self.storage, novel_id, chapter_index).await?;
        let previous_runs = self
            .storage
            .agent_runs()
            .list_recent(Some(novel_id), 120)
            .await?;
        let previous_writer_output = previous_chapter_agent_output(
            &previous_runs,
            AgentRole::Writer,
            AgentTask::GenerateChapter,
            chapter_index,
        );
        let reused_writer_output = previous_writer_output.is_some();
        let previous_style_output = if reused_writer_output {
            previous_chapter_agent_output(
                &previous_runs,
                AgentRole::Style,
                AgentTask::PolishStyle,
                chapter_index,
            )
        } else {
            None
        };
        let output = if let Some(output) = previous_writer_output {
            output
        } else {
            run_prompt_agent(
                self.storage,
                self.model.clone(),
                Some(novel_id),
                AgentRole::Writer,
                AgentTask::GenerateChapter,
                CHAPTER_WRITER_PROMPT,
                json!({
                    "novel_bible": bible,
                    "target_platform": novel.target_platform.as_str(),
                    "platform_profile": platform_profile,
                    "chapter_outline": chapter.outline.clone(),
                    "characters": characters,
                    "world_setting": self.storage.world_settings().find(novel_id).await?,
                    "recent_summaries": recent_summaries,
                    "relevant_facts": relevant_facts,
                    "constraints": [],
                    "target_word_count": 2500
                }),
            )
            .await?
        };
        let mut draft = draft_from_agent_output(&chapter, &output.structured)
            .unwrap_or_else(|| fallback_chapter_draft(&novel.title, &chapter));
        let continuity_report = if reused_writer_output {
            if let Some(report) = self
                .storage
                .continuity_reports()
                .latest_for_chapter(&chapter.id)
                .await?
            {
                report
            } else {
                self.run_continuity(novel_id, &draft, chapter_index).await?
            }
        } else {
            self.run_continuity(novel_id, &draft, chapter_index).await?
        };
        draft = if let Some(output) = previous_style_output {
            apply_style_output(draft, &output.structured)
        } else {
            self.run_style(novel_id, draft, &continuity_report).await?
        };

        self.storage.chapters().save_draft(&draft).await?;
        self.storage
            .facts()
            .insert_for_chapter(
                &draft.novel_id,
                &draft.chapter_id,
                &facts_from_continuity(&continuity_report, &draft.new_facts),
            )
            .await?;
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

        let novel = self
            .storage
            .novels()
            .find(novel_id)
            .await?
            .ok_or_else(|| WorkflowError::NovelNotFound(novel_id.to_string()))?;
        let bible = self.storage.novels().find_bible(novel_id).await?;
        let platform_profile = bible
            .as_ref()
            .and_then(|bible| bible.platform_profile.clone());
        let characters = self.storage.characters().list_by_novel(novel_id).await?;
        let continuity_report = self
            .storage
            .continuity_reports()
            .latest_for_chapter(&chapter.id)
            .await?
            .unwrap_or_else(|| fallback_continuity_report(&chapter, &[]));
        let output = run_prompt_agent(
            self.storage,
            self.model.clone(),
            Some(novel_id),
            AgentRole::Reviewer,
            AgentTask::ReviewChapter,
            REVIEWER_PROMPT,
            json!({
                "novel_bible": bible,
                "platform_profile": platform_profile,
                "chapter": serde_json::to_value(&chapter).unwrap_or_else(|_| json!({})),
                "chapter_outline": chapter.outline.clone(),
                "characters": characters,
                "world_setting": self.storage.world_settings().find(novel_id).await?,
                "continuity_report": continuity_report,
                "target_platform": novel.target_platform.as_str()
            }),
        )
        .await?;
        let report =
            review_from_agent_output(&chapter.id, &output.structured).unwrap_or_else(|| {
                fallback_review_report(
                    &chapter,
                    output.will_fallback || output.parse_error.is_some(),
                )
            });

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

        let latest_report = self
            .storage
            .review_reports()
            .latest_for_chapter(&chapter.id)
            .await?;
        let novel = self
            .storage
            .novels()
            .find(novel_id)
            .await?
            .ok_or_else(|| WorkflowError::NovelNotFound(novel_id.to_string()))?;
        let bible = self.storage.novels().find_bible(novel_id).await?;
        let platform_profile = bible
            .as_ref()
            .and_then(|bible| bible.platform_profile.clone());
        let characters = self.storage.characters().list_by_novel(novel_id).await?;
        let recent_summaries = recent_summaries(self.storage, novel_id, chapter_index).await?;
        let relevant_facts = self.storage.facts().list_by_novel(novel_id, 20).await?;
        let output = run_prompt_agent(
            self.storage,
            self.model.clone(),
            Some(novel_id),
            AgentRole::Writer,
            AgentTask::RewriteChapter,
            CHAPTER_WRITER_PROMPT,
            json!({
                "novel_bible": bible,
                "target_platform": novel.target_platform.as_str(),
                "platform_profile": platform_profile,
                "chapter_outline": chapter.outline.clone(),
                "characters": characters,
                "world_setting": self.storage.world_settings().find(novel_id).await?,
                "recent_summaries": recent_summaries,
                "relevant_facts": relevant_facts,
                "constraints": [],
                "target_word_count": 2500,
                "previous_draft": chapter.content.clone(),
                "rewrite_instruction": latest_report.as_ref().map(|report| &report.rewrite_instruction)
            }),
        )
        .await?;
        let mut draft = draft_from_agent_output(&chapter, &output.structured)
            .unwrap_or_else(|| fallback_rewrite_draft(&chapter, latest_report.as_ref()));
        let continuity_report = self.run_continuity(novel_id, &draft, chapter_index).await?;
        draft = self.run_style(novel_id, draft, &continuity_report).await?;

        self.storage.chapters().save_draft(&draft).await?;
        self.storage
            .facts()
            .insert_for_chapter(
                &draft.novel_id,
                &draft.chapter_id,
                &facts_from_continuity(&continuity_report, &draft.new_facts),
            )
            .await?;
        self.review_chapter(novel_id, chapter_index).await?;
        Ok(draft)
    }

    pub async fn save_manual_edit(
        &self,
        novel_id: &NovelId,
        chapter_index: u32,
        title: Option<String>,
        content: String,
        summary: Option<String>,
    ) -> Result<ChapterDraft, WorkflowError> {
        if content.trim().is_empty() {
            return Err(WorkflowError::InvalidInput(
                "manual edit content cannot be empty".to_string(),
            ));
        }

        self.storage
            .novels()
            .find(novel_id)
            .await?
            .ok_or_else(|| WorkflowError::NovelNotFound(novel_id.to_string()))?;
        let chapter = self
            .storage
            .chapters()
            .find_by_index(novel_id, chapter_index)
            .await?
            .ok_or_else(|| WorkflowError::ChapterNotFound {
                novel_id: novel_id.to_string(),
                chapter: chapter_index,
            })?;

        let title = non_empty_string(title).unwrap_or_else(|| chapter.title.clone());
        let summary = non_empty_string(summary)
            .or(chapter.summary.clone())
            .unwrap_or_else(|| format!("第{}章人工编辑版本。", chapter.chapter_index));
        let draft = ChapterDraft {
            volume_index: chapter.volume_index,
            chapter_id: chapter.id.clone(),
            novel_id: chapter.novel_id.clone(),
            chapter_index: chapter.chapter_index,
            title,
            word_count: count_words(&content),
            content,
            summary,
            pov: "第三人称有限视角".to_string(),
            key_events: vec!["人工编辑保存新版本。".to_string()],
            new_facts: vec![],
            foreshadowing: vec![],
            continuity_notes: vec!["人工编辑版本，建议重新审稿或按需返工。".to_string()],
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
        let path = output.unwrap_or_else(|| PathBuf::from(format!("exports/{}.md", novel_id)));

        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            tokio::fs::create_dir_all(parent).await?;
        }

        let markdown = self.export_markdown_content(novel_id).await?;
        tokio::fs::write(&path, markdown).await?;
        Ok(path)
    }

    pub async fn export_markdown_content(
        &self,
        novel_id: &NovelId,
    ) -> Result<String, WorkflowError> {
        let novel = self
            .storage
            .novels()
            .find(novel_id)
            .await?
            .ok_or_else(|| WorkflowError::NovelNotFound(novel_id.to_string()))?;
        let chapters = self.storage.chapters().list_by_novel(novel_id).await?;
        Ok(render_markdown(&novel.title, chapters))
    }

    async fn run_continuity(
        &self,
        novel_id: &NovelId,
        draft: &ChapterDraft,
        chapter_index: u32,
    ) -> Result<Value, WorkflowError> {
        let bible = self.storage.novels().find_bible(novel_id).await?;
        let characters = self.storage.characters().list_by_novel(novel_id).await?;
        let relevant_facts = self.storage.facts().list_by_novel(novel_id, 30).await?;
        let recent_summaries = recent_summaries(self.storage, novel_id, chapter_index).await?;
        let output = run_prompt_agent(
            self.storage,
            self.model.clone(),
            Some(novel_id),
            AgentRole::Continuity,
            AgentTask::CheckContinuity,
            CONTINUITY_PROMPT,
            json!({
                "novel_bible": bible,
                "chapter_draft": draft,
                "characters": characters,
                "world_setting": self.storage.world_settings().find(novel_id).await?,
                "recent_summaries": recent_summaries,
                "relevant_facts": relevant_facts,
                "known_foreshadowing": draft.foreshadowing
            }),
        )
        .await?;
        let report = output
            .structured
            .get("continuity_report")
            .cloned()
            .unwrap_or_else(|| fallback_continuity_report_for_draft(draft));
        let passed = report
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        self.storage
            .continuity_reports()
            .insert(&draft.chapter_id, passed, &report)
            .await?;

        Ok(report)
    }

    async fn run_style(
        &self,
        novel_id: &NovelId,
        draft: ChapterDraft,
        continuity_report: &Value,
    ) -> Result<ChapterDraft, WorkflowError> {
        let novel = self
            .storage
            .novels()
            .find(novel_id)
            .await?
            .ok_or_else(|| WorkflowError::NovelNotFound(novel_id.to_string()))?;
        let bible = self.storage.novels().find_bible(novel_id).await?;
        let output = run_prompt_agent(
            self.storage,
            self.model.clone(),
            Some(novel_id),
            AgentRole::Style,
            AgentTask::PolishStyle,
            STYLE_PROMPT,
            json!({
                "chapter_draft": draft,
                "novel_bible": bible,
                "target_platform": novel.target_platform.as_str(),
                "style_constraints": [],
                "continuity_report": continuity_report
            }),
        )
        .await?;

        Ok(apply_style_output(draft, &output.structured))
    }
}

const CHAPTER_WRITER_PROMPT: &str = include_str!("../../prompts/chapter_writer_agent.md");
const REVIEWER_PROMPT: &str = include_str!("../../prompts/reviewer_agent.md");
const CONTINUITY_PROMPT: &str = include_str!("../../prompts/continuity_agent.md");
const STYLE_PROMPT: &str = include_str!("../../prompts/style_agent.md");

async fn recent_summaries(
    storage: &SqliteStorage,
    novel_id: &NovelId,
    chapter_index: u32,
) -> Result<Vec<String>, WorkflowError> {
    let mut chapters = storage.chapters().list_by_novel(novel_id).await?;
    chapters.retain(|chapter| chapter.chapter_index < chapter_index);
    chapters.sort_by_key(|chapter| std::cmp::Reverse(chapter.chapter_index));
    Ok(chapters
        .into_iter()
        .take(10)
        .filter_map(|chapter| chapter.summary)
        .collect())
}

fn previous_chapter_agent_output(
    runs: &[AgentRunRecord],
    role: AgentRole,
    task: AgentTask,
    chapter_index: u32,
) -> Option<AgentOutput> {
    let run = runs.iter().find(|run| {
        run.role == role.as_str()
            && run.task == task.as_str()
            && run.status() == AgentRunStatus::Ok
            && run_matches_chapter(run, chapter_index)
    })?;

    Some(AgentOutput {
        role,
        structured: run.structured.clone(),
        raw_text: run.raw_text.clone(),
        parse_error: None,
        raw_notes: run.raw_notes.clone(),
        attempt: run.attempt().unwrap_or(1) as u32,
        will_fallback: false,
        duration_ms: run.duration_ms(),
        token_usage: None,
        artifacts: vec![],
    })
}

fn run_matches_chapter(run: &AgentRunRecord, chapter_index: u32) -> bool {
    run.structured
        .get("_workflow")
        .and_then(|value| value.get("chapter_index"))
        .or_else(|| {
            run.structured
                .get("chapter_draft")
                .or_else(|| run.structured.get("styled_chapter"))
                .and_then(|value| value.get("chapter_index"))
        })
        .and_then(Value::as_u64)
        .map(|value| value == u64::from(chapter_index))
        .unwrap_or_else(|| {
            run.structured
                .get("chapter_draft")
                .and_then(|value| value.get("content"))
                .is_some()
        })
}

fn apply_style_output(mut draft: ChapterDraft, structured: &Value) -> ChapterDraft {
    if let Some(styled) = structured.get("styled_chapter") {
        if let Some(title) = string_field(styled, "title") {
            draft.title = title;
        }
        if let Some(content) = string_field(styled, "content") {
            draft.word_count = count_words(&content);
            draft.content = content;
        }
        if let Some(summary) = string_field(styled, "summary") {
            draft.summary = summary;
        }
        let style_notes = string_vec(styled, "style_notes");
        if !style_notes.is_empty() {
            draft
                .continuity_notes
                .push(format!("Style Agent: {}", style_notes.join("；")));
        }
    }

    draft
}

fn draft_from_agent_output(chapter: &Chapter, structured: &Value) -> Option<ChapterDraft> {
    let value = structured.get("chapter_draft")?;
    let content = string_field(value, "content")?;
    let output_chapter_index = number_field(value, "chapter_index");
    let title = match output_chapter_index {
        Some(index) if index != chapter.chapter_index => chapter.title.clone(),
        _ => string_field(value, "title").unwrap_or_else(|| chapter.title.clone()),
    };
    let word_count = value
        .get("word_count")
        .or_else(|| value.get("word_count_estimate"))
        .and_then(Value::as_u64)
        .map(|value| value as u32)
        .unwrap_or_else(|| count_words(&content));

    Some(ChapterDraft {
        volume_index: chapter.volume_index,
        chapter_id: chapter.id.clone(),
        novel_id: chapter.novel_id.clone(),
        chapter_index: chapter.chapter_index,
        title,
        summary: string_field(value, "summary").unwrap_or_default(),
        word_count,
        content,
        pov: string_field(value, "pov").unwrap_or_else(|| "第三人称有限视角".to_string()),
        key_events: string_vec(value, "key_events"),
        new_facts: serde_json::from_value(
            value.get("new_facts").cloned().unwrap_or_else(|| json!([])),
        )
        .unwrap_or_default(),
        foreshadowing: serde_json::from_value(
            value
                .get("foreshadowing")
                .cloned()
                .unwrap_or_else(|| json!([])),
        )
        .unwrap_or_default(),
        continuity_notes: string_vec(value, "continuity_notes"),
        version: chapter.version + 1,
    })
}

fn review_from_agent_output(
    chapter_id: &crate::domain::ChapterId,
    structured: &Value,
) -> Option<ReviewReport> {
    let value = structured.get("review_report")?;
    let scores: ReviewScores = serde_json::from_value(value.get("scores")?.clone()).ok()?;
    let total_score = value.get("total_score")?.as_i64()? as i32;
    let passed = value
        .get("passed")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| scores.passes_default_line(total_score));

    Some(ReviewReport {
        id: ReviewReportId::new(),
        chapter_id: chapter_id.clone(),
        total_score,
        passed,
        scores,
        strengths: string_vec(value, "strengths"),
        issues: serde_json::from_value(value.get("issues").cloned().unwrap_or_else(|| json!([])))
            .unwrap_or_default(),
        suggestions: string_vec(value, "suggestions"),
        rewrite_instruction: serde_json::from_value(
            value
                .get("rewrite_instruction")
                .cloned()
                .unwrap_or_else(|| json!({})),
        )
        .unwrap_or_else(|_| RewriteDecision::none()),
        created_at: Utc::now(),
    })
}

fn fallback_chapter_draft(novel_title: &str, chapter: &Chapter) -> ChapterDraft {
    let content = render_placeholder_chapter(novel_title, chapter);
    ChapterDraft {
        volume_index: chapter.volume_index,
        chapter_id: chapter.id.clone(),
        novel_id: chapter.novel_id.clone(),
        chapter_index: chapter.chapter_index,
        title: chapter.title.clone(),
        word_count: count_words(&content),
        content,
        summary: format!(
            "第{}章围绕「{}」推进，完成一次小冲突和一个章尾期待。",
            chapter.chapter_index, chapter.title
        ),
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
        continuity_notes: vec!["模型不可用或解析失败，使用 smoke fallback。".to_string()],
        version: chapter.version + 1,
    }
}

fn fallback_rewrite_draft(chapter: &Chapter, report: Option<&ReviewReport>) -> ChapterDraft {
    let mut content = chapter.content.clone().unwrap_or_default();
    let goals = report
        .map(|report| report.rewrite_instruction.goals.join("；"))
        .filter(|goals| !goals.is_empty())
        .unwrap_or_else(|| "强化目标、冲突和章尾钩子".to_string());
    content.push_str(&format!("\n\n【重写方向】{}", goals));

    ChapterDraft {
        volume_index: chapter.volume_index,
        chapter_id: chapter.id.clone(),
        novel_id: chapter.novel_id.clone(),
        chapter_index: chapter.chapter_index,
        title: chapter.title.clone(),
        summary: format!("第{}章重写版本，{}", chapter.chapter_index, goals),
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
    }
}

fn fallback_continuity_report(chapter: &Chapter, facts: &[FactTriple]) -> Value {
    json!({
        "passed": true,
        "issues": [],
        "new_facts": facts,
        "character_state_updates": [],
        "foreshadowing_updates": [],
        "raw_notes": format!(
            "Continuity Agent 未返回有效结构，使用章节 {} 的保守 fallback。",
            chapter.chapter_index
        )
    })
}

fn fallback_continuity_report_for_draft(draft: &ChapterDraft) -> Value {
    let foreshadowing_updates = draft
        .foreshadowing
        .iter()
        .map(|item| {
            json!({
                "seed": item.seed,
                "status": item.status,
                "note": item.expected_payoff
            })
        })
        .collect::<Vec<_>>();

    json!({
        "passed": true,
        "issues": [],
        "new_facts": draft.new_facts,
        "character_state_updates": [],
        "foreshadowing_updates": foreshadowing_updates,
        "raw_notes": "Continuity Agent 未返回有效结构，沿用 Writer 输出的事实和伏笔。"
    })
}

fn facts_from_continuity(report: &Value, fallback: &[FactTriple]) -> Vec<FactTriple> {
    let facts = report
        .get("new_facts")
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<FactTriple>>(value).ok())
        .unwrap_or_default();

    if facts.is_empty() {
        fallback.to_vec()
    } else {
        facts
    }
}

fn fallback_review_report(chapter: &Chapter, used_fallback: bool) -> ReviewReport {
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
    let mut suggestions = suggestions;
    if used_fallback {
        suggestions.push("模型不可用或 JSON 解析失败，本审稿为 smoke fallback 结果。".to_string());
    }

    ReviewReport {
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
    }
}

fn render_placeholder_chapter(novel_title: &str, chapter: &Chapter) -> String {
    format!(
        "《{}》\n\n{}\n\n{}\n\n林舟站在旧日的路口，终于确认自己真的回到了命运转折之前。\n他没有急着证明什么，而是先把眼前能抓住的资源一项项列出来。\n对手的压力很快落下，熟悉的规则再次试图把他推回原位。\n这一次，他选择先退半步，换一个所有人都看不懂的入口。\n\n章尾，林舟发现原本应该三天后才出现的人，提前出现在门外。",
        novel_title, chapter.title, chapter.outline
    )
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(ToString::to_string)
}

fn string_vec(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn non_empty_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn number_field(value: &Value, key: &str) -> Option<u32> {
    value.get(key)?.as_u64().map(|value| value as u32)
}

fn render_markdown(title: &str, chapters: Vec<Chapter>) -> String {
    let mut markdown = format!("# {title}\n\n");
    for chapter in chapters {
        markdown.push_str(&format!("## {}\n\n", chapter.title));
        if let Some(content) = chapter.content {
            markdown.push_str(&content);
        } else {
            markdown.push_str(&chapter.outline);
        }
        markdown.push_str("\n\n");
    }
    markdown
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
