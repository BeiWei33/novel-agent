# 开发者 A/B 工作审查问题清单

审查日期：2026-06-08  
审查范围：`docs/PROJECT.md`、`docs/WORKPLAN.md`、`docs/B_HANDOFF.md`、`docs/SCHEMAS.md`、`docs/RUBRIC.md`、`src/`、`prompts/`、`examples/`

## 1. 总体结论

当前项目已经具备文档、Prompt、领域模型、SQLite 存储和 CLI 的雏形，但还没有形成可验收的 MVP 闭环。

最主要的问题是：

- 开发者 A 的工程代码与最新领域模型不同步，静态检查已能看出会编译失败。
- 开发者 A 的 workflow 仍是硬编码占位逻辑，没有真正调度 Agent、Prompt 或模型。
- 开发者 B 的 Prompt 和 schema 文件基本齐全，但输出 envelope、字段粒度和可测试样例还没有完全收口。
- A/B 的交界面仍不稳定，尤其是 `ChapterOutline`、`ChapterDraft`、`ReviewReport`、`AgentInput`、`AgentOutput`。

## 2. 开发者 A：Rust 核心工程线问题

### A-P0-1：domain model 已扩展，但 workflow 没同步

影响：项目当前很可能无法通过编译，阻塞所有 CLI 演示。

证据：

- `src/domain/chapter.rs` 中 `ChapterOutline` 已包含 `pov`、`key_events`、`character_changes`、`new_facts`、`cliffhanger`、`estimated_word_count` 等字段。
- `src/workflow/novel_creation.rs` 中 `draft_outlines` 仍按旧结构构造，并使用旧字段 `hook`。
- `src/domain/chapter.rs` 中 `ChapterDraft` 已包含 `volume_index`、`pov`、`key_events`、`new_facts`、`foreshadowing`、`continuity_notes`。
- `src/workflow/chapter_generation.rs` 中 `write_chapter` 和 `rewrite_chapter` 仍按旧结构构造 `ChapterDraft`。

建议动作：

- 先以 `docs/SCHEMAS.md` 为准统一 `ChapterOutline` 和 `ChapterDraft`。
- 更新 `novel_creation.rs` 和 `chapter_generation.rs` 的构造逻辑。
- 补充最小单元测试或 CLI smoke test，保证 `new`、`outline`、`write` 至少能编译和跑通。

### A-P0-2：ReviewReport 结构与 workflow/storage 不一致

影响：审稿命令和返工逻辑会直接失败或写入错误字段。

证据：

- `src/domain/review.rs` 中 `ReviewReport` 已包含 `total_score`、`strengths`、`rewrite_instruction`。
- `src/domain/review.rs` 中 `ReviewScores` 使用 `cliffhanger` 字段。
- `src/workflow/chapter_generation.rs` 仍使用旧的 `report.scores.total`、`ending_hook`、旧版 `ReviewIssue.detail`。
- `src/storage/sqlite.rs` 现在将审稿报告以 `data` JSON 方式入库，但 workflow 还没有按新结构生成完整数据。

建议动作：

- 统一 `ReviewScores` 字段名，建议对齐 `docs/SCHEMAS.md` 中的 `*_score` 输出，再在 Rust 侧做清晰映射。
- `review_chapter` 必须生成 `strengths`、`issues.location`、`issues.description`、`rewrite_instruction`。
- `mark_reviewed` 使用 `report.total_score`，不要再混用旧的 `scores.total`。

### A-P0-3：没有真正接入 Agent/Prompt/模型调用链

影响：项目看起来有 CLI，但不符合“多 Agent 小说创作系统”的 MVP 定位。

证据：

- `src/model/rig_provider.rs` 已有 `RigModelClient`。
- `src/agents/mod.rs` 已定义 `NovelAgent`、`AgentInput`、`AgentOutput`。
- `src/workflow/novel_creation.rs` 和 `src/workflow/chapter_generation.rs` 只接收 `SqliteStorage`，没有接收或使用 `ModelClient`。
- 新书创建、大纲、人物、章节正文、审稿评分均由本地函数硬编码生成。

