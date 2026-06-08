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

MVP 阶段工程侧使用统一 `AgentInput` envelope 调用 Prompt Agent：

```json
{
  "task": "create_novel | generate_outline | generate_chapter | review_chapter | rewrite_chapter | extract_facts | polish_style | check_continuity",
  "instructions": "本次调用的格式要求或重试要求",
  "payload": {},
  "context": []
}
```

Prompt 文档里的输入块均指 `payload`。MVP 阶段建议 `payload` 至少能表达以下字段：

```json
{
  "idea": "用户原始创意",
  "target_platform": "qidian | fanqie | general",
  "platform_profile": {},
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

不是每个 Agent 都需要全部字段，但保持统一 `AgentInput` envelope 会让工作流调度更简单。

## 推荐 AgentOutput 结构

Prompt 输出必须使用 `docs/SCHEMAS.md` 中的统一 envelope：

```json
{
  "role": "market | plot | character | worldbuilding | writer | continuity | style | reviewer",
  "structured": {},
  "raw_notes": ""
}
```

工程侧保存模型调用结果时，在 envelope 外追加两类工程字段：

```json
{
  "role": "writer",
  "structured": {
    "chapter_draft": {}
  },
  "raw_notes": "",
  "raw_text": "模型完整原始响应，由工程侧保存",
  "parse_error": null
}
```

原因：

- `structured` 用于正常工作流。
- `raw_notes` 用于模型在合法 JSON 内保留补充说明。
- `raw_text` 用于 JSON 解析失败后的人工排查，只由工程侧保存。
- `parse_error` 用于判断是否需要重试、修复 JSON 或降级保存，只由工程侧生成。

## JSON 解析失败约定

- 模型应始终尝试输出合法 JSON。
- 若模型无法满足某些字段，字段仍保留，用空字符串、空数组或保守默认值占位，并在 `raw_notes` 说明。
- 如果工程侧无法解析 JSON，保存完整 `raw_text`，`structured` 可保存为空 object，`parse_error` 写解析错误。
- 同一 Agent 在一次调用中最多重试 `max_retries` 次；重试 Prompt 应明确要求“只修复 JSON 格式，不改变业务内容”。

## 入库建议

MVP 最优先入库：

- `NovelBible`：新书创建后的全书核心设定。
- `NovelBible.platform_profile`：新书创建后的长期平台策略配置，由 Market Agent 生成初稿，Writer/Reviewer 从这里读取。
- `CharacterCard`：主角、核心配角、主要反派。
- `ChapterOutline`：前 30 章大纲。
- `ChapterDraft`：章节正文、摘要、事实、伏笔。
- `ReviewReport`：评分、问题、返工指令。
- `ContinuityReport`：新增事实和人物状态变化。

## 硬性协作点

以下字段变化需要 A/B 同步：

- `ReviewReport.scores` 的评分字段名。
- `ChapterOutline.new_facts` 和 `ChapterDraft.new_facts` 的事实三元组结构。
- `ChapterOutline.chapter_index` 是否从 1 开始。
- `target_platform` 的枚举值。
- `NovelBible.platform_profile` 的字段和默认值。
- `rewrite_instruction` 是否直接进入自动返工流程。

## 当前建议

- `chapter_index` 从 1 开始，贴近 CLI 用户习惯。
- 所有 Agent 先强制 JSON 输出，失败时保留原始响应。
- 所有事实字段统一为 `{ subject, predicate, object, importance }`。
- `ReviewReport` 的通过线按 `docs/RUBRIC.md` 执行。
- 起点向和番茄向先通过 `platform_profile` 和 Prompt 层约束区分，暂不拆独立工作流。
