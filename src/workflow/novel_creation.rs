use serde::Deserialize;
use serde_json::{Value, json};

use super::agent_runner::run_prompt_agent;
use crate::agents::{AgentOutput, AgentRole, AgentTask, ModelHandle};
use crate::domain::{
    Chapter, ChapterOutline, CharacterArc, CharacterCard, CharacterId, CharacterRelationship,
    FactTriple, Novel, NovelBible, NovelId, OpeningStrategy, PlatformProfile, TargetPlatform,
    TitleCandidate,
};
use crate::error::WorkflowError;
use crate::storage::{AgentRunRecord, AgentRunStatus, SqliteStorage};

pub struct NovelCreationWorkflow<'a> {
    storage: &'a SqliteStorage,
    model: ModelHandle,
}

#[derive(Debug, Clone)]
pub struct NovelCreationResult {
    pub novel: Novel,
    pub bible: NovelBible,
    pub characters: Vec<CharacterCard>,
    pub outlines: Vec<ChapterOutline>,
    pub used_fallback: bool,
}

const DEFAULT_OUTLINE_BATCH_SIZE: u32 = 5;

const MARKET_AGENT_PROMPT: &str = include_str!("../../prompts/market_agent.md");
const PLOT_AGENT_PROMPT: &str = include_str!("../../prompts/plot_agent.md");
const CHARACTER_AGENT_PROMPT: &str = include_str!("../../prompts/character_agent.md");
const WORLDBUILDING_AGENT_PROMPT: &str = include_str!("../../prompts/worldbuilding_agent.md");

impl<'a> NovelCreationWorkflow<'a> {
    pub fn new(storage: &'a SqliteStorage, model: ModelHandle) -> Self {
        Self { storage, model }
    }

    pub async fn create_from_idea(
        &self,
        idea: &str,
        platform: TargetPlatform,
    ) -> Result<NovelCreationResult, WorkflowError> {
        self.create_from_idea_with_chapters(idea, platform, 30)
            .await
    }

    pub async fn create_from_idea_with_chapters(
        &self,
        idea: &str,
        platform: TargetPlatform,
        chapters: u32,
    ) -> Result<NovelCreationResult, WorkflowError> {
        self.create_from_idea_with_outline_batch_size(
            idea,
            platform,
            chapters,
            DEFAULT_OUTLINE_BATCH_SIZE,
        )
        .await
    }

    pub async fn create_from_idea_with_outline_batch_size(
        &self,
        idea: &str,
        platform: TargetPlatform,
        chapters: u32,
        outline_batch_size: u32,
    ) -> Result<NovelCreationResult, WorkflowError> {
        self.create_or_resume_from_idea_with_outline_batch_size(
            None,
            idea,
            platform,
            chapters,
            outline_batch_size,
        )
        .await
    }

    pub async fn resume_from_idea_with_outline_batch_size(
        &self,
        novel_id: &NovelId,
        idea: &str,
        chapters: u32,
        outline_batch_size: u32,
    ) -> Result<NovelCreationResult, WorkflowError> {
        let novel = self
            .storage
            .novels()
            .find(novel_id)
            .await?
            .ok_or_else(|| WorkflowError::NovelNotFound(novel_id.to_string()))?;
        self.create_or_resume_from_idea_with_outline_batch_size(
            Some(novel_id),
            idea,
            novel.target_platform,
            chapters,
            outline_batch_size,
        )
        .await
    }