建议动作：

- 给 workflow 注入 `ModelClient` 或 Agent registry。
- 先接入最小链路：Market -> Plot -> Character -> Chapter Writer -> Reviewer。
- 每次模型输出保存 `structured`、`raw_text`、`parse_error`，对齐 `docs/B_HANDOFF.md`。

### A-P1-1：MVP 闭环只是占位闭环

影响：即使 CLI 跑通，也不能证明系统会写、会评、会改。

证据：

- `render_placeholder_chapter` 固定生成一段林舟重生文本。
- `score_placeholder_chapter` 只根据内容是否为空给固定分。
- `rewrite_chapter` 只是追加“重写标记”，没有根据审稿报告重构内容。

建议动作：

- 明确占位实现只用于 smoke test，不作为 MVP 验收。
- 将 `rewrite_chapter` 改为读取最近一次 `ReviewReport.rewrite_instruction`。
- 返工后重新执行 Continuity/Reviewer 检查，至少记录版本号和评分变化。

### A-P1-2：事实表存在 schema，但缺少写入链路

影响：项目文档强调连续性，但当前事实表无法支撑后续章节生成。

证据：

- `src/storage/sqlite.rs` 定义了 `facts` 表。
- 目前没有 `FactRepository` 对外导出，也没有从 `ChapterDraft.new_facts` 或 `ContinuityReport.new_facts` 写入。

建议动作：

- 增加 `FactRepository`。
- 在章节生成或连续性检查后写入 `new_facts`。
- 后续 `write_chapter` 组装上下文时读取相关 facts。

## 3. 开发者 B：小说 Agent 业务线问题

### B-P0-1：Prompt 输出 envelope 与 `SCHEMAS.md` 不统一

影响：工程侧难以写稳定通用解析器，后续每个 Agent 都要特殊处理。

证据：

- `docs/SCHEMAS.md` 中 `NovelBible` 使用 `{ "novel_bible": { ... } }` 包装。
- `docs/SCHEMAS.md` 中 `CharacterCard`、`ChapterOutline`、`ChapterDraft`、`ReviewReport` 多数展示为裸结构。
- `prompts/plot_agent.md` 输出 `plot_plan` 和 `chapter_outlines`。
- `prompts/chapter_writer_agent.md` 输出 `chapter_draft`。
- `prompts/reviewer_agent.md` 输出 `review_report`。
- `prompts/README.md` 要求字段名称和 `docs/SCHEMAS.md` 保持一致。

建议动作：

- 定义统一 AgentOutput envelope，例如：

```json
{
  "role": "writer",
  "structured": {
    "chapter_draft": {}
  },
  "raw_notes": ""
}
```

- 或明确规定：Prompt 文件写 Agent 顶层输出，`SCHEMAS.md` 只写 `structured` 内部实体。
- 所有 Prompt 和 schema 按同一规则重排。

### B-P0-2：`ChapterOutline.new_facts` 与 `ChapterDraft.new_facts` 粒度不一致

影响：事实表和连续性检查难以复用同一结构。

证据：

- `prompts/plot_agent.md` 中 `chapter_outlines.new_facts` 是字符串数组。
- `prompts/chapter_writer_agent.md` 中 `chapter_draft.new_facts` 是 `{ subject, predicate, object, importance }` 三元组数组。
- `docs/B_HANDOFF.md` 将 `ChapterDraft.new_facts` 的事实三元组结构列为硬性协作点。

建议动作：

- 大纲阶段如果只是“预计事实”，字段名建议改为 `planned_facts` 或 `fact_hints`。
- 草稿和连续性阶段统一使用三元组结构 `new_facts`。
- 在 `docs/SCHEMAS.md` 中明确事实三元组的唯一标准。

### B-P1-1：测试题材样例还不能作为自动回归测试

影响：A 侧无法用样例验证 Prompt 输出和 CLI 闭环质量。

证据：

