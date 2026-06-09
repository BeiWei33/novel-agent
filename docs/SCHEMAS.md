# novel-agent 业务输出 Schema

本文档由开发者 B 维护，是 MVP 阶段 Prompt 输出和业务结构的权威来源。工程侧可以把 `structured` 内部实体映射为 Rust domain model 或 JSON 解析结构。

## 通用约定

- 所有 Agent 只输出一个 JSON object，不在 JSON 外追加解释。
- Prompt 输出统一使用 `AgentOutputEnvelope`。
- `raw_text` 是工程侧保存的完整模型原始响应，不由模型生成。
- `parse_error` 只由工程侧生成，不由模型生成。
- `raw_notes` 是模型在合法 JSON 内写的补充说明；没有补充时写空字符串。
- 可写空字符串或空数组，但不要省略字段。
- `importance` 使用 1 到 5，数字越大越重要。
- `target_platform` 使用 `qidian | fanqie | general`。
- `chapter_index` 从 1 开始。

## AgentOutputEnvelope

Prompt 文件展示的是 Agent 顶层输出。每个 Agent 必须使用同一个 envelope：

```json
{
  "role": "market | plot | character | worldbuilding | writer | continuity | style | reviewer",
  "structured": {},
  "raw_notes": ""
}
```

工程侧保存模型结果时，可以在此基础上追加工程字段：

```json
{
  "role": "writer",
  "structured": {
    "chapter_draft": {}
  },
  "raw_notes": "",
  "raw_text": "完整原始响应，由工程侧保存",
  "parse_error": null
}
```

## AgentInputEnvelope

工程侧调用 Prompt Agent 时使用统一输入 envelope。Prompt 文件中的“输入 payload”只描述 `payload` 内部业务字段。

```json
{
  "task": "create_novel | generate_outline | generate_chapter | review_chapter | rewrite_chapter | extract_facts | polish_style | check_continuity",
  "instructions": "本次调用的格式要求或重试要求",
  "payload": {},
  "context": [
    {
      "kind": "memory | facts | chapter | character | world | review | other",
      "title": "上下文标题",
      "content": "上下文内容"
    }
  ]
}
```

当前工程实现会把 `task`、`instructions`、`payload` 和 `context` 组装进模型用户消息。所有 Prompt 的输入要求都应按 `payload` 字段理解。

## FactTriple

事实三元组是唯一事实结构。`ChapterOutline.new_facts` 表示本章计划产生的事实；`ChapterDraft.new_facts` 和 `ContinuityReport.new_facts` 表示正文实际产生或复核确认的事实。

```json
{
  "subject": "主体",
  "predicate": "关系",
  "object": "客体",
  "importance": 1
}
```

## Foreshadowing

```json
{
  "seed": "伏笔",
  "status": "planted | advanced | paid_off | contradicted",
  "expected_payoff": "预计回收方式"
}
```

## PlatformProfile

平台策略需要落成可执行参数，供 Plot、Writer、Reviewer 使用。`platform_profile` 是新书创建后的长期策略配置，权威持久化位置是 `NovelBible.platform_profile`；Market Agent 负责生成初稿，后续 workflow 从 `NovelBible` 读取并传给 Writer/Reviewer。

```json
{
  "target_platform": "qidian | fanqie | general",
  "opening_speed": "fast | balanced | layered",
  "setup_ratio": 0.25,
  "dialogue_ratio": 0.35,
  "payoff_frequency": "every_chapter | every_2_chapters | every_arc",
  "cliffhanger_strength": "medium | high",
  "review_bias": {
    "opening_hook_score": 1,
    "pacing_score": 1,
    "payoff_score": 1,
    "continuity_score": 1,
    "platform_fit_score": 1
  }
}
```

建议默认：

```json
{
  "qidian": {
    "opening_speed": "layered",
    "setup_ratio": 0.35,
    "dialogue_ratio": 0.3,
    "payoff_frequency": "every_2_chapters",
    "cliffhanger_strength": "medium",
    "review_bias": {
      "continuity_score": 2,
      "platform_fit_score": 1
    }
  },
  "fanqie": {
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
  },
  "general": {
    "opening_speed": "balanced",
    "setup_ratio": 0.25,
    "dialogue_ratio": 0.35,
    "payoff_frequency": "every_chapter",
    "cliffhanger_strength": "medium",
    "review_bias": {
      "platform_fit_score": 1
    }
  }
}
```

## MarketOutput

```json
{
  "role": "market",
  "structured": {
    "market_analysis": {
      "target_platform": "qidian | fanqie | general",
      "genre": "题材",
      "sub_genres": ["子题材"],
      "target_readers": "目标读者画像",
      "reader_expectations": ["读者想获得的体验"],
      "core_selling_points": ["核心卖点"],
      "emotional_hooks": ["情绪钩子"],
      "platform_tags": ["平台标签"],
      "risk_notes": ["风险提示"]
    },
    "title_candidates": [
      {
        "title": "书名",
        "reason": "为什么适合"
      }
    ],
    "intro_candidates": [
      {
        "intro": "100 到 200 字简介",
        "angle": "卖点角度"
      }
    ],
    "opening_strategy": {
      "first_scene": "第一场戏建议",
      "first_conflict": "第一冲突",
      "first_three_chapters_goal": "前三章要完成的读者承诺",
      "avoid": ["开篇应避免的内容"]
    },
    "platform_profile": {},
    "handoff": {
      "plot_focus": ["交给 Plot Agent 的重点"],
      "character_focus": ["交给 Character Agent 的重点"],
      "worldbuilding_focus": ["交给 Worldbuilding Agent 的重点"]
    }
  },
  "raw_notes": ""
}
```