    async fn create_or_resume_from_idea_with_outline_batch_size(
        &self,
        resume_novel_id: Option<&NovelId>,
        idea: &str,
        platform: TargetPlatform,
        chapters: u32,
        outline_batch_size: u32,
    ) -> Result<NovelCreationResult, WorkflowError> {
        let target_chapters = chapters.max(1);
        let outline_batch_size = outline_batch_size.max(1);
        let mut novel = if let Some(novel_id) = resume_novel_id {
            self.storage
                .novels()
                .find(novel_id)
                .await?
                .ok_or_else(|| WorkflowError::NovelNotFound(novel_id.to_string()))?
        } else {
            let novel = Novel::draft(infer_title(idea), infer_genre(idea), platform);
            self.storage.novels().insert(&novel).await?;
            novel
        };
        let previous_runs = self
            .storage
            .agent_runs()
            .list_recent(Some(&novel.id), 120)
            .await?;

        let market_output = if let Some(output) =
            previous_agent_output(&previous_runs, AgentRole::Market, AgentTask::CreateNovel)
        {
            output
        } else {
            run_prompt_agent(
                self.storage,
                self.model.clone(),
                Some(&novel.id),
                AgentRole::Market,
                AgentTask::CreateNovel,
                MARKET_AGENT_PROMPT,
                json!({
                    "idea": idea,
                    "target_platform": platform.as_str(),
                    "genre_hint": novel.genre.clone(),
                    "constraints": []
                }),
            )
            .await?
        };
        if let Some(title) = first_title_candidate(&market_output.structured) {
            novel.title = title;
        }

        let seed_plot = plot_generation_from_agent_runs(&novel.id, &previous_runs, target_chapters);
        let plot_generation = self
            .generate_plot_outline_batches_with_seed(
                &novel.id,
                idea,
                market_output
                    .structured
                    .get("market_analysis")
                    .cloned()
                    .unwrap_or_else(|| json!({})),
                platform,
                target_chapters,
                outline_batch_size,
                seed_plot,
            )
            .await?;
        let character_output = if let Some(output) =
            previous_agent_output(&previous_runs, AgentRole::Character, AgentTask::CreateNovel)
        {
            output
        } else {
            run_prompt_agent(
            self.storage,
            self.model.clone(),
            Some(&novel.id),
            AgentRole::Character,
            AgentTask::CreateNovel,
            CHARACTER_AGENT_PROMPT,
            json!({
                "idea": idea,
                "market_analysis": market_output.structured.get("market_analysis").cloned().unwrap_or_else(|| json!({})),
                "plot_plan": plot_generation.structured.get("plot_plan").cloned().unwrap_or_else(|| json!({})),
                "target_platform": platform.as_str(),
                "existing_characters": [],
                "scope": {
                    "focus_chapters": target_chapters,
                    "max_characters": 4,
                    "max_relationships_per_character": 2,
                    "max_turning_points_per_character": 3,
                    "max_plan_items_per_character": 4
                }
            }),
        )
            .await?
        };
        let provisional_characters =
            characters_from_agent_output(&novel, &character_output.structured)
                .unwrap_or_else(|| draft_characters(&novel, idea));
        let stored_characters = if resume_novel_id.is_some() {
            self.storage.characters().list_by_novel(&novel.id).await?
        } else {
            Vec::new()
        };
        let should_insert_characters = stored_characters.is_empty();
        let characters = if stored_characters.is_empty() {
            provisional_characters
        } else {
            stored_characters
        };
        let worldbuilding_output = if let Some(output) = previous_agent_output(
            &previous_runs,
            AgentRole::Worldbuilding,
            AgentTask::CreateNovel,
        ) {
            output
        } else {
            run_prompt_agent(
            self.storage,
            self.model.clone(),
            Some(&novel.id),
            AgentRole::Worldbuilding,
            AgentTask::CreateNovel,
            WORLDBUILDING_AGENT_PROMPT,
            json!({
                "idea": idea,
                "market_analysis": market_output.structured.get("market_analysis").cloned().unwrap_or_else(|| json!({})),
                "plot_plan": plot_generation.structured.get("plot_plan").cloned().unwrap_or_else(|| json!({})),
                "characters": worldbuilding_character_context(&characters),
                "target_platform": platform.as_str(),
                "scope": {
                    "focus_chapters": target_chapters,
                    "max_organizations": 2,
                    "max_locations": 3,
                    "max_facts_to_seed": 8
                }
            }),
        )
            .await?
        };
        let used_fallback = market_output.will_fallback
            || plot_generation.used_fallback
            || character_output.will_fallback
            || worldbuilding_output.will_fallback
            || market_output.parse_error.is_some()
            || character_output.parse_error.is_some()
            || worldbuilding_output.parse_error.is_some();

        let bible = bible_from_agent_outputs(
            &novel,
            idea,
            &market_output.structured,
            &plot_generation.structured,
        )
        .unwrap_or_else(|| draft_bible(&novel, idea));
        let outlines = plot_generation.outlines;

        self.storage.novels().insert(&novel).await?;
        self.storage.novels().save_bible(&bible).await?;

        if should_insert_characters {
            for character in &characters {
                self.storage.characters().insert(character).await?;
            }
        }

        if let Some(world_setting) = worldbuilding_output.structured.get("world_setting") {
            self.storage
                .world_settings()
                .save(&novel.id, world_setting)
                .await?;
        }

        let seed_facts = facts_from_value(
            worldbuilding_output
                .structured
                .get("facts_to_seed")
                .cloned()
                .unwrap_or_else(|| json!([])),
        );
        self.storage
            .facts()
            .insert_seed_facts(&novel.id, &seed_facts)
            .await?;

        for outline in &outlines {
            let chapter = Chapter::outlined(
                outline.novel_id.clone(),
                outline.volume_index,
                outline.chapter_index,
                outline.title.clone(),
                outline_to_text(outline),
            );
            self.storage.chapters().upsert_outline(&chapter).await?;
        }

        Ok(NovelCreationResult {
            novel,
            bible,
            characters,
            outlines,
            used_fallback,
        })
    }

