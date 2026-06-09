# novel-agent UI 内容指南

日期：2026-06-09
负责人：开发者 B
面向对象：开发者 C 的 Web 工作台、审稿面板和 AgentRun 面板

本文档只定义前端展示口径，不新增 API DTO。字段来源以 `docs/API.md`、`docs/SCHEMAS.md` 和当前 Rust domain 为准。

## 1. 审稿面板目标

审稿面板不是模型原文展示区，而是作者的返工决策面板。第一屏必须让作者立刻知道：

- 本章能不能过线。
- 哪些维度拖后腿。
- 应该局部修、开头修、结尾修、语言润色，还是整章重写。
- 哪些事实、伏笔和人物状态不能改丢。

推荐数据源：

```http
GET /api/novels/{novel_id}/chapters/{chapter_index}/review
GET /api/novels/{novel_id}/chapters/{chapter_index}/continuity
GET /api/novels/{novel_id}/facts?limit=100
GET /api/novels/{novel_id}/runs?role=reviewer&task=review_chapter&limit=20
```

## 2. ReviewReport 展示字段

### 2.1 顶部状态

| 字段 | UI 标签 | 展示建议 |
| --- | --- | --- |
| `total_score` | 总分 | 大号数字，低于 75 用警示色 |
| `passed` | 通过状态 | `通过` / `需返工` |
| `created_at` | 审稿时间 | 使用相对时间或本地时间 |
| `rewrite_instruction.needed` | 返工建议 | `无需返工` / `建议返工` |
| `rewrite_instruction.priority` | 优先级 | `低` / `中` / `高` |

通过线文案：

```text
通过线：总分 >= 75，节奏 >= 7，连续性 >= 8，章尾钩子 >= 7
```

### 2.2 评分维度

| 字段 | 中文标签 | 前端分组 |
| --- | --- | --- |
| `opening_hook_score` | 开头吸引力 | 读者进入 |
| `pacing_score` | 情节推进 | 读者进入 |
| `payoff_score` | 爽点回报 | 情绪回报 |
| `character_score` | 人物表现 | 人物与对白 |
| `dialogue_score` | 对话自然度 | 人物与对白 |
| `continuity_score` | 设定一致性 | 连续性 |
| `cliffhanger_score` | 章尾钩子 | 连载动力 |
| `platform_fit_score` | 平台适配度 | 连载动力 |

分数标签：

| 分数 | 标签 | 展示建议 |
| --- | --- | --- |
| 9-10 | 强 | 稳定正向 |
| 7-8 | 达标 | 中性或轻正向 |
| 5-6 | 偏弱 | 需要关注 |
| 0-4 | 严重 | 放入优先返工列表 |

### 2.3 问题列表

`issues[]` 是作者最需要看的列表，排序建议：

1. `severity = high`
2. `severity = medium`
3. `severity = low`
4. 同级内按 `dimension` 的业务顺序：`continuity`、`pacing`、`cliffhanger`、`opening_hook`、`payoff`、`character`、`dialogue`、`platform_fit`

字段展示：

| 字段 | UI 标签 | 展示建议 |
| --- | --- | --- |
| `severity` | 严重程度 | 见下表 |
| `dimension` | 维度 | 使用中文标签 |
| `location` | 位置 | 可显示为小字，空值用 `整章` |
| `description` | 问题说明 | 主体文字，不要截断到不可读 |

严重程度标签：

| 值 | 中文 | 含义 |
| --- | --- | --- |
| `high` | 高 | 影响章节成立或会破坏连续性 |
| `medium` | 中 | 影响阅读动力，需要返工 |
| `low` | 低 | 可在人工精修时处理 |

### 2.4 修改建议

`suggestions[]` 用作作者待办清单。建议展示为可勾选列表，但勾选状态只存在前端本地即可，不要求后端持久化。

空状态：

```text
暂无修改建议。本章达到当前连载通过线。
```

## 3. 返工类型

`rewrite_instruction.rewrite_type` 是字符串，前端应按以下值展示；未知值显示为 `自定义返工`，不要报错。

| 值 | 中文标签 | 使用场景 | 推荐操作按钮 |
| --- | --- | --- | --- |
| `none` | 无需返工 | 已通过或只需人工轻修 | 隐藏重写强调态 |
| `partial` | 局部返工 | 主事件可保留，但节奏、爽点或对白偏弱 | `按建议局部重写` |
| `full` | 整章重写 | 总分过低、连续性硬伤或章节不成立 | `整章重写` |
| `opening` | 开头重写 | 开篇慢、目标不清、冲突进入太晚 | `重写开头` |
| `ending` | 结尾重写 | 章尾钩子弱、下一章期待不足 | `重写结尾` |
| `style` | 语言润色 | 事实成立但表达、对白或网文语感弱 | `语言润色` |

展示字段：

