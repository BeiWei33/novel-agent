# 开发者 B 交接说明

本文档记录小说业务侧对工程接口的最低诉求，方便开发者 A 接入 Prompt 和 schema。

## 已交付文件

- `prompts/market_agent.md`
- `prompts/plot_agent.md`
- `prompts/character_agent.md`
- `prompts/worldbuilding_agent.md`
- `prompts/chapter_writer_agent.md`
- `prompts/continuity_agent.md`
- `prompts/style_agent.md`
- `prompts/reviewer_agent.md`
- `docs/SCHEMAS.md`
- `docs/RUBRIC.md`
- `examples/urban_rebirth.md`
- `examples/fantasy_upgrade.md`
- `examples/romance_comeback.md`

## 推荐 AgentInput 上下文

MVP 阶段建议 `AgentInput` 至少能表达以下字段：

```json
{
  "idea": "用户原始创意",
  "target_platform": "qidian | fanqie | general",
  "novel_bible": {},
  "market_analysis": {},
  "plot_plan": {},
  "chapter_outline": {},
  "characters": [],
  "world_setting": {},
  "recent_summaries": [],
  "relevant_facts": [],
  "known_foreshadowing": [],
  "constraints": [],
  "target_word_count": 2500
}
```

不是每个 Agent 都需要全部字段，但保持统一 envelope 会让工作流调度更简单。

## 推荐 AgentOutput 结构

建议工程侧保留两类输出：

```json
{
  "role": "market | plot | character | worldbuilding | writer | continuity | style | reviewer",
  "structured": {},
  "raw_text": "模型原始响应",
  "parse_error": null
}
```

原因：

- `structured` 用于正常工作流。
- `raw_text` 用于 JSON 解析失败后的人工排查。
- `parse_error` 用于判断是否需要重试、修复 JSON 或降级保存。

## 入库建议

MVP 最优先入库：

- `NovelBible`：新书创建后的全书核心设定。
- `CharacterCard`：主角、核心配角、主要反派。
- `ChapterOutline`：前 30 章大纲。
- `ChapterDraft`：章节正文、摘要、事实、伏笔。
- `ReviewReport`：评分、问题、返工指令。
- `ContinuityReport`：新增事实和人物状态变化。

## 硬性协作点

以下字段变化需要 A/B 同步：

- `ReviewReport.scores` 的评分字段名。
- `ChapterDraft.new_facts` 的事实三元组结构。
- `ChapterOutline.chapter_index` 是否从 1 开始。
- `target_platform` 的枚举值。
- `rewrite_instruction` 是否直接进入自动返工流程。

## 当前建议

- `chapter_index` 从 1 开始，贴近 CLI 用户习惯。
- 所有 Agent 先强制 JSON 输出，失败时保留原始响应。
- `ReviewReport` 的通过线按 `docs/RUBRIC.md` 执行。
- 起点向和番茄向先只做 Prompt 层策略差异，暂不拆独立工作流。