    pub async fn generate_outline(
        &self,
        novel_id: &NovelId,
        chapters: u32,
    ) -> Result<Vec<ChapterOutline>, WorkflowError> {
        self.generate_outline_with_batch_size(novel_id, chapters, DEFAULT_OUTLINE_BATCH_SIZE)
            .await
    }

    pub async fn generate_outline_with_batch_size(
        &self,
        novel_id: &NovelId,
        chapters: u32,
        outline_batch_size: u32,
    ) -> Result<Vec<ChapterOutline>, WorkflowError> {
        let outline_batch_size = outline_batch_size.max(1);
        let novel = self
            .storage
            .novels()
            .find(novel_id)
            .await?
            .ok_or_else(|| WorkflowError::NovelNotFound(novel_id.to_string()))?;
        let idea = self
            .storage
            .novels()
            .find_bible(novel_id)
            .await?
            .map(|bible| bible.premise)
            .unwrap_or_else(|| novel.title.clone());
        let outlines = self
            .generate_plot_outline_batches(
                &novel.id,
                &idea,
                json!({}),
                novel.target_platform,
                chapters.max(1),
                outline_batch_size,
            )
            .await?
            .outlines;

        for outline in &outlines {
            let chapter = Chapter::outlined(
                outline.novel_id.clone(),
                outline.volume_index,
                outline.chapter_index,
                outline.title.clone(),
                outline_to_text(outline),
            );
            self.storage.chapters().upsert_outline(&chapter).await?;
        }

        Ok(outlines)
    }

    async fn generate_plot_outline_batches(
        &self,
        novel_id: &NovelId,
        idea: &str,
        market_analysis: Value,
        target_platform: TargetPlatform,
        target_chapters: u32,
        outline_batch_size: u32,
    ) -> Result<PlotOutlineGeneration, WorkflowError> {
        self.generate_plot_outline_batches_with_seed(
            novel_id,
            idea,
            market_analysis,
            target_platform,
            target_chapters,
            outline_batch_size,
            None,
        )
        .await
    }

    async fn generate_plot_outline_batches_with_seed(
        &self,
        novel_id: &NovelId,
        idea: &str,
        market_analysis: Value,
        target_platform: TargetPlatform,
        target_chapters: u32,
        outline_batch_size: u32,
        seed: Option<PlotOutlineGeneration>,
    ) -> Result<PlotOutlineGeneration, WorkflowError> {
        let target_chapters = target_chapters.max(1);
        let outline_batch_size = outline_batch_size.max(1);
        let mut all_outlines = seed
            .as_ref()
            .map(|seed| seed.outlines.clone())
            .unwrap_or_default();
        all_outlines.retain(|outline| (1..=target_chapters).contains(&outline.chapter_index));
        all_outlines.sort_by_key(|outline| outline.chapter_index);
        all_outlines.dedup_by_key(|outline| outline.chapter_index);
        let mut plot_plan = seed
            .and_then(|seed| seed.structured.get("plot_plan").cloned())
            .filter(|value| !value.is_null());
        let mut used_fallback = false;
        let mut start = 1;

        while start <= target_chapters {
            let end = target_chapters.min(start + outline_batch_size - 1);
            let existing = missing_outline_indices(start, end, &all_outlines).is_empty();
            if existing {
                start = end + 1;
                continue;
            }

            let batch_size = end - start + 1;
            let output = run_prompt_agent(
                self.storage,
                self.model.clone(),
                Some(novel_id),
                AgentRole::Plot,
                AgentTask::GenerateOutline,
                PLOT_AGENT_PROMPT,
                json!({
                    "idea": idea,
                    "market_analysis": market_analysis.clone(),
                    "target_platform": target_platform.as_str(),
                    "target_chapters": batch_size,
                    "total_chapters": target_chapters,
                    "chapter_start": start,
                    "chapter_end": end,
                    "known_constraints": [],
                    "existing_plot_plan": plot_plan.clone().unwrap_or_else(|| json!({})),
                    "previous_chapter_outlines": outline_context(&all_outlines),
                    "batch_policy": {
                        "output_only_this_range": true,
                        "keep_absolute_chapter_index": true
                    }
                }),
            )
            .await?;

            if plot_plan.is_none() {
                plot_plan = output.structured.get("plot_plan").cloned();
            }

            let mut batch_outlines = if output.parse_error.is_none() {
                normalize_batch_outlines(novel_id, &output.structured, start, end)
            } else {
                Vec::new()
            };

            let missing = missing_outline_indices(start, end, &batch_outlines);
            if output.will_fallback || output.parse_error.is_some() || !missing.is_empty() {
                used_fallback = true;
            }
            if !missing.is_empty() {
                let fallback = draft_outlines_for_indices(novel_id, idea, &missing);
                batch_outlines.extend(fallback);
            }

            batch_outlines.sort_by_key(|outline| outline.chapter_index);
            all_outlines.retain(|outline| !(start..=end).contains(&outline.chapter_index));
            all_outlines.extend(batch_outlines);
            all_outlines.sort_by_key(|outline| outline.chapter_index);
            all_outlines.dedup_by_key(|outline| outline.chapter_index);
            start = end + 1;
        }

        all_outlines.sort_by_key(|outline| outline.chapter_index);
        all_outlines.dedup_by_key(|outline| outline.chapter_index);
        if all_outlines.len() < target_chapters as usize {
            let missing = missing_outline_indices(1, target_chapters, &all_outlines);
            all_outlines.extend(draft_outlines_for_indices(novel_id, idea, &missing));
            all_outlines.sort_by_key(|outline| outline.chapter_index);
            used_fallback = true;
        }

        Ok(PlotOutlineGeneration {
            structured: combined_plot_structured(
                plot_plan.unwrap_or_else(|| json!({})),
                &all_outlines,
            ),
            outlines: all_outlines,
            used_fallback,
        })
    }
}

