# novel-agent 业务输出 Schema

本文档由开发者 B 维护，用于约束 Agent 输出格式。工程侧可以把这些结构映射为 Rust domain model 或 JSON 解析结构。

## 通用约定

- 所有 Agent 输出顶层必须是 JSON object。
- Agent 原始补充信息统一放入 `raw_notes`。
- 可写空字符串或空数组，但不要省略字段。
- `importance` 使用 1 到 5，数字越大越重要。
- `target_platform` 使用 `qidian | fanqie | general`。

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
    }
  }
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
  "new_facts": ["新增事实"],
  "foreshadowing": ["伏笔"],
  "payoff": "本章回报",
  "cliffhanger": "章尾钩子",
  "estimated_word_count": 2500
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
  "word_count_estimate": 2500,
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

```json
{
  "needed": true,
  "rewrite_type": "partial | full",
  "priority": "low | medium | high",
  "trigger": "low_score | continuity_failure | weak_cliffhanger | user_request | other",
  "target_sections": ["需要返工的段落或事件"],
  "goals": ["返工目标"],
  "preserve": ["必须保留的内容"],
  "change": ["必须修改的内容"],
  "avoid": ["避免内容"],
  "acceptance_criteria": ["验收标准"]
}
```

