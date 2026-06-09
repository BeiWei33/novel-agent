use async_trait::async_trait;
use serde_json::{Value, json};

use super::{ModelClient, ModelMetadata, ModelRequest, ModelResponse, ModelUsage};
use crate::error::ModelError;

#[derive(Debug, Clone)]
pub struct SmokeModelClient {
    model: String,
}

impl SmokeModelClient {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }
}

#[async_trait]
impl ModelClient for SmokeModelClient {
    fn metadata(&self) -> ModelMetadata {
        ModelMetadata::new("smoke", self.model.clone())
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let system = request.system_prompt.as_deref().unwrap_or_default();
        let role = role_for_system(system);
        let structured = match role {
            "market" => market_structured(&request.prompt),
            "plot" => plot_structured(&request.prompt),
            "character" => character_structured(),
            "worldbuilding" => worldbuilding_structured(),
            "writer" => writer_structured(&request.prompt),
            "continuity" => continuity_structured(),
            "style" => style_structured(&request.prompt),
            "reviewer" => reviewer_structured(),
            _ => json!({}),
        };
        let text = json!({
            "role": role,
            "structured": structured,
            "raw_notes": format!("local smoke provider: {}", self.model),
        })
        .to_string();

        Ok(ModelResponse {
            raw: text.clone(),
            usage: Some(estimate_usage(&request, &text)),
            text,
        })
    }
}

fn role_for_system(system: &str) -> &'static str {
    if system.contains("Market Agent Prompt") {
        "market"
    } else if system.contains("Plot Agent Prompt") {
        "plot"
    } else if system.contains("Character Agent Prompt") {
        "character"
    } else if system.contains("Worldbuilding Agent Prompt") {
        "worldbuilding"
    } else if system.contains("Chapter Writer Agent Prompt") {
        "writer"
    } else if system.contains("Continuity Agent Prompt") {
        "continuity"
    } else if system.contains("Style Agent Prompt") {
        "style"
    } else if system.contains("Reviewer Agent Prompt") {
        "reviewer"
    } else {
        "unknown"
    }
}

fn market_structured(prompt: &str) -> Value {
    let platform = target_platform(prompt);
    json!({
        "market_analysis": {
            "genre": genre_hint(prompt),
            "target_readers": platform_readers(platform),
            "core_selling_points": [
                "开篇直接给出命运转折点",
                "主角主动破局形成短周期回报",
                "事实和伏笔可追踪回收"
            ],
            "reader_expectations": [
                "主角行动清晰",
                "冲突持续升级",
                "章尾保留下一步压力"
            ],
            "emotional_hooks": [
                "低谷后的翻盘",
                "掌控感逐步增强"
            ],
            "platform_tags": [platform, "smoke", "mvp"]
        },
        "title_candidates": [
            {
                "title": title_for_prompt(prompt),
                "reason": "本地 smoke provider 生成的稳定演示书名。"
            }
        ],
        "opening_strategy": {
            "first_scene": "主角回到命运转折点，当场发现旧危机提前压来。",
            "first_conflict": "资源不足和对手施压同时出现，主角必须立刻选择。",
            "first_three_chapters_goal": "确立目标、对手和第一轮反击期待。"
        },
        "platform_profile": platform_profile(platform)
    })
}

fn plot_structured(prompt: &str) -> Value {
    let chapters = target_chapters(prompt);
    let start = chapter_start(prompt);
    let end = chapter_end(prompt).unwrap_or_else(|| start + chapters - 1);
    let outlines = (start..=end)
        .map(|index| {
            json!({
                "volume_index": 1,
                "chapter_index": index,
                "title": format!("第{}章 {}", index, outline_title(index)),
                "pov": "第三人称有限视角",
                "goal": format!("第{}章推进主角的第一轮主动破局。", index),
                "conflict": "主角资源不足，但对手和环境规则同时施压。",
                "key_events": [
                    format!("第{}章完成一次明确事件推进。", index),
                    "主角用信息差换取主动权"
                ],
                "character_changes": [
                    "主角从被动确认局势转为主动试探规则"
                ],
                "new_facts": [
                    fact("林舟", "推进", format!("第{}章阶段目标", index), 2)
                ],
                "foreshadowing": [
                    "一个不符合记忆的细节被记录下来"
                ],
                "payoff": "给出一次小胜或关键信息回报。",
                "cliffhanger": "章尾出现新的压力点，迫使主角继续行动。",
                "estimated_word_count": 2500
            })
        })
        .collect::<Vec<_>>();

    json!({
        "plot_plan": {
            "main_conflict": "主角想改写命运，但资源、信息差和对手持续制造压力。",
            "protagonist_goal": "抓住第二次机会，建立自己的主动权。",
            "antagonistic_force": "掌握局部资源的竞争者和旧规则。",
            "long_term_hook": "原时间线没有出现的异常正在改变后续局势。"
        },
        "chapter_outlines": outlines
    })
}