struct PlotOutlineGeneration {
    structured: Value,
    outlines: Vec<ChapterOutline>,
    used_fallback: bool,
}

fn previous_agent_output(
    runs: &[AgentRunRecord],
    role: AgentRole,
    task: AgentTask,
) -> Option<AgentOutput> {
    let run = runs.iter().find(|run| {
        run.role == role.as_str() && run.task == task.as_str() && run.status() == AgentRunStatus::Ok
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

fn plot_generation_from_agent_runs(
    novel_id: &NovelId,
    runs: &[AgentRunRecord],
    target_chapters: u32,
) -> Option<PlotOutlineGeneration> {
    let mut outlines = Vec::new();
    let mut plot_plan = None;

    for run in runs.iter().rev() {
        if run.role != AgentRole::Plot.as_str()
            || run.task != AgentTask::GenerateOutline.as_str()
            || run.status() != AgentRunStatus::Ok
        {
            continue;
        }
        if plot_plan.is_none() {
            plot_plan = run.structured.get("plot_plan").cloned();
        }
        outlines.extend(normalize_batch_outlines(
            novel_id,
            &run.structured,
            1,
            target_chapters,
        ));
    }

    outlines.retain(|outline| (1..=target_chapters).contains(&outline.chapter_index));
    outlines.sort_by_key(|outline| outline.chapter_index);
    outlines.dedup_by_key(|outline| outline.chapter_index);
    if outlines.is_empty() {
        return None;
    }

    Some(PlotOutlineGeneration {
        structured: combined_plot_structured(plot_plan.unwrap_or_else(|| json!({})), &outlines),
        outlines,
        used_fallback: false,
    })
}

fn bible_from_agent_outputs(
    novel: &Novel,
    idea: &str,
    market: &Value,
    plot: &Value,
) -> Option<NovelBible> {
    let market_analysis = market.get("market_analysis")?;
    let plot_plan = plot.get("plot_plan");
    let opening_strategy = market
        .get("opening_strategy")
        .cloned()
        .and_then(|value| serde_json::from_value::<OpeningStrategy>(value).ok())
        .unwrap_or_else(|| draft_bible(novel, idea).opening_strategy);
    let title_candidates = market
        .get("title_candidates")
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<TitleCandidate>>(value).ok())
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| draft_bible(novel, idea).title_candidates);
    let platform_profile = market
        .get("platform_profile")
        .cloned()
        .and_then(|value| serde_json::from_value::<PlatformProfile>(value).ok());

    Some(NovelBible {
        novel_id: novel.id.clone(),
        title_candidates,
        premise: string_vec(market_analysis, "core_selling_points")
            .first()
            .cloned()
            .unwrap_or_else(|| draft_bible(novel, idea).premise),
        genre: string_field(market_analysis, "genre").unwrap_or_else(|| novel.genre.clone()),
        target_platform: novel.target_platform,
        target_readers: string_field(market_analysis, "target_readers")
            .unwrap_or_else(|| platform_reader(novel.target_platform).to_string()),
        core_selling_points: string_vec(market_analysis, "core_selling_points"),
        reader_expectations: string_vec(market_analysis, "reader_expectations"),
        main_conflict: plot_plan
            .and_then(|plan| string_field(plan, "main_conflict"))
            .unwrap_or_else(|| draft_bible(novel, idea).main_conflict),
        protagonist_goal: plot_plan
            .and_then(|plan| string_field(plan, "protagonist_goal"))
            .unwrap_or_else(|| draft_bible(novel, idea).protagonist_goal),
        emotional_value: string_vec(market_analysis, "emotional_hooks")
            .join("；")
            .if_empty_else(|| draft_bible(novel, idea).emotional_value),
        tone: "中文网文，节奏明确，信息密度高，章尾保持期待。".to_string(),
        platform_tags: string_vec(market_analysis, "platform_tags"),
        world_rules: vec![
            "关键能力和资源变化必须进入事实表。".to_string(),
            "反派与配角需要独立动机，不能只服务主角升级。".to_string(),
        ],
        constraints: vec![
            "不仿写指定作者风格。".to_string(),
            "不直接复用已有小说正文或受版权保护的世界观。".to_string(),
        ],
        opening_strategy,
        platform_profile,
    })
}

fn characters_from_agent_output(novel: &Novel, structured: &Value) -> Option<Vec<CharacterCard>> {
    let values = structured.get("characters")?.as_array()?;
    let characters = values
        .iter()
        .filter_map(|value| serde_json::from_value::<GeneratedCharacter>(value.clone()).ok())
        .map(|character| CharacterCard {
            id: CharacterId::new(),
            novel_id: novel.id.clone(),
            id_hint: character.id_hint,
            name: character.name,
            role: character.role,
            identity: character.identity,
            personality: character.personality,
            desire: character.desire,
            motivation: character.motivation,
            secret: character.secret,
            abilities: character.abilities,
            limitations: character.limitations,
            current_state: character.current_state,
            relationship_map: character.relationship_map,
            arc: character.arc,
            first_appearance_chapter: character.first_appearance_chapter,
            chapter_1_to_30_plan: character.chapter_1_to_30_plan,
        })
        .collect::<Vec<_>>();

    (!characters.is_empty()).then_some(characters)
}

fn outlines_from_agent_output(
    novel_id: &NovelId,
    structured: &Value,
) -> Option<Vec<ChapterOutline>> {
    let values = structured.get("chapter_outlines")?.as_array()?;
    let outlines = values
        .iter()
        .filter_map(|value| serde_json::from_value::<GeneratedChapterOutline>(value.clone()).ok())
        .map(|outline| ChapterOutline {
            novel_id: novel_id.clone(),
            volume_index: outline.volume_index,
            chapter_index: outline.chapter_index,
            title: outline.title,
            pov: outline.pov,
            goal: outline.goal,
            conflict: outline.conflict,
            key_events: outline.key_events,
            character_changes: outline.character_changes,
            new_facts: outline.new_facts,
            payoff: outline.payoff,
            foreshadowing: outline.foreshadowing,
            cliffhanger: outline.cliffhanger,
            estimated_word_count: outline.estimated_word_count,
        })
        .collect::<Vec<_>>();

    (!outlines.is_empty()).then_some(outlines)
}

fn normalize_batch_outlines(
    novel_id: &NovelId,
    structured: &Value,
    start: u32,
    end: u32,
) -> Vec<ChapterOutline> {
    let Some(outlines) = outlines_from_agent_output(novel_id, structured) else {
        return Vec::new();
    };
    let mut in_range = outlines
        .iter()
        .filter(|outline| (start..=end).contains(&outline.chapter_index))
        .cloned()
        .collect::<Vec<_>>();
    if !in_range.is_empty() {
        in_range.sort_by_key(|outline| outline.chapter_index);
        in_range.dedup_by_key(|outline| outline.chapter_index);
        return in_range;
    }

    let expected = (end - start + 1) as usize;
    outlines
        .into_iter()
        .take(expected)
        .enumerate()
        .map(|(offset, mut outline)| {
            outline.novel_id = novel_id.clone();
            outline.chapter_index = start + offset as u32;
            outline
        })
        .collect()
}

fn combined_plot_structured(plot_plan: Value, outlines: &[ChapterOutline]) -> Value {
    let chapter_outlines = outlines
        .iter()
        .map(chapter_outline_to_value)
        .collect::<Vec<_>>();

    json!({
        "plot_plan": plot_plan,
        "chapter_outlines": chapter_outlines
    })
}

fn chapter_outline_to_value(outline: &ChapterOutline) -> Value {
    json!({
        "volume_index": outline.volume_index,
        "chapter_index": outline.chapter_index,
        "title": outline.title,
        "pov": outline.pov,
        "goal": outline.goal,
        "conflict": outline.conflict,
        "key_events": outline.key_events,
        "character_changes": outline.character_changes,
        "new_facts": outline.new_facts,
        "foreshadowing": outline.foreshadowing,
        "payoff": outline.payoff,
        "cliffhanger": outline.cliffhanger,
        "estimated_word_count": outline.estimated_word_count
    })
}

fn outline_context(outlines: &[ChapterOutline]) -> Vec<Value> {
    outlines
        .iter()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|outline| {
            json!({
                "chapter_index": outline.chapter_index,
                "title": outline.title,
                "goal": outline.goal,
                "cliffhanger": outline.cliffhanger
            })
        })
        .collect()
}

