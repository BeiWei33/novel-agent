# 测试题材样例：女性向逆袭复仇文

## 输入创意

```text
现代女性向逆袭复仇，女主被未婚夫和闺蜜联手夺走公司后，回到签署股权转让协议前一天。
```

## 推荐平台

`fanqie`

## 核心卖点

- 重生节点贴近核心背叛，开篇冲突直接。
- 女主不是单纯打脸，而是重新掌控证据、股权和舆论。
- 情绪价值来自清醒、反制、事业重建和关系筛选。

## 前三章测试目标

1. 第 1 章：女主醒来发现明天就要签股权转让协议，未婚夫已经布好局。
2. 第 2 章：女主假意配合，暗中调取财务异常和聊天记录。
3. 第 3 章：签约现场女主反手抛出第一份证据，让对手阵脚大乱，但真正的幕后资金方露面。

## 关键人物

- 姜晚：女主，前世被背叛，今生冷静反制。
- 陆承泽：未婚夫，擅长情感操控和资本包装。
- 宋晴：闺蜜，嫉妒女主资源，知道部分公司秘密。
- 顾行川：投资人，和女主有利益合作，不急于站队。

## 检验点

- 打脸要有证据链，不要靠吵架取胜。
- 女主行动要主动，不能只等反派露馅。
- 感情线服务信任重建，不抢复仇主线。

## 回归验收 JSON

```json
{
  "fixture_id": "romance_comeback",
  "input": {
    "idea": "现代女性向逆袭复仇，女主被未婚夫和闺蜜联手夺走公司后，回到签署股权转让协议前一天。",
    "target_platform": "fanqie"
  },
  "expected_checks": {
    "market": {
      "min_title_candidates": 3,
      "required_tags": ["重生", "复仇", "逆袭", "女性向"],
      "must_include_selling_points": ["背叛节点", "证据链", "事业反击"]
    },
    "plot": {
      "min_chapter_outlines": 30,
      "chapter_1": {
        "must_have_goal": true,
        "must_have_conflict": true,
        "must_have_cliffhanger": true,
        "required_events": ["回到签约前", "未婚夫布局", "女主决定反制"]
      }
    },
    "character": {
      "required_roles": ["protagonist", "antagonist", "ally"],
      "protagonist_must_have": ["主动取证", "清醒反制", "事业目标"]
    },
    "worldbuilding": {
      "required_world_elements": ["公司股权结构", "证据链来源", "舆论和资金压力"],
      "required_hard_rules": ["复仇必须依赖证据链", "股权和资金操作不能随意跳步"],
      "required_seed_facts": ["姜晚回到签约前一天", "陆承泽已经布好股权局"]
    },
    "writer": {
      "chapter_1_min_word_count": 1800,
      "must_include": ["股权转让危机", "隐藏证据", "情绪反击"],
      "forbidden": ["只靠争吵胜利", "女主被动等待", "感情线压过主线"]
    },
    "continuity": {
      "must_track_facts": ["股权转让时间点", "证据取得方式", "未婚夫和闺蜜的关系"],
      "require_character_state_updates": true,
      "max_high_severity_issues": 0
    },
    "style": {
      "must_preserve": ["女主清醒反制", "证据链逻辑", "情绪爽点"],
      "must_improve": ["压缩解释", "强化签约现场压迫", "提高对白张力"],
      "forbidden": ["只靠争吵打脸", "过早弱化复仇主线"]
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