## NovelBible

```json
{
  "novel_bible": {
    "title_candidates": [
      {
        "title": "书名",
        "reason": "推荐理由"
      }
    ],
    "premise": "一句话卖点",
    "genre": "题材",
    "target_platform": "qidian | fanqie | general",
    "target_readers": "目标读者画像",
    "core_selling_points": ["核心卖点"],
    "reader_expectations": ["读者期待"],
    "main_conflict": "全书主冲突",
    "protagonist_goal": "主角长期目标",
    "emotional_value": "情绪价值",
    "tone": "文风和叙事口吻",
    "platform_tags": ["平台标签"],
    "world_rules": ["世界规则"],
    "constraints": ["禁写项或硬限制"],
    "opening_strategy": {
      "first_scene": "第一场戏",
      "first_conflict": "第一冲突",
      "first_three_chapters_goal": "前三章目标"
    },
    "platform_profile": {}
  }
}
```

## PlotOutput

Plot Agent 的输出结构保持不变。工程侧为了避免真实模型一次输出过长，会在输入 payload 中传入可选批次字段：

- `target_chapters`：本批次需要输出的章节数。
- `total_chapters`：本次大纲目标总章节数。
- `chapter_start` / `chapter_end`：本批次闭区间，全书绝对章号。
- `existing_plot_plan`：前序批次已经确认的主线规划，可为空对象。
- `previous_chapter_outlines`：最近几章摘要，用于保持批次衔接。
- `batch_policy.keep_absolute_chapter_index`：为 true 时，`chapter_index` 必须使用全书绝对章号。

```json
{
  "role": "plot",
  "structured": {
    "plot_plan": {
      "main_conflict": "全书主冲突",
      "protagonist_goal": "主角长期目标",
      "antagonistic_force": "主要阻力或反派势力",
      "long_term_hook": "支撑百万字连载的长期期待",
      "volume_plan": [
        {
          "volume_index": 1,
          "title": "卷名",
          "goal": "本卷目标",
          "core_conflict": "本卷冲突",
          "payoff": "本卷回报"
        }
      ],
      "foreshadowing": [
        {
          "seed": "伏笔",
          "planted_in_chapter": 1,
          "expected_payoff": "预计回收方式"
        }
      ]
    },
    "chapter_outlines": [],
    "risk_notes": ["剧情风险"]
  },
  "raw_notes": ""
}
```

## CharacterOutput

Character Agent 输入 payload 支持可选 `scope` 字段，用于控制真实模型输出规模：

- `scope.focus_chapters`：人物计划优先覆盖的前期章节范围。
- `scope.max_characters`：建议输出的人物数量上限。
- `scope.max_relationships_per_character`：单个人物关系数量上限。
- `scope.max_turning_points_per_character`：单个人物成长转折数量上限。
- `scope.max_plan_items_per_character`：单个人物章节计划条数上限。

```json
{
  "role": "character",
  "structured": {
    "characters": [],
    "relationship_overview": "主要人物关系总览",
    "consistency_rules": ["后续写作不能违背的人物规则"],
    "risk_notes": ["人物塑造风险"]
  },
  "raw_notes": ""
}
```

## CharacterCard

```json
{
  "id_hint": "protagonist",
  "name": "姓名",
  "role": "protagonist | antagonist | ally | rival | mentor | love_interest | supporting",
  "identity": "身份",
  "personality": ["性格关键词"],
  "desire": "表层欲望",
  "motivation": "深层动机",
  "secret": "秘密或隐藏信息",
  "abilities": ["能力、资源或优势"],
  "limitations": ["弱点、限制或代价"],
  "current_state": "当前状态",
  "relationship_map": [
    {
      "target": "角色名",
      "relationship": "关系",
      "tension": "张力"
    }
  ],
  "arc": {
    "start": "初始状态",
    "turning_points": ["关键转折"],
    "expected_end": "阶段终点"
  },
  "first_appearance_chapter": 1,
  "chapter_1_to_30_plan": ["前 30 章中的作用"]
}
```

## WorldbuildingOutput

Worldbuilding Agent 的输出结构保持不变。工程侧为了控制真实模型输出规模，会在输入 payload 中传入可选 `scope` 字段：

- `scope.focus_chapters`：本次世界观优先服务的前期章节范围。
- `scope.max_organizations`：建议输出的组织数量上限。
- `scope.max_locations`：建议输出的地点数量上限。
- `scope.max_facts_to_seed`：建议写入事实种子的数量上限。