fn worldbuilding_character_context(characters: &[CharacterCard]) -> Vec<Value> {
    characters
        .iter()
        .take(6)
        .map(|character| {
            json!({
                "id_hint": character.id_hint,
                "name": character.name,
                "role": character.role,
                "identity": character.identity,
                "desire": character.desire,
                "motivation": character.motivation,
                "limitations": limited_strings(&character.limitations, 3),
                "current_state": character.current_state,
                "first_appearance_chapter": character.first_appearance_chapter,
                "chapter_1_to_30_plan": limited_strings(&character.chapter_1_to_30_plan, 3)
            })
        })
        .collect()
}

fn limited_strings(values: &[String], max: usize) -> Vec<String> {
    values.iter().take(max).cloned().collect()
}

fn missing_outline_indices(start: u32, end: u32, outlines: &[ChapterOutline]) -> Vec<u32> {
    (start..=end)
        .filter(|index| {
            !outlines
                .iter()
                .any(|outline| outline.chapter_index == *index)
        })
        .collect()
}

fn first_title_candidate(structured: &Value) -> Option<String> {
    structured
        .get("title_candidates")?
        .as_array()?
        .first()?
        .get("title")?
        .as_str()
        .map(ToString::to_string)
}

fn facts_from_value(value: Value) -> Vec<FactTriple> {
    serde_json::from_value(value).unwrap_or_default()
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

trait EmptyStringExt {
    fn if_empty_else(self, fallback: impl FnOnce() -> String) -> String;
}

impl EmptyStringExt for String {
    fn if_empty_else(self, fallback: impl FnOnce() -> String) -> String {
        if self.trim().is_empty() {
            fallback()
        } else {
            self
        }
    }
}

#[derive(Debug, Deserialize)]
struct GeneratedCharacter {
    id_hint: String,
    name: String,
    role: String,
    identity: String,
    personality: Vec<String>,
    desire: String,
    motivation: String,
    secret: String,
    abilities: Vec<String>,
    limitations: Vec<String>,
    current_state: String,
    relationship_map: Vec<CharacterRelationship>,
    arc: CharacterArc,
    first_appearance_chapter: u32,
    chapter_1_to_30_plan: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GeneratedChapterOutline {
    volume_index: u32,
    chapter_index: u32,
    title: String,
    pov: String,
    goal: String,
    conflict: String,
    key_events: Vec<String>,
    character_changes: Vec<String>,
    new_facts: Vec<FactTriple>,
    foreshadowing: Vec<String>,
    payoff: String,
    cliffhanger: String,
    estimated_word_count: u32,
}

fn draft_bible(novel: &Novel, idea: &str) -> NovelBible {
    NovelBible {
        novel_id: novel.id.clone(),
        title_candidates: vec![TitleCandidate {
            title: novel.title.clone(),
            reason: "先用工程占位书名承接创意，后续由 Market Agent 生成候选。".to_string(),
        }],
        premise: format!("围绕「{}」建立清晰主线，前期用强冲突快速拉住读者。", idea),
        genre: novel.genre.clone(),
        target_platform: novel.target_platform,
        target_readers: platform_reader(novel.target_platform).to_string(),
        core_selling_points: vec![
            "开篇给出明确困境和逆袭目标".to_string(),
            "每 3-5 章形成一个小回报周期".to_string(),
            platform_selling_point(novel.target_platform).to_string(),
        ],
        reader_expectations: vec![
            "主角主动破局".to_string(),
            "阶段性反击兑现".to_string(),
            "重要伏笔可追踪回收".to_string(),
        ],
        main_conflict: "主角想改变命运，但资源、信息差和对手持续制造压力。".to_string(),
        protagonist_goal: "抓住第二次机会，建立自己的主动权。".to_string(),
        emotional_value: "压迫感后的翻盘、成长后的掌控感、伏笔回收时的爽感。".to_string(),
        tone: "中文网文，节奏明确，信息密度高，章尾保持期待。".to_string(),
        platform_tags: vec![novel.genre.clone(), novel.target_platform.to_string()],
        world_rules: vec![
            "关键能力和资源变化必须进入事实表。".to_string(),
            "反派与配角需要独立动机，不能只服务主角升级。".to_string(),
        ],
        constraints: vec![
            "不仿写指定作者风格。".to_string(),
            "不直接复用已有小说正文或受版权保护的世界观。".to_string(),
        ],
        opening_strategy: OpeningStrategy {
            first_scene: "主角回到命运转折点，先确认变化，再立刻做选择。".to_string(),
            first_conflict: "资源不足与旧日对手施压同时出现。".to_string(),
            first_three_chapters_goal: "建立主角目标、主要对手和第一轮反击期待。".to_string(),
        },
        platform_profile: Some(default_platform_profile(novel.target_platform)),
    }
}

fn default_platform_profile(platform: TargetPlatform) -> PlatformProfile {
    match platform {
        TargetPlatform::Qidian => PlatformProfile {
            target_platform: platform,
            opening_speed: "layered".to_string(),
            setup_ratio: 0.35,
            dialogue_ratio: 0.30,
            payoff_frequency: "every_2_chapters".to_string(),
            cliffhanger_strength: "medium".to_string(),
            review_bias: json!({
                "continuity_score": 2,
                "platform_fit_score": 1
            }),
        },
        TargetPlatform::Fanqie => PlatformProfile {
            target_platform: platform,
            opening_speed: "fast".to_string(),
            setup_ratio: 0.18,
            dialogue_ratio: 0.42,
            payoff_frequency: "every_chapter".to_string(),
            cliffhanger_strength: "high".to_string(),
            review_bias: json!({
                "opening_hook_score": 2,
                "pacing_score": 2,
                "payoff_score": 1,
                "cliffhanger_score": 1
            }),
        },
        TargetPlatform::General => PlatformProfile {
            target_platform: platform,
            opening_speed: "balanced".to_string(),
            setup_ratio: 0.25,
            dialogue_ratio: 0.35,
            payoff_frequency: "every_chapter".to_string(),
            cliffhanger_strength: "medium".to_string(),
            review_bias: json!({
                "platform_fit_score": 1
            }),
        },
    }
}

fn draft_characters(novel: &Novel, idea: &str) -> Vec<CharacterCard> {
    vec![
        CharacterCard {
            id: CharacterId::new(),
            novel_id: novel.id.clone(),
            id_hint: "protagonist".to_string(),
            name: "林舟".to_string(),
            role: "protagonist".to_string(),
            identity: infer_protagonist_identity(idea).to_string(),
            personality: vec![
                "冷静".to_string(),
                "抗压".to_string(),
                "行动力强".to_string(),
            ],
            desire: "抓住第二次机会，建立自己的主动权。".to_string(),
            motivation: "不再让重要的人和机会从手里流失。".to_string(),
            secret: "掌握一段只有自己知道的未来信息。".to_string(),
            abilities: vec!["复盘局势".to_string(), "快速执行".to_string()],
            limitations: vec![
                "前期资源不足".to_string(),
                "容易被旧习惯影响判断".to_string(),
            ],
            current_state: "新书创建阶段，核心欲望已确立。".to_string(),
            relationship_map: vec![CharacterRelationship {
                target: "陈启明".to_string(),
                relationship: "旧日对手".to_string(),
                tension: "林舟掌握信息差，但资源处于劣势。".to_string(),
            }],
            arc: CharacterArc {
                start: "被局势推着走。".to_string(),
                turning_points: vec!["第一次主动反击".to_string(), "承担破局代价".to_string()],
                expected_end: "能主动设计局势并承担代价。".to_string(),
            },
            first_appearance_chapter: 1,
            chapter_1_to_30_plan: vec![
                "前三章确立目标".to_string(),
                "十章内完成第一轮反击".to_string(),
            ],
        },
        CharacterCard {
            id: CharacterId::new(),
            novel_id: novel.id.clone(),
            id_hint: "antagonist_primary".to_string(),
            name: "陈启明".to_string(),
            role: "antagonist".to_string(),
            identity: "掌握局部资源的竞争者".to_string(),
            personality: vec![
                "谨慎".to_string(),
                "现实".to_string(),
                "擅长施压".to_string(),
            ],
            desire: "保住自己既有利益，不允许主角破局。".to_string(),
            motivation: "害怕失去资源位置，也不相信后来者能改写规则。".to_string(),
            secret: "".to_string(),
            abilities: vec!["资源整合".to_string(), "施压谈判".to_string()],
            limitations: vec!["低估主角的信息差".to_string()],
            current_state: "尚未察觉主角的全部底牌。".to_string(),
            relationship_map: vec![CharacterRelationship {
                target: "林舟".to_string(),
                relationship: "竞争对手".to_string(),
                tension: "明面资源占优，但被主角提前布局牵动。".to_string(),
            }],
            arc: CharacterArc {
                start: "轻视主角。".to_string(),
                turning_points: vec!["第一次被主角破局".to_string()],
                expected_end: "被迫正面对抗主角。".to_string(),
            },
            first_appearance_chapter: 1,
            chapter_1_to_30_plan: vec![
                "前三章施压".to_string(),
                "十章内升级为明线对手".to_string(),
            ],
        },
    ]
}

fn draft_outlines_for_indices(
    novel_id: &NovelId,
    idea: &str,
    indices: &[u32],
) -> Vec<ChapterOutline> {
    indices
        .iter()
        .copied()
        .map(|index| ChapterOutline {
            novel_id: novel_id.clone(),
            volume_index: 1,
            chapter_index: index,
            title: format!("第{}章 {}", index, outline_title(index)),
            pov: "第三人称有限视角".to_string(),
            goal: format!("让「{}」的核心期待向前推进一步。", idea),
            conflict: outline_conflict(index),
            key_events: vec![format!("第{}章完成一次明确事件推进。", index)],
            character_changes: vec![format!("主角在第{}章获得新的判断或压力。", index)],
            new_facts: vec![FactTriple {
                subject: "林舟".to_string(),
                predicate: "推进".to_string(),
                object: format!("第{}章阶段目标", index),
                importance: 2,
            }],
            payoff: outline_payoff(index),
            foreshadowing: vec![format!("第{}章埋下一个后续可回收的信息差。", index)],
            cliffhanger: outline_hook(index),
            estimated_word_count: 2500,
        })
        .collect()
}

fn outline_to_text(outline: &ChapterOutline) -> String {
    format!(
        "目标：{}\n冲突：{}\n回报：{}\n章尾钩子：{}\n伏笔：{}",
        outline.goal,
        outline.conflict,
        outline.payoff,
        outline.cliffhanger,
        outline.foreshadowing.join("；")
    )
}

fn infer_title(idea: &str) -> String {
    if idea.contains("重生") {
        "重启逆袭线".to_string()
    } else if idea.contains("玄幻") || idea.contains("仙侠") {
        "长夜登天录".to_string()
    } else {
        "命运改写者".to_string()
    }
}

fn infer_genre(idea: &str) -> String {
    for genre in [
        "都市",
        "重生",
        "玄幻",
        "仙侠",
        "科幻",
        "末世",
        "游戏",
        "无限流",
    ] {
        if idea.contains(genre) {
            return genre.to_string();
        }
    }
    "原创长篇".to_string()
}

fn infer_protagonist_identity(idea: &str) -> &'static str {
    if idea.contains("外卖") {
        "回到十年前的外卖站新人"
    } else if idea.contains("玄幻") {
        "被宗门边缘化的低阶修行者"
    } else {
        "被命运推回起点的普通人"
    }
}

fn platform_reader(platform: TargetPlatform) -> &'static str {
    match platform {
        TargetPlatform::General => "偏好强目标、强节奏和清晰回报的中文网文读者",
        TargetPlatform::Qidian => "偏好体系感、升级线和长期伏笔的起点读者",
        TargetPlatform::Fanqie => "偏好开篇速度、情绪反馈和短周期爽点的番茄读者",
    }
}