fn character_structured() -> Value {
    json!({
        "characters": [
            {
                "id_hint": "protagonist",
                "name": "林舟",
                "role": "protagonist",
                "identity": "回到命运转折点的普通人",
                "personality": ["冷静", "抗压", "行动力强"],
                "desire": "抓住第二次机会，建立自己的主动权。",
                "motivation": "不再让重要机会从手里流失。",
                "secret": "掌握一段只有自己知道的未来信息。",
                "abilities": ["复盘局势", "快速执行"],
                "limitations": ["前期资源不足", "旧习惯会影响判断"],
                "current_state": "刚确认重来，目标已经确立。",
                "relationship_map": [
                    {
                        "target": "陈启明",
                        "relationship": "竞争对手",
                        "tension": "陈启明资源占优，但低估了林舟的信息差。"
                    }
                ],
                "arc": {
                    "start": "被局势推着走。",
                    "turning_points": ["第一次主动反击", "承担破局代价"],
                    "expected_end": "能主动设计局势并承担代价。"
                },
                "first_appearance_chapter": 1,
                "chapter_1_to_30_plan": ["前三章确立目标", "十章内完成第一轮反击"]
            },
            {
                "id_hint": "antagonist_primary",
                "name": "陈启明",
                "role": "antagonist",
                "identity": "掌握局部资源的竞争者",
                "personality": ["谨慎", "现实", "擅长施压"],
                "desire": "保住既有利益，不允许主角破局。",
                "motivation": "害怕失去资源位置，也不相信后来者能改写规则。",
                "secret": "",
                "abilities": ["资源整合", "施压谈判"],
                "limitations": ["低估主角的信息差"],
                "current_state": "尚未察觉主角的全部底牌。",
                "relationship_map": [
                    {
                        "target": "林舟",
                        "relationship": "竞争对手",
                        "tension": "明面资源占优，但被主角提前布局牵动。"
                    }
                ],
                "arc": {
                    "start": "轻视主角。",
                    "turning_points": ["第一次被主角破局"],
                    "expected_end": "被迫正面对抗主角。"
                },
                "first_appearance_chapter": 1,
                "chapter_1_to_30_plan": ["前三章施压", "十章内升级为明线对手"]
            }
        ]
    })
}

fn worldbuilding_structured() -> Value {
    json!({
        "world_setting": {
            "overview": "普通城市和旧时间线信息差共同构成本地 smoke 演示世界。",
            "core_rules": [
                "未来信息只能帮助主角提前选择，不能直接替代行动。",
                "每次破局都会改变部分后续事件。"
            ],
            "power_or_resource_system": [
                "信息差",
                "现金流",
                "人脉信任"
            ],
            "costs_and_limits": [
                "前期资源不足",
                "过度暴露会引来对手警惕"
            ],
            "hard_rules": [
                "事实变化必须进入事实表",
                "章尾压力必须和主线相关"
            ]
        },
        "facts_to_seed": [
            fact("林舟", "拥有", "旧时间线记忆", 3),
            fact("陈启明", "掌握", "局部资源优势", 2)
        ]
    })
}