| 字段 | UI 标签 | 用法 |
| --- | --- | --- |
| `goals[]` | 返工目标 | 放在返工卡片顶部 |
| `preserve[]` | 必须保留 | 与事实表、伏笔表并列展示 |
| `change[]` | 必须修改 | 作为重写前检查清单 |
| `avoid[]` | 避免事项 | 放在折叠区，防止作者误改 |

## 4. Continuity 与 Facts 侧栏

连续性侧栏数据源是 `GET /api/novels/{novel_id}/chapters/{chapter_index}/continuity`。

| 字段 | UI 标签 | 展示建议 |
| --- | --- | --- |
| `report.passed` | 连续性状态 | `连续性通过` / `存在冲突` |
| `report.issues[]` | 连续性问题 | 优先展示在审稿问题上方 |
| `report.new_facts[]` | 新事实 | 可直接加入事实表视图 |
| `report.character_state_updates[]` | 人物状态变化 | 按人物聚合 |
| `report.foreshadowing_updates[]` | 伏笔变化 | 按 `seed/status/note` 展示 |

事实表字段来自 `GET /api/novels/{novel_id}/facts?limit=100`：

| 字段 | UI 标签 |
| --- | --- |
| `subject` | 主体 |
| `predicate` | 关系 |
| `object` | 内容 |
| `importance` | 重要度 |
| `chapter_id` | 来源章节 |

重要度展示：

| `importance` | 标签 |
| --- | --- |
| 4-5 | 核心事实 |
| 2-3 | 常规事实 |
| 0-1 | 背景事实 |

## 5. AgentRun 面板输出摘要

AgentRun 原始 JSON 只放详情抽屉，列表和时间线优先展示摘要。当前 API 的 `output_summary` 可直接使用；如果为空，前端可按 role 从 `structured` 派生。

| role | 主要摘要字段 |
| --- | --- |
| `market` | `market_analysis.genre_fit`、`title_candidates[0].title`、`opening_strategy.first_conflict` |
| `plot` | `plot_plan.main_conflict`、`chapter_outlines.length`、首章 `title` |
| `character` | `characters.length`、主角姓名、主要对立角色 |
| `worldbuilding` | `world_setting` 的核心规则、`facts_to_seed.length` |
| `writer` | `chapter_draft.title`、`word_count`、`summary` |
| `continuity` | `continuity_report.passed`、`issues.length`、`new_facts.length` |
| `style` | `styled_chapter.title`、`style_notes[]`、`changes.length` |
| `reviewer` | `review_report.total_score`、`passed`、`issues.length` |

状态标签：

| status | 中文 |
| --- | --- |
| `ok` | 成功 |
| `fallback` | Fallback |
| `parse_error` | 解析失败 |

真实模型验收和 UI 调试都应把 `fallback`、`parse_error` 放到显眼位置。

## 6. 三个 UI demo 项目

现有样例文件可作为 UI mock 数据来源：

| 文件 | 题材 | 平台 | UI 验收重点 |
| --- | --- | --- | --- |
| `examples/urban_rebirth.md` | 都市重生商业文 | 番茄 | 快节奏冲突、短周期爽点、现实压力 |
| `examples/fantasy_upgrade.md` | 玄幻升级文 | 起点 | 世界规则、升级代价、长期伏笔 |
| `examples/romance_comeback.md` | 女性向逆袭复仇 | 番茄 | 情绪回报、关系张力、复仇推进 |

第一版 Web demo 推荐直接使用 `docs/WEB_DEMO_CONTENT.md` 中的 `urban_rebirth_fanqie_demo` 内容包。该内容包已经包含作品信息、第一章正文、ReviewReport mock、Continuity/facts mock、AgentRun mock 和 DeepSeek 负向对照摘要。

每个 demo 项目至少准备：

- 作品基础信息和 NovelBible。
- 30 章目录。
- 3 张人物卡。
- 第 1 章正文。
- v1 / v2 两个章节版本。
- 一份 ReviewReport。
- 一份 ContinuityReport。
- 5 条 AgentRun。

## 7. 空状态文案

| 场景 | 文案 |
| --- | --- |
| 没有审稿报告 | `本章还没有审稿。生成正文后可以运行 Reviewer Agent。` |
| 没有连续性报告 | `本章还没有连续性检查结果。生成或重写章节后会自动检查。` |
| 没有 facts | `暂无事实记录。生成章节后，Continuity Agent 会提取可追踪事实。` |
| 没有 AgentRun | `暂无运行记录。创建作品或生成章节后会显示 Agent 执行时间线。` |
| 真实模型失败 | `本次模型调用未通过验收，请查看 AgentRun 的 fallback 或 parse_error。` |

## 8. 前端不要依赖的内容

- 不要解析 `raw_text` 做业务展示。
- 不要把 `raw_notes` 当成稳定字段。
- 不要假设 `rewrite_type` 只有固定枚举；未知值必须能降级展示。
- 不要把 `AgentRun.status = ok` 等同于文本质量过线；质量以 ReviewReport 为准。