```json
{
  "role": "worldbuilding",
  "structured": {
    "world_setting": {},
    "facts_to_seed": [],
    "risk_notes": ["设定风险"]
  },
  "raw_notes": ""
}
```

## WorldSetting

```json
{
  "genre_type": "都市 | 玄幻 | 仙侠 | 科幻 | 末世 | 游戏 | 无限流 | 其他",
  "overview": "世界观总述",
  "power_system": {
    "name": "体系名称",
    "levels": ["等级或阶段"],
    "rules": ["规则"],
    "costs": ["代价"],
    "limits": ["限制"]
  },
  "organizations": [
    {
      "name": "组织名",
      "role": "剧情功能",
      "resources": ["资源"],
      "conflicts": ["冲突"]
    }
  ],
  "locations": [
    {
      "name": "地点名",
      "description": "描述",
      "story_use": "剧情用途"
    }
  ],
  "taboos": ["禁忌"],
  "hard_rules": ["硬规则"]
}
```

## ChapterOutline

```json
{
  "volume_index": 1,
  "chapter_index": 1,
  "title": "章节标题",
  "pov": "叙事视角",
  "goal": "本章目标",
  "conflict": "本章冲突",
  "key_events": ["关键事件"],
  "character_changes": ["人物变化"],
  "new_facts": [
    {
      "subject": "主体",
      "predicate": "关系",
      "object": "客体",
      "importance": 1
    }
  ],
  "foreshadowing": ["伏笔"],
  "payoff": "本章回报",
  "cliffhanger": "章尾钩子",
  "estimated_word_count": 2500
}
```

## WriterOutput

```json
{
  "role": "writer",
  "structured": {
    "chapter_draft": {}
  },
  "raw_notes": ""
}
```

## ChapterDraft

```json
{
  "volume_index": 1,
  "chapter_index": 1,
  "title": "章节标题",
  "content": "章节正文",
  "summary": "章节摘要",
  "word_count": 2500,
  "pov": "叙事视角",
  "key_events": ["关键事件"],
  "new_facts": [
    {
      "subject": "主体",
      "predicate": "关系",
      "object": "客体",
      "importance": 1
    }
  ],
  "foreshadowing": [
    {
      "seed": "伏笔",
      "status": "planted | advanced | paid_off",
      "expected_payoff": "预计回收方式"
    }
  ],
  "continuity_notes": ["连续性备注"]
}
```

## ContinuityOutput

```json
{
  "role": "continuity",
  "structured": {
    "continuity_report": {}
  },
  "raw_notes": ""
}
```

## ContinuityReport

```json
{
  "passed": true,
  "issues": [
    {
      "severity": "low | medium | high",
      "type": "character | world | timeline | fact | foreshadowing | other",
      "location": "位置",
      "description": "问题说明",
      "suggestion": "修复建议"
    }
  ],
  "new_facts": [
    {
      "subject": "主体",
      "predicate": "关系",
      "object": "客体",
      "importance": 1
    }
  ],
  "character_state_updates": [
    {
      "character": "角色名",
      "before": "原状态",
      "after": "新状态",
      "reason": "变化原因"
    }
  ],
  "foreshadowing_updates": [
    {
      "seed": "伏笔",
      "status": "planted | advanced | paid_off | contradicted",
      "note": "说明"
    }
  ]
}
```

## StyleOutput

```json
{
  "role": "style",
  "structured": {
    "styled_chapter": {}
  },
  "raw_notes": ""
}
```

## ReviewerOutput

```json
{
  "role": "reviewer",
  "structured": {
    "review_report": {}
  },
  "raw_notes": ""
}
```

## ReviewReport

```json
{
  "total_score": 75,
  "passed": true,
  "scores": {
    "opening_hook_score": 8,
    "pacing_score": 8,
    "payoff_score": 8,
    "character_score": 8,
    "dialogue_score": 8,
    "continuity_score": 8,
    "cliffhanger_score": 8,
    "platform_fit_score": 8
  },
  "strengths": ["优点"],
  "issues": [
    {
      "severity": "low | medium | high",
      "dimension": "opening_hook | pacing | payoff | character | dialogue | continuity | cliffhanger | platform_fit",
      "location": "位置",
      "description": "问题说明"
    }
  ],
  "suggestions": ["修改建议"],
  "rewrite_instruction": {
    "needed": false,
    "rewrite_type": "none | partial | full",
    "priority": "low | medium | high",
    "goals": ["返工目标"],
    "preserve": ["保留内容"],
    "change": ["修改内容"],
    "avoid": ["避免内容"]
  }
}
```

## RewriteInstruction

`ReviewReport.rewrite_instruction` 使用以下结构。MVP 阶段不额外增加外层 `trigger` 和 `acceptance_criteria`，避免和当前 Rust domain model 分叉。

```json
{
  "needed": true,
  "rewrite_type": "partial | full",
  "priority": "low | medium | high",
  "goals": ["返工目标"],
  "preserve": ["必须保留的内容"],
  "change": ["必须修改的内容"],
  "avoid": ["避免内容"]
}
```