fn writer_structured(prompt: &str) -> Value {
    let rewritten = prompt.contains("rewrite_chapter") || prompt.contains("previous_draft");
    let title = if rewritten {
        "第1章 回到起点·重写"
    } else {
        "第1章 回到起点"
    };
    let content = if rewritten {
        "林舟重新站在旧日路口时，先把上一版里迟疑的选择压了下去。\n他不再解释自己为什么知道答案，只用一个更直接的动作逼对手露出破绽。\n陈启明以为他仍会退让，却发现林舟已经把下一步压力推回了桌面。\n章尾，原本不该出现的电话提前响起，来电人正是旧时间线的关键变量。"
    } else {
        "林舟站在旧日路口，确认自己真的回到了命运转折之前。\n他没有急着证明什么，而是先把眼前能抓住的资源一项项列出来。\n陈启明的压力很快落下，熟悉的规则再次试图把他推回原位。\n这一次，林舟选择先退半步，换一个所有人都看不懂的入口。\n章尾，原本应该三天后才出现的人，提前出现在门外。"
    };

    json!({
        "chapter_draft": {
            "volume_index": 1,
            "chapter_index": 1,
            "title": title,
            "content": content,
            "summary": "林舟确认重来并做出第一步主动选择。",
            "word_count": content.chars().filter(|ch| !ch.is_whitespace()).count(),
            "pov": "第三人称有限视角",
            "key_events": [
                "林舟确认回到命运转折点",
                "陈启明第一次施压",
                "章尾出现异常变量"
            ],
            "new_facts": [
                fact("林舟", "确认", "自己回到命运转折点", 3)
            ],
            "foreshadowing": [
                {
                    "seed": "提前出现的人",
                    "status": "planted",
                    "expected_payoff": "下一章解释此人为何提前出现。"
                }
            ],
            "continuity_notes": [
                "本地 smoke provider 生成，保持事实表可追踪。"
            ]
        }
    })
}

fn continuity_structured() -> Value {
    json!({
        "continuity_report": {
            "passed": true,
            "issues": [],
            "new_facts": [
                fact("林舟", "确认", "自己回到命运转折点", 3)
            ],
            "character_state_updates": [
                {
                    "character": "林舟",
                    "state": "从确认重来转为主动试探规则"
                }
            ],
            "foreshadowing_updates": [
                {
                    "seed": "提前出现的人",
                    "status": "planted",
                    "note": "下一章需要解释提前出现的原因。"
                }
            ],
            "raw_notes": "连续性通过。"
        }
    })
}

fn style_structured(prompt: &str) -> Value {
    let rewritten = prompt.contains("重写") || prompt.contains("previous_draft");
    let title = if rewritten {
        "第1章 回到起点·定稿"
    } else {
        "第1章 回到起点"
    };
    let content = if rewritten {
        "林舟重新站在旧日路口时，雨声像一根线，把十年前的夜晚重新缝回眼前。\n他没有再给自己找理由，只把能动用的资源写成三行，随后径直拨通了那个本不该现在出现的号码。\n陈启明还在等他退让，却先等来了局势反转的第一张牌。\n章尾，电话那端沉默片刻，说出了一个林舟记忆里从未提前出现的名字。"
    } else {
        "林舟站在旧日路口时，雨水正顺着外卖站的卷帘门往下淌。\n他确认自己回到了十年前，没有急着证明奇迹，只先把能抓住的资源一项项列出来。\n陈启明的压力很快落下，熟悉的规则再次试图把他推回原位。\n这一次，林舟退半步，却换了一个所有人都看不懂的入口。\n章尾，原本应该三天后才出现的人，提前站在了门外。"
    };

    json!({
        "styled_chapter": {
            "title": title,
            "content": content,
            "summary": "林舟确认重来，利用信息差完成第一步主动试探。",
            "style_notes": [
                "压缩解释，保留主角行动和章尾异常。"
            ]
        }
    })
}

fn reviewer_structured() -> Value {
    json!({
        "review_report": {
            "total_score": 82,
            "passed": true,
            "scores": {
                "opening_hook_score": 8,
                "pacing_score": 8,
                "payoff_score": 8,
                "character_score": 8,
                "dialogue_score": 7,
                "continuity_score": 8,
                "cliffhanger_score": 8,
                "platform_fit_score": 8
            },
            "strengths": [
                "主角目标清晰",
                "章尾异常能推动下一章"
            ],
            "issues": [],
            "suggestions": [
                "下一章优先解释提前出现的人，并把压力继续升级。"
            ],
            "rewrite_instruction": {
                "needed": false,
                "rewrite_type": "none",
                "priority": "low",
                "goals": [],
                "preserve": [],
                "change": [],
                "avoid": []
            }
        }
    })
}