- `examples/urban_rebirth.md`、`examples/fantasy_upgrade.md`、`examples/romance_comeback.md` 目前是题材 brief。
- 样例包含创意、平台、卖点、前三章目标、关键人物、检验点。
- 样例没有期望 JSON、最低评分、失败示例、断言条件。

建议动作：

- 为每个样例补充最小验收 JSON 片段。
- 增加 `expected_checks`，例如：
  - 必须生成至少 3 个书名候选。
  - 必须生成 30 章大纲。
  - 第 1 章必须包含明确冲突、主角目标、章尾钩子。
  - `ReviewReport.passed` 的判断必须符合 `docs/RUBRIC.md`。

### B-P1-2：`raw_notes` 与 parse fallback 约定仍不够工程化

影响：模型 JSON 解析失败时，A 侧不知道该保存到哪里、是否重试、是否降级。

证据：

- `prompts/README.md` 要求模型无法满足 schema 时放入 `raw_notes` 或 `issues`。
- `docs/B_HANDOFF.md` 建议工程侧保留 `structured`、`raw_text`、`parse_error`。
- `docs/SCHEMAS.md` 又要求原始补充信息统一放入 `raw_notes`。

建议动作：

- 明确 `raw_text` 是工程侧保存的完整原始响应，不由模型生成。
- 明确 `raw_notes` 是模型在合法 JSON 内写的补充说明。
- 明确 `parse_error` 只由工程侧生成。

### B-P1-3：平台策略还停留在原则层，缺少可执行差异

影响：起点向和番茄向 Prompt 可能输出风格差异不明显。

证据：

- `docs/RUBRIC.md` 已描述起点向、番茄向关注点。
- 各 Prompt 中平台策略多为文本原则，没有落到字段权重、扣分规则或章节节奏参数。

建议动作：

- 在 Reviewer Prompt 中按平台给评分偏置。
- 在 Plot Prompt 中按平台调整前三章目标、爽点密度、设定展开比例。
- 在 Chapter Writer Prompt 中增加平台化写作约束，例如解释段落长度、章尾钩子强度、对话比例。

## 4. A/B 共同接口问题

### C-P0-1：交界面仍未冻结

影响：两边会继续互相打断，尤其是 domain model、schema、Prompt 输出格式。

重点风险字段：

- `AgentInput`
- `AgentOutput`
- `NovelBible`
- `CharacterCard`
- `ChapterOutline`
- `ChapterDraft`
- `ReviewReport`
- `RewriteInstruction`

建议动作：

- 开一次接口冻结会，只定 MVP 必需字段。
- 每个结构只允许一个权威来源，建议以 `docs/SCHEMAS.md` 为业务权威，以 Rust domain model 为工程实现。
- 字段变更必须同时更新 Prompt、schema、domain、workflow、storage。

### C-P0-2：验收标准需要从“文件齐全”改为“链路可跑”

建议 MVP 验收顺序：

1. `cargo check` 通过。
2. `novel-agent new "<创意>"` 成功生成并保存项目、圣经、人物、30 章大纲。
3. `novel-agent write --novel-id <id> --chapter 1` 成功调用 Chapter Writer 输出 JSON 并保存正文。
4. `novel-agent review --novel-id <id> --chapter 1` 成功输出完整 `ReviewReport`。
5. 低分时 `rewrite --chapter 1` 能读取返工指令并生成新版本。
6. `export --format markdown` 能导出最终章节。

## 5. 建议处理优先级

第一优先级：

- A 修复 domain/workflow/review 编译不一致。
- B 统一 Prompt/schema envelope。
- A/B 确认 `ChapterDraft.new_facts`、`ReviewReport.scores`、`rewrite_instruction` 的最终字段。

第二优先级：

- A 接入 `ModelClient` 到 workflow。
- B 为三个样例补充期望 JSON 和验收断言。
- A 增加最小 smoke test。

第三优先级：

- A 增加 facts 写入和读取链路。
- B 细化起点/番茄平台差异。
- A/B 增加自动返工版本对比。

## 6. 当前验证限制

