# 测试题材样例：玄幻升级文

## 输入创意

```text
玄幻升级文，边城少年继承一座会记录因果债的古塔，每还清一笔因果就能解锁一层能力。
```

## 推荐平台

`qidian`

## 核心卖点

- 古塔不是单纯金手指，而是因果债系统，能力获得需要代价和选择。
- 每个副本都可以围绕“欠债、偿还、反噬、回报”形成闭环。
- 长线悬念是古塔真正主人和主角家族灭门真相。

## 前三章测试目标

1. 第 1 章：边城被妖潮围困，主角为救妹妹触发古塔，背上第一笔因果债。
2. 第 2 章：主角获得短暂能力，却发现使用能力会加重债印。
3. 第 3 章：主角还清一名死去守军的遗愿，解锁第一层能力，同时被城中宗门盯上。

## 关键人物

- 沈砚：主角，谨慎、重情、有底线。
- 沈青禾：妹妹，体内藏有古塔碎片线索。
- 裴照夜：宗门外门执事，表面招揽，实则试探。
- 赤鳞妖王：第一卷外部压力来源。

## 检验点

- 升级必须有规则、代价和阶段目标。
- 战斗不能只写招式名，要有策略和局势变化。
- 每次能力突破都要带出更大的因果问题。

## 回归验收 JSON

```json
{
  "fixture_id": "fantasy_upgrade",
  "input": {
    "idea": "玄幻升级文，边城少年继承一座会记录因果债的古塔，每还清一笔因果就能解锁一层能力。",
    "target_platform": "qidian"
  },
  "expected_checks": {
    "market": {
      "min_title_candidates": 3,
      "required_tags": ["玄幻", "升级", "因果", "古塔"],
      "must_include_selling_points": ["能力代价", "长期伏笔", "阶段闭环"]
    },
    "plot": {
      "min_chapter_outlines": 30,
      "chapter_1": {
        "must_have_goal": true,
        "must_have_conflict": true,
        "must_have_cliffhanger": true,
        "required_events": ["妖潮", "救妹妹", "触发古塔", "第一笔因果债"]
      },
      "must_include_long_term_hook": "古塔真正主人或家族灭门真相"
    },
    "character": {
      "required_roles": ["protagonist", "antagonist", "supporting"],
      "protagonist_must_have": ["底线", "救亲人动机", "能力限制"]
    },
    "worldbuilding": {
      "required_world_elements": ["因果债规则", "古塔层级", "边城妖潮压力"],
      "required_hard_rules": ["借力必须偿还因果", "能力突破必须有代价"],
      "required_seed_facts": ["古塔记录因果债", "沈砚第一次借力会留下债印"]
    },
    "writer": {
      "chapter_1_min_word_count": 2000,
      "must_include": ["能力代价", "战斗策略", "更大因果问题"],
      "forbidden": ["无代价突破", "只报招式名", "规则随意变化"]
    },
    "continuity": {
      "must_track_facts": ["第一笔因果债", "古塔借力代价", "妹妹状态"],
      "require_character_state_updates": true,
      "max_high_severity_issues": 0
    },
    "cross_artifact_consistency": {
      "must_match": ["主角姓名", "因果债状态", "妹妹状态"],
      "critical_fields": ["protagonist_name", "debt_state", "family_status"],
      "max_high_severity_issues": 0
    },
    "style": {
      "must_preserve": ["因果债规则", "战斗局势变化", "长期伏笔"],
      "must_improve": ["减少设定堆叠", "强化战斗画面", "突出能力代价"],
      "forbidden": ["无代价升级", "空泛玄学解释"]
    },
    "review": {
      "pass_line": {
        "total_score": 75,
        "pacing_score": 7,
        "continuity_score": 8,
        "cliffhanger_score": 7
      }
    }
  }
}
```