fn platform_selling_point(platform: TargetPlatform) -> &'static str {
    match platform {
        TargetPlatform::General => "兼顾长期主线和短周期爽点",
        TargetPlatform::Qidian => "用体系化资源与成长线建立长期追读期待",
        TargetPlatform::Fanqie => "前 3 章提高情绪反馈频率，降低阅读门槛",
    }
}

fn outline_title(index: u32) -> &'static str {
    match index {
        1 => "回到起点",
        2 => "第一道裂缝",
        3 => "旧账新算",
        4..=10 => "局势加速",
        11..=20 => "暗线浮出",
        _ => "反击成形",
    }
}

fn outline_hook(index: u32) -> String {
    match index {
        1 => "主角发现这次重来并不完全等同于记忆中的过去。".to_string(),
        2 => "一个小选择引出原时间线从未出现的阻碍。".to_string(),
        3 => "对手第一次意识到主角不是普通新人。".to_string(),
        _ => format!("第{}章结尾抛出下一章必须处理的新压力。", index),
    }
}

fn outline_conflict(index: u32) -> String {
    if index <= 3 {
        "主角资源不足，却必须立刻做出会影响后续命运的选择。".to_string()
    } else {
        "主角的计划推进时，被对手或环境规则迫使付出代价。".to_string()
    }
}

fn outline_payoff(index: u32) -> String {
    if index % 5 == 0 {
        "回收一个早期伏笔，让主角获得阶段性优势。".to_string()
    } else {
        "给出小胜或关键信息，让读者确认主线正在推进。".to_string()
    }
}