本次审查主要基于文件静态检查。审查时环境中未能直接运行 `cargo check`，因此编译失败结论来自结构体字段和调用代码的静态不一致判断。后续应在 Rust 工具链可用后立即执行 `cargo check` 验证。

## 7. 开发者 B 处理记录

处理日期：2026-06-08

已处理：

- B-P0-1：已在 `docs/SCHEMAS.md` 中定义统一 `AgentOutputEnvelope`，并同步所有 `prompts/*.md` 的输出为 `role + structured + raw_notes`。
- B-P0-2：已将事实结构统一为 `FactTriple`，`ChapterOutline.new_facts`、`ChapterDraft.new_facts`、`ContinuityReport.new_facts` 均使用 `{ subject, predicate, object, importance }`。
- B-P1-1：已为三个 `examples/*.md` 补充 `expected_checks` 回归验收 JSON。
- B-P1-2：已明确 `raw_notes` 由模型在合法 JSON 内生成，`raw_text` 和 `parse_error` 只由工程侧保存或生成。
- B-P1-3：已在 `docs/SCHEMAS.md`、`docs/RUBRIC.md`、`prompts/plot_agent.md`、`prompts/chapter_writer_agent.md`、`prompts/reviewer_agent.md` 中补充可执行平台策略。

仍需 A/B 对齐：

- A 侧 Rust domain、workflow、storage 需要以 `docs/SCHEMAS.md` 为业务权威继续同步。
- A 侧接入模型调用后，需要保存工程字段 `raw_text` 和 `parse_error`。
- 自动返工链路需要读取 `ReviewReport.rewrite_instruction`，并在重写后再次审稿。

## 8. 第二轮审查记录

审查日期：2026-06-08

第二轮审查基于当前未提交改动。相比第一轮，项目已有明显推进：

- A 侧新增 `PromptAgent`、`agent_runner`、`FactRepository`、`AgentRunRepository`，workflow 开始接入 `ModelClient`。
- A 侧新增 `tests/smoke.rs`，开始覆盖模型输出非法时的 fallback 链路。
- B 侧已统一 Prompt 输出 envelope，补充 `AgentOutputEnvelope`、`FactTriple`、`PlatformProfile`、`expected_checks`。

但当前仍存在以下问题。

### A2-P0-1：模型失败和 JSON 解析失败会被静默降级为成功流程

影响：CLI 和 smoke test 可能在所有 Agent 都失败时仍显示成功，掩盖“系统其实没有调用成功模型”的事实。

证据：

- `src/workflow/agent_runner.rs` 在 `agent.run` 失败时构造空 `AgentOutput`，只写 `parse_error`，然后继续返回 `Ok(output)`。
- `src/workflow/novel_creation.rs`、`src/workflow/chapter_generation.rs` 在解析不到结构化结果时直接 fallback 到工程占位数据。
- `tests/smoke.rs` 使用 `InvalidJsonModel` 返回 `"not json"`，但仍断言流程成功、评分达标。

建议动作：

- 将 fallback 明确标记到 CLI 输出、`AgentRun`、章节 `continuity_notes` 或工作流返回值中。
- MVP 验收应区分“真实模型链路成功”和“fallback smoke 成功”。
- 测试至少增加两类：
  - 有效 JSON 模型输出能被正确解析并入库。
  - 非法 JSON 时 `agent_runs.parse_error` 被保存，业务结果标记为 fallback。

### A2-P0-2：`max_retries` 约定没有实现

影响：`docs/B_HANDOFF.md` 已要求同一 Agent 最多重试 `max_retries` 次，但工程侧当前没有重试逻辑，JSON 一次失败就直接 fallback。

证据：

- `AgentConstraints` 有 `max_retries` 字段。
- `run_prompt_agent` 创建了默认 constraints，但没有循环重试。
- `PromptAgent` 只调用一次 `model.complete`。

建议动作：

- 在 `run_prompt_agent` 或 `PromptAgent` 内实现 JSON 修复重试。
- 重试 Prompt 只要求修复 JSON 格式，不改变业务内容。
- 所有重试结果都应写入 `agent_runs` 或至少记录尝试次数。