fn target_platform(prompt: &str) -> &'static str {
    if prompt.contains("\"qidian\"") {
        "qidian"
    } else if prompt.contains("\"fanqie\"") {
        "fanqie"
    } else {
        "general"
    }
}

fn platform_profile(platform: &str) -> Value {
    match platform {
        "qidian" => json!({
            "target_platform": "qidian",
            "opening_speed": "layered",
            "setup_ratio": 0.35,
            "dialogue_ratio": 0.30,
            "payoff_frequency": "every_2_chapters",
            "cliffhanger_strength": "medium",
            "review_bias": {
                "continuity_score": 2,
                "platform_fit_score": 1
            }
        }),
        "fanqie" => json!({
            "target_platform": "fanqie",
            "opening_speed": "fast",
            "setup_ratio": 0.18,
            "dialogue_ratio": 0.42,
            "payoff_frequency": "every_chapter",
            "cliffhanger_strength": "high",
            "review_bias": {
                "opening_hook_score": 2,
                "pacing_score": 2,
                "payoff_score": 1,
                "cliffhanger_score": 1
            }
        }),
        _ => json!({
            "target_platform": "general",
            "opening_speed": "balanced",
            "setup_ratio": 0.25,
            "dialogue_ratio": 0.35,
            "payoff_frequency": "every_chapter",
            "cliffhanger_strength": "medium",
            "review_bias": {
                "platform_fit_score": 1
            }
        }),
    }
}

fn genre_hint(prompt: &str) -> &'static str {
    if prompt.contains("玄幻") || prompt.contains("修行") {
        "玄幻"
    } else if prompt.contains("女性向") || prompt.contains("复仇") {
        "现代言情"
    } else if prompt.contains("都市") || prompt.contains("外卖") {
        "都市"
    } else {
        "原创长篇"
    }
}

fn title_for_prompt(prompt: &str) -> &'static str {
    if prompt.contains("玄幻") || prompt.contains("古塔") {
        "因果塔债"
    } else if prompt.contains("复仇") || prompt.contains("股权") {
        "签约前夜"
    } else if prompt.contains("外卖") {
        "重回外卖站"
    } else {
        "命运改写者"
    }
}

fn platform_readers(platform: &str) -> &'static str {
    match platform {
        "qidian" => "偏好体系感、升级线和长期伏笔的起点读者",
        "fanqie" => "偏好开篇速度、情绪反馈和短周期爽点的番茄读者",
        _ => "偏好强目标、强节奏和清晰回报的中文网文读者",
    }
}

fn target_chapters(prompt: &str) -> u32 {
    numeric_payload_field(prompt, "target_chapters")
        .filter(|value| *value > 0)
        .unwrap_or(30)
}

fn chapter_start(prompt: &str) -> u32 {
    numeric_payload_field(prompt, "chapter_start")
        .filter(|value| *value > 0)
        .unwrap_or(1)
}

fn chapter_end(prompt: &str) -> Option<u32> {
    numeric_payload_field(prompt, "chapter_end").filter(|value| *value > 0)
}

fn numeric_payload_field(prompt: &str, field: &str) -> Option<u32> {
    let marker = format!("\"{field}\":");
    let Some(start) = prompt.find(&marker).map(|index| index + marker.len()) else {
        return None;
    };
    let digits = prompt[start..]
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse::<u32>().ok()
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

fn fact(
    subject: impl Into<String>,
    predicate: impl Into<String>,
    object: impl Into<String>,
    importance: i32,
) -> Value {
    json!({
        "subject": subject.into(),
        "predicate": predicate.into(),
        "object": object.into(),
        "importance": importance
    })
}

fn estimate_usage(request: &ModelRequest, text: &str) -> ModelUsage {
    let system_chars = request
        .system_prompt
        .as_ref()
        .map(|value| value.chars().count())
        .unwrap_or_default();
    let prompt_tokens = rough_tokens(system_chars + request.prompt.chars().count());
    let completion_tokens = rough_tokens(text.chars().count());

    ModelUsage {
        prompt_tokens: Some(prompt_tokens),
        completion_tokens: Some(completion_tokens),
        total_tokens: Some(prompt_tokens.saturating_add(completion_tokens)),
    }
}

fn rough_tokens(chars: usize) -> u32 {
    ((chars / 2).max(1)).min(u32::MAX as usize) as u32
}
