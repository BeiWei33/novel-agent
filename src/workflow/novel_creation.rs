use crate::domain::{
    Chapter, ChapterOutline, CharacterArc, CharacterCard, CharacterId, CharacterRelationship,
    FactTriple, Novel, NovelBible, NovelId, OpeningStrategy, TargetPlatform, TitleCandidate,
};
use crate::error::WorkflowError;
use crate::storage::SqliteStorage;

pub struct NovelCreationWorkflow<'a> {
    storage: &'a SqliteStorage,
}

#[derive(Debug, Clone)]
pub struct NovelCreationResult {
    pub novel: Novel,
    pub bible: NovelBible,
    pub characters: Vec<CharacterCard>,
    pub outlines: Vec<ChapterOutline>,
}

impl<'a> NovelCreationWorkflow<'a> {
    pub fn new(storage: &'a SqliteStorage) -> Self {
        Self { storage }
    }

    pub async fn create_from_idea(
        &self,
        idea: &str,
        platform: TargetPlatform,
    ) -> Result<NovelCreationResult, WorkflowError> {
        let novel = Novel::draft(infer_title(idea), infer_genre(idea), platform);
        let bible = draft_bible(&novel, idea);
        let characters = draft_characters(&novel, idea);
        let outlines = draft_outlines(&novel.id, idea, 30);

        self.storage.novels().insert(&novel).await?;
        self.storage.novels().save_bible(&bible).await?;

        for character in &characters {
            self.storage.characters().insert(character).await?;
        }

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
        })
    }

    pub async fn generate_outline(
        &self,
        novel_id: &NovelId,
        chapters: u32,
    ) -> Result<Vec<ChapterOutline>, WorkflowError> {
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
        let outlines = draft_outlines(&novel.id, &idea, chapters);

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
            personality: vec!["冷静".to_string(), "抗压".to_string(), "行动力强".to_string()],
            desire: "抓住第二次机会，建立自己的主动权。".to_string(),
            motivation: "不再让重要的人和机会从手里流失。".to_string(),
            secret: "掌握一段只有自己知道的未来信息。".to_string(),
            abilities: vec!["复盘局势".to_string(), "快速执行".to_string()],
            limitations: vec!["前期资源不足".to_string(), "容易被旧习惯影响判断".to_string()],
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
            chapter_1_to_30_plan: vec!["前三章确立目标".to_string(), "十章内完成第一轮反击".to_string()],
        },
        CharacterCard {
            id: CharacterId::new(),
            novel_id: novel.id.clone(),
            id_hint: "antagonist_primary".to_string(),
            name: "陈启明".to_string(),
            role: "antagonist".to_string(),
            identity: "掌握局部资源的竞争者".to_string(),
            personality: vec!["谨慎".to_string(), "现实".to_string(), "擅长施压".to_string()],
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
            chapter_1_to_30_plan: vec!["前三章施压".to_string(), "十章内升级为明线对手".to_string()],
        },
    ]
}

fn draft_outlines(novel_id: &NovelId, idea: &str, chapters: u32) -> Vec<ChapterOutline> {
    (1..=chapters)
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
    for genre in ["都市", "重生", "玄幻", "仙侠", "科幻", "末世", "游戏", "无限流"] {
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