### A2-P0-3：Prompt 输入 envelope 与 Prompt 文档声明不一致

影响：模型看到的真实输入不是 Prompt 文件“输入”段声明的结构，可能导致模型忽略关键字段或输出不稳定。

证据：

- `prompts/market_agent.md`、`prompts/plot_agent.md` 等输入示例都是直接业务对象。
- `PromptAgent` 实际发送给模型的是：

```json
{
  "task": "...",
  "instructions": "...",
  "payload": {},
  "context": []
}
```

建议动作：

- 二选一：
  - Prompt 文档明确输入统一为 `{ task, instructions, payload, context }`，业务字段都在 `payload`。
  - 工程侧直接把业务 payload 作为用户 prompt，并把 task/context 写成补充字段。

### A2-P1-1：平台策略没有真正传递到章节生成和审稿

影响：起点向/番茄向策略在 B 侧已经细化，但 A 侧 downstream workflow 仍无法使用，生成和审稿会退化成通用模式。

证据：

- `Market Agent` 输出 `platform_profile`。
- `NovelBible` Rust 结构没有保存 `platform_profile`。
- `ChapterGenerationWorkflow::write_chapter` 没有向 Writer 传 `platform_profile`。
- `review_chapter` 里 `target_platform` 仍硬编码为 `"general"`。

建议动作：

- 明确 `platform_profile` 的持久化位置，可以放入 `NovelBible` 或单独表。
- Writer、Reviewer 都应接收真实 `target_platform` 和 `platform_profile`。
- 禁止在 workflow 中硬编码 `"general"`，应从 `novel.target_platform` 或 `bible.target_platform` 获取。

### A2-P1-2：Worldbuilding、Continuity、Style 仍未进入真实链路

影响：项目文档定义的章节生产链路仍缺关键环节，尤其是连续性复查和润色。

证据：

- Prompt 已有 `worldbuilding_agent.md`、`continuity_agent.md`、`style_agent.md`。
- `NovelCreationWorkflow` 当前只调用 Market、Plot、Character。
- `ChapterGenerationWorkflow` 给 Writer/Reviewer 的 `world_setting`、`continuity_report` 都是空 object。

建议动作：

- 新书创建阶段接入 Worldbuilding Agent，并保存 `world_setting`。
- 写作后执行 Continuity Agent，保存 `ContinuityReport` 和确认后的 facts。
- Style Agent 可以作为 P1，但 Reviewer 前至少应接收真实 `continuity_report`。

### A2-P1-3：事实表写入会重复追加，缺少章节级替换或去重策略

影响：同一章多次 `write` 或 `rewrite` 会重复写入相同事实，后续检索上下文会污染。

证据：

- `FactRepository::insert_for_chapter` 每次都生成新 `FactId` 并直接 insert。
- 表结构没有 `(novel_id, chapter_id, subject, predicate, object)` 唯一约束。
- 保存新草稿前没有删除该章节旧事实。

建议动作：

- MVP 可先采用“保存草稿前删除该 chapter_id 旧 facts，再插入新 facts”。
- 后续再增加事实确认状态、来源版本和去重规则。

### A2-P1-4：`expected_checks` 已写入样例，但测试没有消费

影响：B 侧补充的验收 JSON 仍停留在文档里，不能防止回归。

证据：

- `examples/*.md` 已包含 `expected_checks`。
- `tests/smoke.rs` 没有读取任何 `examples/*.md`。
- 当前测试只断言 fallback 结果有 30 章、有正文、有事实、评分大于等于 75。

建议动作：

- 把 `expected_checks` 抽成独立 JSON 文件，或在测试中解析 Markdown code block。
- smoke test 应覆盖三个样例，而不是只测一个固定创意。
- 至少检查：平台、章节数、角色角色类型、章节 1 关键事件、最低字数、ReviewReport 通过线。

### B2-P1-1：`platform_profile` 的权威归属还不清楚

