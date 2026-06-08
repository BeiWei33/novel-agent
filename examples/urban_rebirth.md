# 测试题材样例：都市重生商业文

## 输入创意

```text
都市重生商业文，主角回到十年前，从外卖站开始逆袭。
```

## 推荐平台

`fanqie` 或 `general`

## 核心卖点

- 主角知道未来十年本地生活、短视频、即时配送和社区团购的爆发节点。
- 起点低，从外卖站小人物切入，读者容易代入。
- 商战爽点可以和底层逆袭、家庭遗憾修复结合。

## 前三章测试目标

1. 第 1 章：主角重生到被站长甩锅、即将背债的夜晚，利用未来经验避开事故。
2. 第 2 章：主角抓住暴雨配送危机，反向证明调度方案，第一次获得团队信任。
3. 第 3 章：主角发现未来巨头的本地试点机会，但前世仇人也盯上同一条线。

## 关键人物

- 林舟：重生主角，前世创业失败，懂本地生活行业趋势。
- 许蔓：前世错过的运营高手，当前还是便利店店长。
- 周启明：站点承包商，短视但有资源。
- 陈岳：未来竞争对手，擅长资本包装。

## 检验点

- 开篇是否足够快，不要长篇解释前世。
- 商业决策必须具体，不能靠“未来信息”无脑碾压。
- 主角每次获利都要伴随新的风险或对手注意。

## 回归验收 JSON

```json
{
  "fixture_id": "urban_rebirth",
  "input": {
    "idea": "都市重生商业文，主角回到十年前，从外卖站开始逆袭。",
    "target_platform": "fanqie"
  },
  "expected_checks": {
    "market": {
      "min_title_candidates": 3,
      "required_tags": ["都市", "重生", "商业", "逆袭"],
      "must_include_selling_points": ["未来行业节点", "底层逆袭", "本地生活"]
    },
    "plot": {
      "min_chapter_outlines": 30,
      "chapter_1": {
        "must_have_goal": true,
        "must_have_conflict": true,
        "must_have_cliffhanger": true,
        "required_events": ["重生", "外卖站危机", "避开事故"]
      }
    },
    "character": {
      "required_roles": ["protagonist", "antagonist"],
      "protagonist_must_have": ["未来经验", "创业失败遗憾", "主动目标"]
    },
    "worldbuilding": {
      "required_world_elements": ["外卖站规则", "本地生活行业节点", "资金和人脉约束"],
      "required_hard_rules": ["未来信息不能无代价碾压", "商业决策必须受资源限制"],
      "required_seed_facts": ["林舟掌握未来行业节点", "外卖站存在即时危机"]
    },
    "writer": {
      "chapter_1_min_word_count": 1800,
      "must_include": ["暴雨或配送压力", "站点责任", "主角主动决策"],
      "forbidden": ["长篇前世流水账", "无代价碾压"]
    },
    "continuity": {
      "must_track_facts": ["主角使用未来经验", "站点责任归属", "关键人物关系变化"],
      "require_character_state_updates": true,
      "max_high_severity_issues": 0
    },
    "style": {
      "must_preserve": ["商业决策逻辑", "主角主动性", "章尾钩子"],
      "must_improve": ["压缩前世解释", "提高行动和对白比例", "降低行业分析腔"],
      "forbidden": ["机械商业分析", "只讲趋势不写场景"]
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