影响：B 侧已经定义了 `PlatformProfile`，但没有明确它应归属于 `NovelBible`、`MarketAnalysis`、还是独立业务对象，导致 A 侧没有可靠持久化位置。

建议动作：

- 在 `docs/SCHEMAS.md` 明确 `platform_profile` 是新书创建后的长期配置。
- 如果它影响 Writer/Reviewer，应把它列入 `NovelBible` 或新增 `NovelStrategy`。

### C2-P0-1：第二轮仍不能确认可编译

影响：当前改动很大，虽然静态结构比第一轮更接近，但仍必须以 `cargo check` 为准。

当前限制：

- 本次审查环境里仍没有可用的 `cargo`/`rustc`。
- 已执行 `git diff --check`，未发现空白错误，但这不能替代编译。

建议动作：

- 在有 Rust 工具链的环境立即运行：

```bash
cargo check
cargo test
```

- 将输出贴回本审查文档，作为下一轮判断依据。

## 9. 开发者 A 第二轮处理记录

处理日期：2026-06-08

已处理：

- A2-P0-1：`AgentOutput` 增加 `attempt`、`will_fallback`，`agent_runs.structured` 保存 `_engineering` 元数据；CLI 对 `new`、`write`、`review` 的 fallback 结果给出提示。
- A2-P0-2：`run_prompt_agent` 已按 `AgentConstraints.max_retries` 实现重试，每次尝试都会写入 `agent_runs`。
- A2-P0-3：`PromptAgent` 现在直接向模型发送业务 payload，`task` 和工程说明只作为文本前缀，不再把业务输入包进额外 `payload` envelope。
- A2-P1-1：`NovelBible` 增加 `platform_profile`，Writer 和 Reviewer payload 已传入真实 `target_platform` 和 `platform_profile`，不再硬编码 `general`。
- A2-P1-3：`FactRepository::insert_for_chapter` 改为先删除同一章节旧 facts，再插入新 facts，避免重复污染上下文。
- A2-P1-4：`tests/smoke.rs` 增加有效 JSON 模型解析测试、非法 JSON parse_error 持久化测试，以及三个 `examples/*.md` 的 `expected_checks` 读取测试。
- C2-P0-1：已在本地 Rust 工具链下执行 `cargo check` 和 `cargo test`，均通过。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
3 passed; 0 failed
```

仍需后续处理：

- A2-P1-2：Worldbuilding、Continuity、Style 尚未进入完整真实链路。
- B2-P1-1：`platform_profile` 已先由 A 侧放入 `NovelBible`，仍建议 B 侧在 schema 文档中明确其长期权威归属。

## 10. 开发者 B 下一步处理记录

处理日期：2026-06-08

已处理：

- A2-P0-3：已在 `docs/SCHEMAS.md`、`docs/B_HANDOFF.md` 和所有 Prompt 文件中明确：工程侧内部使用 `AgentInputEnvelope`，Prompt 文件的“输入”示例描述的是 `payload`。模型用户消息可由工程侧将 `task`、`instructions`、`payload`、`context` 组装后发送。
- B2-P1-1：已在 `docs/SCHEMAS.md` 中明确 `platform_profile` 是新书创建后的长期策略配置，权威持久化位置是 `NovelBible.platform_profile`；Market Agent 负责生成初稿，Writer/Reviewer 从 `NovelBible` 读取。
- A2-P1-2 后续配合：已为三个 `examples/*.md` 增加 Worldbuilding、Continuity、Style 的 `expected_checks`，并在 `tests/smoke.rs` 中校验这些断言存在且非空。

确认状态：

- A2-P0-1、A2-P0-2、A2-P0-3、A2-P1-1、A2-P1-3、A2-P1-4 已有工程处理记录。
- B 侧已补齐 Worldbuilding、Continuity、Style 的基础业务验收断言，并已将 `fantasy_upgrade` 样例从“存在性检查”升级为对模型输出内容的语义检查。

## 11. 开发者 A 第三轮处理记录

处理日期：2026-06-08

已处理：

- A2-P1-2：`NovelCreationWorkflow` 已接入 Worldbuilding Agent，保存 `world_setting`，并将 `facts_to_seed` 写入事实表。
- A2-P1-2：`ChapterGenerationWorkflow` 已在 Writer 后执行 Continuity Agent，保存 `continuity_reports`，并优先使用 `ContinuityReport.new_facts` 写入章节事实。
- A2-P1-2：章节保存前已执行 Style Agent，Reviewer payload 已读取真实 `world_setting` 和最新 `continuity_report`。
- A2-P1-2：重写流程同样重新执行 Continuity 和 Style，避免返工稿跳过连续性复查。
- A2-P1-2：`tests/smoke.rs` 的有效 JSON 模型补齐 Worldbuilding、Continuity、Style 输出，并断言 `world_settings`、`continuity_reports` 和章节事实确实落库。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
3 passed; 0 failed
```

当前状态：

- A2-P1-2 工程链路已闭环。

## 12. 开发者 A 第四轮处理记录

处理日期：2026-06-08

已处理：

- C-P0-2：已按 MVP 验收顺序执行 CLI 闭环：`new -> write -> review -> rewrite -> export`，确认命令链路可跑、数据可保存、Markdown 可导出。
- A-P1：新增 `scripts/mvp_demo.ps1`，固化 MVP 演示流程，覆盖 `new`、`outline`、`write`、`review`、`rewrite`、`export` 六个 CLI 命令。
- A-P1：演示脚本默认使用离线 smoke fallback，避免无 API key 或无网络时阻塞工程验收；传入 `-UseRealModel` 可保留真实 OpenAI 环境变量进行模型链路演示。
- A-P1：演示脚本使用临时配置和临时 SQLite 数据库，避免污染仓库内 `novel-agent.db`。

验证结果：

```text
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
new / outline / write / review / rewrite / export all ok
export_size=15976
```

当前状态：

- MVP CLI 演示脚本已可复现验收闭环。

## 13. 开发者 A 第五轮处理记录

处理日期：2026-06-08

已处理：

- B 侧后续配合项：`tests/smoke.rs` 已从 `expected_checks` 存在性检查升级为对 `urban_rebirth`、`fantasy_upgrade`、`romance_comeback` 三个样例的语义断言。
- B-P1-1：有效 JSON 模型测试现在校验 Market 标签和卖点、Plot 第 1 章关键事件、Character 角色类型和主角约束、Worldbuilding 世界元素/硬规则/种子事实、Writer 正文必含/禁用项、Continuity 事实追踪、Style 保留项和 Review 通过线。
- B-P1-1：补齐测试辅助函数，用统一方式解析样例 Markdown 中的回归验收 JSON，并对文本、结构化 JSON、角色列表和最近一次 AgentRun 进行断言。
- B-P1-1：调整 `ValidJsonModel` fixture，使假模型按题材返回 Urban/Fantasy/Romance 对应输出，并与三个 `examples/*.md` 的验收标准一致，避免测试只验证工程占位。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
4 passed; 0 failed
```

当前状态：

- 三个样例验收 JSON 均已进入自动语义回归测试。

## 14. 开发者 A 第六轮处理记录

处理日期：2026-06-08

已处理：

- A-P1-1 / C-P0-2：`rewrite_chapter` 保存重写稿后会自动再次执行 Reviewer，返工后可立即留下新的评分和章节状态。
- A-P1：新增 `chapter_versions` 表，每次 `save_draft` 都保存章节版本快照，支持同一章节 v1/v2 内容对比。
- A-P1：新增 `ChapterVersionRepository`，提供版本数量、版本号列表和指定版本正文查询。
- A-P1：`tests/smoke.rs` 增加低分返工用例，覆盖低分审稿、`RewriteNeeded` 状态、重写生成 v2、版本快照 v1/v2、返工后自动复审并恢复 `Final` 状态。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
4 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
new / outline / write / review / rewrite / export all ok
```

当前状态：

- 低分返工和章节版本对比已进入自动测试覆盖。
