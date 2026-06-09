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

## 15. 开发者 A 第七轮处理记录

处理日期：2026-06-09

已处理：

- C-P0-1：新增 `docs/INTERFACE_FREEZE.md`，冻结 MVP 阶段 A/B 协作接口。
- C-P0-1：明确 `docs/SCHEMAS.md` 是业务字段权威，Rust domain/storage 可增加工程字段但不得改变业务字段含义。
- C-P0-1：明确 Agent 输入输出 envelope、冻结结构清单、工作流顺序、持久化对象和字段变更规则。
- C-P0-1：将变更验证命令固定为 `cargo check`、`cargo test` 和 `scripts/mvp_demo.ps1`，真实模型验收使用 `-UseRealModel`。

验证结果：

```text
cargo test
4 passed; 0 failed
```

当前状态：

- MVP 接口冻结规则已落文档。

## 16. 开发者 B 第七轮处理记录

处理日期：2026-06-09

已处理：

- B-P0-1 / B-P1-2：`PromptAgent` 现在会严格校验 AgentOutput envelope，合法 JSON 也必须包含匹配的 `role`、对象型 `structured` 和字符串型 `raw_notes`。
- B-P1-2：缺少 envelope、角色不匹配或 `structured` 类型不对时，会写入 `parse_error` 并触发既有重试/fallback 逻辑，避免裸 JSON 被误判为真实模型成功输出。
- B-P1-2：`tests/smoke.rs` 增加合法 JSON 但缺少 AgentOutput envelope 的回归用例，确认该类输出会被标记为 parse error。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
5 passed; 0 failed
```

当前状态：

- 真实模型链路的输出 envelope 已进入严格解析和自动回归覆盖。

## 17. 开发者 A 第八轮复核记录

处理日期：2026-06-09

已处理：

- A/B 接口复核：在 B 侧收紧 `AgentOutput` envelope 解析后，重新执行 A 侧冻结验收命令。
- C-P0-1：`docs/INTERFACE_FREEZE.md` 的当前验收状态已补充严格 envelope 校验覆盖项。
- C-P0-2：确认严格解析没有破坏离线 MVP CLI 闭环，fallback 路径仍能完成 `new -> outline -> write -> review -> rewrite -> export`。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
5 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
new / outline / write / review / rewrite / export all ok
export_size=15976
```

当前状态：

- MVP 冻结接口、严格输出解析、离线验收闭环已完成交叉复核。

## 18. 开发者 B 第九轮处理记录

处理日期：2026-06-09

已处理：

- B-P1-2：`run_prompt_agent` 的重试提示现在会携带上一轮 `parse_error`，让模型能针对 JSON 语法、envelope 缺失、角色不匹配等具体原因修复输出。
- B-P1-2：重试提示仍只要求修复 JSON 格式和 AgentOutput envelope，不把完整坏响应塞回 prompt，避免扩大上下文和复制错误内容。
- B-P1-2：`tests/smoke.rs` 增加重试修复用例，模型第一次返回缺少 envelope 的合法 JSON，第二次只有看到“上一次解析错误”才返回合法 AgentOutput，确认错误原因确实进入 retry prompt。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
6 passed; 0 failed
```

当前状态：

- 模型输出失败后的重试链路已能带着具体解析原因进行修复。

## 19. 开发者 A 第九轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：`ModelResponse` 增加可选 `usage`，用于后续接入支持 token 用量的模型 provider。
- A-P2：`AgentOutput` 增加 `duration_ms` 和 `token_usage`，`run_prompt_agent` 对每次 Agent 尝试计时。
- A-P2：`agent_runs.structured._engineering` 已保存 `duration_ms` 和 `token_usage`，当前 Rig/OpenAI 路径先记录 `token_usage: null`。
- A-P2：`tests/smoke.rs` 增加工程元数据落库断言，确认 AgentRun 至少包含耗时字段和 token usage 预留字段。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
6 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
new / outline / write / review / rewrite / export all ok
export_size=15976
```

当前状态：

- Agent 执行耗时已进入持久化记录，token 统计接口已预留到模型响应和 AgentRun 元数据。

## 20. 开发者 B 第十轮处理记录

处理日期：2026-06-09

已处理：

- B-P0-1 / B-P1-2：`AgentOutput.structured` 现在会按 Agent 角色做最小 schema 校验，合法 envelope 也必须包含该 Agent 的关键顶层字段。
- B-P1-2：当前最小校验覆盖 `market_analysis/title_candidates/opening_strategy/platform_profile`、`plot_plan/chapter_outlines`、`characters`、`world_setting/facts_to_seed`、`chapter_draft`、`continuity_report`、`styled_chapter`、`review_report`。
- B-P1-2：缺少必需字段或字段类型不对时会写入 `parse_error`，进入既有重试/fallback 链路。
- B-P1-2：`tests/smoke.rs` 增加 “envelope 正确但 structured 缺必需字段” 的回归用例，确认内部 schema 缺口会被识别。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
7 passed; 0 failed
```

当前状态：

- Agent 输出已完成 envelope 严格解析和按角色的最小 `structured` schema 校验。

## 21. 开发者 B 第十一路处理记录

处理日期：2026-06-09

已处理：

- 真实模型链路 preflight：当前环境未设置 `OPENAI_API_KEY`，无法实际执行 OpenAI 真实模型端到端验收。
- C-P0-2：`scripts/mvp_demo.ps1 -UseRealModel` 现在会在缺少 `OPENAI_API_KEY` 时提前失败，避免真实模型验收悄悄退回 smoke fallback。
- C-P0-2：`scripts/mvp_demo.ps1` 会检测 CLI 输出中的 fallback / Agent 调用失败 / 解析失败提示；在 `-UseRealModel` 模式下观察到这些提示会将验收判定为失败。
- C-P0-1：`docs/INTERFACE_FREEZE.md` 已补充真实模型验收要求：必须有 `OPENAI_API_KEY`，且真实模式不得出现 fallback 输出。

验证结果：

```text
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -UseRealModel
expected missing OPENAI_API_KEY failure observed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
new / outline / write / review / rewrite / export all ok

cargo test
7 passed; 0 failed
```

当前状态：

- 真实模型验收尚未实际调用 API；已补齐缺 key 和 fallback 误通过的防护。设置 `OPENAI_API_KEY` 后可重新执行 `scripts/mvp_demo.ps1 -UseRealModel`。

## 22. 开发者 A 第十轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：新增 `SmokeModelClient`，支持 `provider = "smoke"` 的本地确定性模型 provider。
- A-P2：`ModelProvider::parse` 支持 `smoke`、`local`、`offline` 别名，CLI 会按配置在 `openai` 与 `smoke` provider 之间切换。
- A-P2：`SmokeModelClient` 对 Market、Plot、Character、Worldbuilding、Writer、Continuity、Style、Reviewer 返回合法 `AgentOutput` envelope，可用于离线 demo 和 CI，不再依赖“OpenAI 缺 key 后 fallback”来跑通演示。
- A-P2：`PromptAgent` 已把 `ModelResponse.usage` 传入 `AgentOutput.token_usage`，本地 smoke provider 会写入粗略 token usage，OpenAI/Rig 路径继续保留 `null`。
- A-P1 / C-P0-2：`scripts/mvp_demo.ps1` 默认使用 `provider = "smoke"`；`-Provider openai` 保留离线 fallback 验证；`-UseRealModel` 仍强制走 OpenAI 且执行 key/fallback preflight。
- A-P2：`tests/smoke.rs` 增加 provider 解析测试和本地 smoke provider 非 fallback 工作流测试。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
9 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
provider=smoke
new / outline / write / review / rewrite / export all ok
export_size=10132

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai
openai fallback branch ok
export_size=15976
```

当前状态：

- A-P2 的模型 provider 切换已落地；默认离线演示走本地合法 Agent 输出，OpenAI fallback 与真实模型 preflight 分支仍保留。

## 23. 开发者 B 第十二轮处理记录

处理日期：2026-06-09

已处理：

- 真实模型链路扩展：`ModelProvider` 新增 `deepseek`，CLI 配置可使用 `provider = "deepseek"`。
- 真实模型链路扩展：`RigModelClient` 接入 `rig_core::providers::deepseek`，使用 `DEEPSEEK_API_KEY` 和 Rig DeepSeek provider 调用模型。
- C-P0-2：`scripts/mvp_demo.ps1` 支持 `-Provider deepseek`，默认模型为 `deepseek-v4-flash`；`-UseRealModel` 模式按 provider 检查 `OPENAI_API_KEY` 或 `DEEPSEEK_API_KEY`。
- C-P0-1：`docs/INTERFACE_FREEZE.md` 已补充 OpenAI / DeepSeek 两套真实模型验收命令和 key 要求。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
9 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
provider=smoke
new / outline / write / review / rewrite / export all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek
deepseek fallback branch ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel
expected missing DEEPSEEK_API_KEY failure observed in current process
```

当前状态：

- DeepSeek provider 已接入工程和 demo 脚本；当前 Codex 进程仍未继承到 `DEEPSEEK_API_KEY`，所以尚未实际调用 DeepSeek API。让该环境变量对当前进程可见后，可执行 `scripts/mvp_demo.ps1 -Provider deepseek -UseRealModel` 做真实验收。

## 24. 开发者 A 第十一路处理记录

处理日期：2026-06-09

已处理：

- A-P2：`ModelClient` 增加 `complete_stream` 默认方法，并新增 `ModelStreamResponse` / `ModelStreamChunk`，所有 provider 都可通过统一接口返回流式 chunks。
- A-P2：当前 Rig/OpenAI/DeepSeek 路径暂用完整响应拆块作为 fallback stream；`smoke` provider 同样可通过该接口稳定返回 chunks，后续可替换为 Rig 原生 streaming 实现。
- A-P2：为避免半截 JSON 污染 Agent workflow，结构化保存仍等待完整 `AgentOutput` 解析成功后执行。
- A-P2：CLI `write` 和 `rewrite` 增加 `--stream`，对已生成章节正文做分块输出，适合演示和手动验收。
- A-P1 / C-P0-2：`scripts/mvp_demo.ps1` 增加 `-StreamWrite`，会在 `write` 和 `rewrite` 阶段启用 `--stream`。
- A-P2：`tests/smoke.rs` 增加本地 smoke provider 流式 chunks 回归测试，确认 chunks 可拼回完整响应且保留 token usage。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
10 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
provider=smoke
new / outline / write / review / rewrite / export all ok
export_size=10132

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
export_size=10132
```

当前状态：

- A-P2 的流式输出能力已落地到模型接口、CLI 参数、demo 脚本和自动测试；真实 provider 原生 streaming 可作为后续增强替换底层实现。

## 25. 开发者 A 第十二轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：`AgentRunRepository` 增加 `list_recent`，支持按 `novel_id` 过滤最近运行记录。
- A-P2：新增 `AgentRunRecord` 只读结构，包含 role、task、structured、raw_text、raw_notes、parse_error 和 created_at。
- A-P2：CLI 新增 `runs` 命令，可输出最近 Agent 运行记录的角色、任务、小说 ID、尝试次数、状态、耗时和 token。
- A-P2：`scripts/mvp_demo.ps1` 已把 `runs --novel-id <id> --limit 5` 纳入默认演示闭环。
- A-P2：`tests/smoke.rs` 增加 AgentRun 查询断言，确认记录能按小说过滤，并包含 market/reviewer 等关键角色和 `_engineering` 元数据。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
10 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
provider=smoke
new / outline / write / review / rewrite / export / runs all ok
export_size=10132
```

当前状态：

- A-P2 的任务运行记录已从“仅落库”推进到 storage 查询、CLI 可视化、demo 验收和自动测试覆盖。

## 26. 开发者 A 第十三轮处理记录

处理日期：2026-06-09

已处理：

- A 侧交付收口：新增 `docs/MVP_ACCEPTANCE.md`，作为 CLI MVP 的交付验收入口。
- C-P0-2：验收说明集中列出离线工程验收命令、流式演示命令、OpenAI/DeepSeek 真实模型验收命令和通过口径。
- C-P0-2：明确当前环境尚未实际调用真实模型 API；真实 provider 已接入，缺 key 或出现 fallback/解析失败时不得判定为真实验收通过。
- A/B 协作：README 已挂载 `docs/MVP_ACCEPTANCE.md`，并将当前测试期望更新为 `cargo test 11 passed`。

验证结果：

```text
文档收口，无代码变更。
沿用上一轮已通过结果：
cargo check ok
cargo test 11 passed; 0 failed
scripts/mvp_demo.ps1 ok
scripts/mvp_demo.ps1 -StreamWrite ok
```

当前状态：

- MVP 离线工程验收已有独立交付说明；后续真实模型验收只需在 key 可见环境运行对应 `-UseRealModel` 命令。

## 27. 开发者 A 第十四轮处理记录

处理日期：2026-06-09

已处理：

- A 侧提交前复核：发现 `novel-agent.toml.example` 仍默认使用 `openai/gpt-5`，与当前默认离线 smoke 验收口径不一致。
- C-P0-2：将 `novel-agent.toml.example` 默认 provider 改为 `smoke`，保证新环境不需要 API key 即可跑本地工程验收。
- C-P0-2：在模板注释中保留 OpenAI 和 DeepSeek 真实 provider 示例，避免真实模型配置入口丢失。

验证结果：

```text
git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- 配置模板已与 README、MVP 验收说明和 demo 默认行为对齐。

## 28. 开发者 A 第十五轮处理记录

处理日期：2026-06-09

已处理：

- A 侧提交前最终复核：按核心代码、CLI/demo/tests、文档验收三组查看 diff，确认可以拆分提交。
- A 侧提交前最终复核：确认 `.gitignore` 已覆盖 `target/`、SQLite 数据库、`exports/` 和本地 `novel-agent.toml`，当前未跟踪文件均为应纳入交付的新文件。
- C-P0-2：重新执行最终工程验收，确认当前测试集为 11 个回归用例，默认 demo 和流式 demo 均可跑通。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
11 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
new / outline / write / review / rewrite / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed; runs all ok

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- 提交前复核完成；下一步可按功能拆分提交或由负责人统一提交。

## 29. 开发者 A 第十六轮处理记录

处理日期：2026-06-09

已处理：

- A-P1 / C-P0-2：CLI 新增 `versions` 命令，可列出章节版本快照。
- A-P1 / C-P0-2：`versions --show <v>` 可输出指定版本正文，`versions --from <a> --to <b>` 可输出基础版本对比。
- A-P1 / C-P0-2：版本对比包含 v1/v2 字数、字数变化、共同前缀字符和正文预览。
- A-P1 / C-P0-2：`scripts/mvp_demo.ps1` 已在 rewrite 后执行 `versions --from 1 --to 2`，演示闭环现在覆盖版本对比。
- A/B 协作：README、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md` 已补充 `versions` 命令和验收口径。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
11 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
new / outline / write / review / rewrite / versions / export / runs all ok
versions: v1/v2 compare observed
```

当前状态：

- 章节版本快照已从“保存和 repository 测试”推进到 CLI 可查看、可对比、demo 可验收。

## 30. 开发者 A 第十七轮处理记录

处理日期：2026-06-09

已处理：

- A-P1 / C-P0-2：CLI 新增 `edit` 命令，可通过 `--input <path>` 读取人工编辑稿并保存为章节新版本。
- A-P1 / C-P0-2：新增 `ChapterGenerationWorkflow::save_manual_edit`，人工稿复用 `ChapterDraft` / `chapter_versions` 版本通道，不额外调用 Agent。
- A-P1：保存新草稿时清空旧审稿分数，避免人工 v3 仍挂载 v2 的历史评分；人工稿保存后状态回到 `Drafted`，可继续执行 `review`。
- A-P1 / C-P0-2：`scripts/mvp_demo.ps1` 已在 rewrite 后写入无 BOM 的人工编辑文件，执行 `edit`，并用 `versions --from 2 --to 3` 验证人工版本对比。
- A/B 协作：README、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md` 已补充人工编辑版本链路和验收口径。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
12 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
new / outline / write / review / rewrite / versions / edit / versions / export / runs all ok
versions: v2/v3 manual edit compare observed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed; edit and v2/v3 compare observed

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- 人工介入改稿已进入 MVP 闭环：生成或返工后的章节可由人工编辑保存为 vN，再通过 `versions` 对比、`review` 复审和 `export` 导出。

## 31. 开发者 B DeepSeek 真实链路处理记录

处理日期：2026-06-09

已处理：

- 真实模型环境复核：当前进程已可读取 `DEEPSEEK_API_KEY`。
- DeepSeek 实测问题定位：最新 `deepseek-chat` demo 库中，Plot Agent 在 30 章大纲输出时多次出现 `EOF while parsing a string`，属于长 JSON 输出截断；`new` 最终依赖 fallback 大纲保存了 30 章，不能判定为真实模型验收通过。
- B-P1 / C-P0-2：CLI `new` 新增 `--chapters` 参数，默认仍为 30；真实模型短链路可先生成较少章节，降低 Plot Agent 长 JSON 截断风险。
- B-P1 / C-P0-2：`NovelCreationWorkflow` 新增 `create_from_idea_with_chapters`，默认 `create_from_idea` 保持 30 章行为。
- B-P1 / C-P0-2：`scripts/mvp_demo.ps1` 新增 `-NewChapters`、`-OutlineChapters`、`-SkipOutline`、`-SkipRewrite`，默认完整 demo 不变；真实 provider 可先跑短链路。
- B-P1 / C-P0-2：DeepSeek demo 默认模型调整为 `deepseek-chat`，避免继续默认使用本轮实测不稳定的 `deepseek-v4-flash`。
- B-P1：新增回归测试，确认创建新书时可限制初始大纲章节数，并只保存对应数量的章节。
- A/B 协作：README、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md` 已补充短链路验收命令和通过口径。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
12 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
new / outline / write / review / rewrite / versions / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel -NewChapters 6 -SkipOutline -SkipRewrite
real DeepSeek short path ok
agent_runs.parse_error = 0
agent roles ok: market, plot, character, worldbuilding, writer, continuity, style, reviewer
saved chapters = 6

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- DeepSeek key 已可见；短链路真实 API 验证通过。完整 30 章真实模型验收仍需继续压测或拆分 Plot Agent 输出，避免长 JSON 截断。

## 32. 开发者 A 第十八轮处理记录

处理日期：2026-06-09

已处理：

- A-P1 / C-P0-2：`NovelCreationWorkflow` 将 Plot Agent 大纲生成改为分批调用，默认批次大小为 10；demo 默认用 6 章一批，降低真实模型长 JSON 截断风险。
- A-P1 / C-P0-2：`new --outline-batch-size` 和 `outline --batch-size` 已接入 CLI；`scripts/mvp_demo.ps1` 已接入 `-NewOutlineBatchSize` 与 `-OutlineBatchSize`。
- A-P1：分批合并支持区间过滤、绝对章号矫正和缺章 fallback；如果模型在 11-20 批次仍从 1 重新编号，工程侧会按批次 offset 归一化为全书章号。
- A-P1：`SmokeModelClient` 已理解 `chapter_start` / `chapter_end`，离线 provider 和真实 provider 使用同一批次 payload 语义。
- B 协作：`prompts/plot_agent.md` 与 `docs/SCHEMAS.md` 已补充 Plot 批次输入约定；README、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md` 已更新验收口径。
- B-P1：新增/更新回归测试，确认短大纲仍只跑一批，13 章按 6 章一批可合并为连续 1-13 章，默认 30 章 smoke 链路无 fallback。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
13 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
new / outline / write / review / rewrite / versions / edit / versions / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed; edit and v2/v3 compare observed

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- Plot Agent 长 JSON 截断问题已完成工程侧拆分修复；本轮未重新调用真实 DeepSeek 完整 30 章链路，后续真实验收应运行默认 `scripts/mvp_demo.ps1 -Provider deepseek -UseRealModel` 并确认 `agent_runs.parse_error = 0`。

## 33. 开发者 A 第十九轮处理记录

处理日期：2026-06-09

已处理：

- C-P0-2：`scripts/mvp_demo.ps1` 新增 `-RunsLimit`，默认检查最近 80 条 AgentRun，避免早期 Market/Plot 批次错误被原来的 `runs --limit 5` 漏掉。
- C-P0-2：真实模型模式下，demo 会扫描 `runs` 输出；发现 `status=fallback` 或 `status=parse_error` 时直接失败。
- C-P0-2：默认 demo 结果现在输出 `runs_limit=<n>`，便于确认本次验收覆盖范围。
- A/B 协作：README、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md` 已补充 AgentRun 覆盖检查口径。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
13 passed; 0 failed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
new / outline / write / review / rewrite / versions / edit / versions / export / runs all ok
runs_limit=80; all listed AgentRun statuses ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed; runs_limit=80; all listed AgentRun statuses ok

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- 真实模型验收防线从“观察终端 fallback 文本 + 最近 5 条 runs”提升为“终端文本 + 最近 80 条 AgentRun 状态检查”。完整真实链路仍需在 key 可见且允许外网/API 调用的环境重跑。

## 34. 开发者 A 第二十轮处理记录

处理日期：2026-06-09

已处理：

- C-P0-2：`AgentRunRecord` 新增统一状态判断，按 `parse_error` 优先、其次 `_engineering.will_fallback` 的口径输出 `ok` / `fallback` / `parse_error`。
- C-P0-2：`runs` CLI 新增 `--summary`，输出 `agent_run_summary total=<n> ok=<n> fallback=<n> parse_error=<n>`。
- C-P0-2：`runs` CLI 新增 `--fail-on-bad-status`，当最近记录中存在 fallback 或 parse_error 时返回非零退出码。
- C-P0-2：`scripts/mvp_demo.ps1` 改为在真实模型模式下调用 `runs --summary --fail-on-bad-status`，不再依赖 PowerShell 正则扫描 `runs` 行判定坏状态。
- C-P0-2：修正 demo 旧文本检测的误报，避免 `fallback=0` / `parse_error=0` 摘要被当成失败提示。
- A/B 协作：README、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md` 已同步新的 AgentRun 状态检查命令和验收口径。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
storage unit: 1 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=21 ok=21 fallback=0 parse_error=0

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=21 ok=21 fallback=0 parse_error=0

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- AgentRun 验收状态已经从脚本文字扫描下沉到 Rust CLI 退出码；真实完整链路仍需在 key 可见且允许外网/API 调用的环境重跑。

## 35. 开发者 A 第二十一轮处理记录

处理日期：2026-06-09

已处理：

- A-P0 / Web P0：新增 `src/api.rs`，接入 axum，提供本地 REST API router。
- A-P0：CLI 新增 `serve --bind <addr>`，默认可用 `cargo run -- serve --bind 127.0.0.1:3001` 启动 API 服务。
- A-P0：新增作品 API：`GET /api/novels`、`POST /api/novels`、`GET /api/novels/{novel_id}`。
- A-P0：新增大纲 API：`POST /api/novels/{novel_id}/outline`。
- A-P0：新增章节 API：章节列表、章节详情、写章节、审稿、最新审稿报告、重写。
- A-P0：新增版本 API：版本列表和指定版本正文。
- A-P0：新增 AgentRun API：`GET /api/novels/{novel_id}/runs?limit=<n>`，复用 CLI 同一套 `ok` / `fallback` / `parse_error` 状态和 summary 口径。
- A-P0：`NovelRepository` 新增 `list_recent`，支撑作品列表 API。
- A-P0：新增 `docs/API.md`，给 C 侧提供 OpenAPI 风格接口说明；README、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 已同步 API 入口和状态。
- A-P1：新增 API smoke test，覆盖新建作品、作品列表、章节生成、审稿和 AgentRun 查询。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 2 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- Web 工作台 P0 API 骨架已落地，C 侧可以开始按 `docs/API.md` 对接作品、章节、审稿、版本和 AgentRun。后续 A 侧可继续补导出 API、CORS 配置和 SSE 流式接口。

## 36. 开发者 A 第二十二轮处理记录

处理日期：2026-06-09

已处理：

- A-P1：`ChapterGenerationWorkflow` 新增 `export_markdown_content`，CLI 文件导出和 API 内容导出共用同一份 Markdown 渲染逻辑。
- A-P1：新增导出 API：`GET /api/novels/{novel_id}/export/markdown`，返回 `format`、`filename` 和完整 Markdown 内容。
- A-P1：新增章节生成 SSE：`POST /api/novels/{novel_id}/chapters/{chapter_index}/write/stream`。
- A-P1：新增重写 SSE：`POST /api/novels/{novel_id}/chapters/{chapter_index}/rewrite/stream`。
- A-P1：SSE 事件名固定为 `started`、`chapter_chunk`、`completed`；`chapter_chunk` data 包含 `operation`、`chapter_index`、`chunk_index`、`text`。
- A-P1：API router 接入 permissive CORS，支持 Web 工作台本地跨端口开发。
- A-P1：API smoke test 扩展到 CORS preflight、SSE 响应和 Markdown 导出。
- A/C 协作：`docs/API.md`、README、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 已同步导出、SSE 和 CORS 口径。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 2 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- Web 工作台 P1 API 能力已补齐导出、SSE 和 CORS；C 侧可以开始接入章节生成进度流、Markdown 导出入口和本地跨端口调试。

## 37. 开发者 A 第二十三轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：`AgentRunRecord` 新增统一的 `attempt`、`duration_ms`、`prompt_tokens`、`completion_tokens`、`total_tokens` 读取方法，避免 CLI 和 API 重复解析 `_engineering`。
- A-P2：`AgentRunStatusSummary` 新增 `duration_ms_total`、`tokenized_runs`、`prompt_tokens`、`completion_tokens`、`total_tokens` 汇总字段。
- A-P2：`runs --summary` 输出新增耗时和 token 汇总，运行面板可直接读取 CLI 结果做人工验收。
- A-P2：AgentRun API summary 同步返回耗时和 token 汇总；单条 run 继续返回 `attempt`、`duration_ms`、`total_tokens`。
- A-P2：storage unit test 和 API smoke test 已覆盖 token 汇总字段。
- A/C 协作：`docs/API.md`、README、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 已同步 AgentRun 汇总口径。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 2 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23 prompt_tokens=53120 completion_tokens=17729 total_tokens=70849

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23 prompt_tokens=53120 completion_tokens=17729 total_tokens=70849

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- AgentRun 已具备状态、耗时和 token 汇总，足够支撑 C 侧运行面板的 P1 展示；真实 cost 统计仍需后续补 provider 价格配置后再计算。

## 38. 开发者 A 第二十四轮处理记录

处理日期：2026-06-09

已处理：

- A-P1：新增 `scripts/api_demo.ps1`，使用临时配置和临时 SQLite 启动本地 `serve`，自动调用 CORS、作品创建、作品列表、章节生成、审稿、SSE、Markdown 导出和 AgentRun 查询。
- A-P1：API demo 支持 `smoke` / `openai` / `deepseek` provider 参数；真实模式沿用 key preflight，默认离线 smoke 不访问网络。
- A-P1：API demo 会自动寻找空闲端口、等待 `/health` 就绪，并在结束时停止临时 API 进程。
- A-P1：修复 `draft_from_agent_output` 身份字段信任模型输出的问题；`chapter_id`、`novel_id`、`volume_index`、`chapter_index` 现在以 storage 中的目标章节为准，避免模型或 smoke provider 返回 `chapter_index=1` 时覆盖用户请求的第 2 章。
- A-P1：API smoke test 增加 `write/stream` 的 `chapter_index=2` 断言，防止章节身份回归。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 已同步 API demo 命令和验收口径。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 2 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / write / review / SSE / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23 prompt_tokens=53110 completion_tokens=17729 total_tokens=70839

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23 prompt_tokens=53120 completion_tokens=17729 total_tokens=70849
```

当前状态：

- API demo 配置已落地，C 侧可用单条脚本验证本地后端接口；章节生成 workflow 已防止模型输出覆盖目标章节身份。

## 39. 开发者 B 真实模型与 xhigh 处理记录

处理日期：2026-06-09

已处理：

- B-P1 / C-P0-2：DeepSeek 完整真实链路已重新压测通过；默认 30 章新建、重复 outline、write、review、rewrite、versions、edit、export、runs 全流程返回 `agent_run_summary total=23 ok=23 fallback=0 parse_error=0`。
- B-P1 / C-P0-2：确认本地 `cliproxyapi` 端口 `127.0.0.1:8317` 可用，`/v1/models` 返回 `gpt-5.4`、`gpt-5.4-mini`、`gpt-5.5` 等模型；`chat/completions` 可正常返回。
- B-P1 / C-P0-2：确认 `gpt-5.5` 支持 `reasoning_effort = "xhigh"` 和 `reasoning = { effort = "xhigh" }` 两种 OpenAI-compatible 参数形式。
- B-P1：`ModelConfig` 新增可选 `reasoning_effort`；`RigModelClient` 会把该字段透传到 OpenAI-compatible provider 的 additional params。
- B-P1：`scripts/mvp_demo.ps1` 新增 `-ReasoningEffort` 参数，可直接运行 `-Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel`。
- B-P1：修复 `scripts/mvp_demo.ps1` 临时 TOML 拼接问题，改为按行写入配置，避免 `model` 与 `reasoning_effort` 连在同一行导致 TOML parse error。
- B-P1：真实模型调用超时放宽，结构类 Agent 为 240 秒，Plot/Writer/Style 为 300 秒，避免 `gpt-5.5 + xhigh` 在 Character Agent 阶段被过早中断。
- B-P1：`novel-agent.toml.example`、README、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md` 已同步 OpenAI-compatible 代理和 `gpt-5.5 + xhigh` 用法。
- B-P1：`docs/SCHEMAS.md` 已补充 Worldbuilding Agent 输入 `scope` 约束；`prompts/worldbuilding_agent.md` 已限制 organizations、locations 和 seed facts 的规模，降低真实模型百科式展开风险。
- A/B 协作：修复 `src/api.rs` 缺少 `JobsResponse` / `JobResponse` 导致 `cargo test` 编译失败的问题，并统一 API 默认 Plot batch 为 5。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 2 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel -StepRetries 1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=613501

cliproxyapi /v1/models
models_ok=true; model_count=5; includes gpt-5.5

cliproxyapi chat/completions gpt-5.5 reasoning_effort=xhigh
reply=OK

cargo run -- --config <temp> new ... --chapters 1 --outline-batch-size 1
provider=openai; model=gpt-5.5; reasoning_effort=xhigh
exit_code=0
agent_runs=4; fallback=0; parse_error=0; duration_ms_total=406814

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -SkipOutline -SkipRewrite
agent_run_summary total=13 ok=13 fallback=0 parse_error=0

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -ReasoningEffort xhigh -SkipOutline -SkipRewrite
agent_run_summary total=13 ok=13 fallback=0 parse_error=0

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- 真实 DeepSeek 完整链路已从“待重新压测”变为“通过”；本地 OpenAI-compatible 代理可作为 `openai` provider 使用，当前推荐命令为 `-Model gpt-5.5 -ReasoningEffort xhigh`。

## 39. 开发者 A 第二十五轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：新增进程内后台任务队列，API 可创建 `create_novel`、`write_chapter`、`review_chapter`、`rewrite_chapter` 异步任务。
- A-P2：新增 `GET /api/jobs?limit=<n>` 和 `GET /api/jobs/{job_id}`，任务状态固定为 `queued`、`running`、`succeeded`、`failed`。
- A-P2：任务成功后的 `result` 复用同步接口同形 DTO，例如写作和重写返回 `{ "draft": {} }`，审稿返回 `{ "report": {} }`。
- A-P2：API smoke test 覆盖 `POST /write/jobs`、任务轮询成功和 job 列表查询。
- A-P2：`scripts/api_demo.ps1` 已覆盖后台任务创建、轮询、结果章节身份检查和 job 列表查询。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 已同步后台任务接口和 MVP in-process 边界。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 2 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / write / review / SSE / jobs / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23 prompt_tokens=53120 completion_tokens=17729 total_tokens=70849

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23 prompt_tokens=53120 completion_tokens=17729 total_tokens=70849

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- 后台任务队列已完成 MVP 级 API 暴露和脚本验收；当前实现为单进程内存队列，足够支持 Web 工作台 P1 的非阻塞生成状态，后续若要跨进程或重启恢复，应升级为 SQLite 持久化 job 表。

## 40. 开发者 A 第二十六轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：后台任务记录从进程内 `HashMap` 下沉到 SQLite `api_jobs` 表。
- A-P2：新增 `JobRepository`、`JobRecord`、`JobStatus`，支持创建任务、查询单个任务、查询最近任务和更新 `queued/running/succeeded/failed` 状态。
- A-P2：`src/api.rs` 移除内存 `JobStore`，`POST /jobs` 创建后立即落库，后台 `tokio::spawn` 写回任务状态、结果或错误。
- A-P2：API smoke test 增加“重建 router 后仍可查询已完成 job”的断言，覆盖服务状态刷新后的任务记录可见性。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 已同步 SQLite job 记录边界。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 2 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / write / review / SSE / jobs / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- 后台任务已具备 SQLite 历史记录能力，完成/失败任务可在服务重启后查询；MVP 暂不恢复进程退出时仍在运行的任务，后续可补 job recovery 或 worker loop。

## 41. 开发者 A 第二十七轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：`api_jobs` 新增 `payload` 字段，保存创建任务时的请求参数，便于 Web 工作台展示任务上下文和做重试预填。
- A-P2：`SqliteStorage::migrate` 增加旧表兼容逻辑；已有 `api_jobs` 表缺少 `payload` 列时会自动补列。
- A-P2：`serve` 启动前会把上次进程遗留的 `queued` / `running` 任务标记为 `failed`，避免 UI 永久等待不会再执行的任务。
- A-P2：Storage 单元测试覆盖旧 `api_jobs` 表补 `payload`、任务 payload 持久化，以及遗留未完成任务收口为 failed。
- A-P2：API smoke test 和 `scripts/api_demo.ps1` 已覆盖 job payload 的 `chapter_index`。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 已同步 job payload 和遗留任务收口规则。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / write / review / SSE / jobs(payload) / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- 后台任务已具备持久化 payload 和中断收口能力；下一步若继续增强，可基于 payload 实现显式 `POST /api/jobs/{job_id}/retry` 或真正的 worker recovery。

## 42. 开发者 A 第二十八轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：新增 `POST /api/jobs/{job_id}/retry`，支持基于 failed 源任务的 `payload` 创建新的重试 job。
- A-P2：retry 不覆盖原任务记录，便于 Web 工作台展示失败历史和新重试任务。
- A-P2：重试逻辑复用 create/write/review/rewrite job 的同一后台执行路径，避免普通创建和重试创建行为漂移。
- A-P2：API smoke test 覆盖 failed `write_chapter` job retry 创建新 job 并执行成功；非 failed job retry 返回 `400 Bad Request`。
- A-P2：`scripts/api_demo.ps1` 覆盖 completed job retry 被拒绝，防止 UI 误把已完成任务重复提交。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 已同步 retry 接口和边界。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / write / review / SSE / jobs / retry-completed-400 / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- Web 工作台现在可以对失败任务做一键重试；下一步可继续补批量章节 job、job 取消接口，或把 retry 关系字段化为 `source_job_id`。

## 43. 开发者 A 第二十九轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：`api_jobs` 和 `JobRecord` 增加 `source_job_id`，普通任务为 `null`，retry 新任务指向源失败任务。
- A-P2：`POST /api/jobs/{job_id}/retry` 现在通过 `create_with_source` 创建新 job，保留原失败任务并显式记录来源关系。
- A-P2：`SqliteStorage::migrate` 增加旧表兼容逻辑；已有 `api_jobs` 表缺少 `payload` 或 `source_job_id` 列时会自动补列，并在补列后创建来源索引。
- A-P2：Storage 单元测试覆盖旧表补 `payload` / `source_job_id`、来源关系持久化，以及遗留 `queued` / `running` 任务收口为 `failed`。
- A-P2：API smoke test 覆盖 failed `write_chapter` job retry 创建带 `source_job_id` 的新 job 并执行成功。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 和 `scripts/api_demo.ps1` 已同步 `source_job_id` 字段语义。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / write / review / SSE / jobs(source_job_id) / retry-completed-400 / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- Web 工作台可以直接用 `source_job_id` 串起失败任务和重试任务；下一步可继续补 job 取消接口或批量章节 job。

## 44. 开发者 A 第三十轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：新增 `POST /api/jobs/{job_id}/cancel`，支持取消 `queued` / `running` 后台任务。
- A-P2：`JobStatus` 增加终态 `cancelled`，取消后保留 `payload` / `source_job_id`，`result` 清空，`error` 写入取消原因。
- A-P2：后台 worker 的 `set_running` / `complete` / `fail` 写回现在只允许从未完成态推进，避免 cancelled 终态被后续完成或失败结果覆盖。
- A-P2：API smoke test 覆盖 queued job cancel 成功、二次 cancel 返回 `400 Bad Request`、completed job cancel 返回 `400 Bad Request`。
- A-P2：Storage 单元测试覆盖 cancelled job 不会被 `set_running` / `complete` 覆盖。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 和 `scripts/api_demo.ps1` 已同步 cancel 接口和 `cancelled` 状态。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / write / review / SSE / jobs(source_job_id) / retry-completed-400 / cancel-completed-400 / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- Web 工作台现在可以对未完成任务执行取消；下一步可继续补批量章节 job。

## 45. 开发者 B xhigh 与 SQLite 迁移回归处理记录

处理日期：2026-06-09

已处理：

- B-P1：修复旧 `api_jobs` 表迁移顺序问题；`idx_api_jobs_source_job_id` 现在会在补齐 `source_job_id` 列之后创建，避免旧库迁移时报 `no such column: source_job_id`。
- B-P1：Storage 回归测试同步断言遗留 `running` 源 job 和 `queued` retry job 都会被 `fail_incomplete` 标记为 `failed`，与 `serve` 重启收口语义一致。
- B-P1 / C-P0-2：重新验证本地 OpenAI-compatible `cliproxyapi` 的 `gpt-5.5 + reasoning_effort=xhigh` 快速真实链路。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 13 passed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 2 -OutlineChapters 2 -NewOutlineBatchSize 1 -OutlineBatchSize 1 -SkipOutline -SkipRewrite
最终 agent_run_summary total=10 ok=10 fallback=0 parse_error=0 duration_ms_total=884815
export_size=14132
```

备注：

- 本次 `xhigh` 快速链路中 `write` 步骤首次子命令返回 `exit code -1`，demo 脚本自动重试后成功；落库 AgentRun 全部为 `ok`，未出现 fallback 或 parse error。
- 这次结果可确认 provider、AgentOutput 解析、SQLite 迁移和 AgentRun 汇总链路可用；后续第 46 轮已完成 `-StepRetries 0` 的严格短链路验收。

## 46. 开发者 B xhigh 无重试验收补强记录

处理日期：2026-06-09

已处理：

- B-P1：AgentOutput 解析增强为“优先 fenced JSON；否则从首个 `{` 做括号配平，提取完整 JSON 对象”，可接受真实模型直接输出 JSON 后追加说明文本的情况。
- B-P1：新增回归测试 `direct_agent_output_json_with_trailing_text_is_parsed`，确认非 fenced JSON 后带尾巴说明不会再产生 parse error，同时既有缺 envelope / 缺 required field 测试仍保持失败。
- B-P1：`Character create_novel` 模型调用 timeout 从 240 秒放宽到 360 秒；本地 `gpt-5.5 + xhigh` 曾出现 240 秒边缘超时，放宽后可减少真实高推理模式下的误失败。
- C-P0-2：`scripts/mvp_demo.ps1` 在 `-UseRealModel` 时会给 `runs` 加 `--fail-on-bad-status`，并补充 `status=parse_error` / `parse_error=<非零>` 文本检测，避免真实验收中 `parse_error=1` 被误判通过。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 14 passed

cargo run --quiet -- --config <bad-xhigh-temp-config> runs --novel-id <bad-xhigh-novel-id> --limit 80 --summary --fail-on-bad-status
expected failure observed: AgentRun status check failed: fallback=0, parse_error=1 in listed 10 runs

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 2 -OutlineChapters 2 -NewOutlineBatchSize 1 -OutlineBatchSize 1 -SkipOutline -SkipRewrite -StepRetries 0
agent_run_summary total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=799126
export_size=15722
```

当前状态：

- `gpt-5.5 + reasoning_effort=xhigh` 已完成 2 章快速真实链路无重试验收，且真实 demo 现在会严格拒绝 fallback / parse_error。
- 后续如继续压测，建议优先跑 6 章短链路或默认完整链路，观察长链路耗时和 provider 波动。

## 47. 开发者 B xhigh 6 章压测与 Character 收敛记录

处理日期：2026-06-09

已处理：

- B-P1：`Character create_novel` 输入新增 `scope`，传入 `focus_chapters`、`max_characters`、`max_relationships_per_character`、`max_turning_points_per_character`、`max_plan_items_per_character`，降低真实模型在人物卡阶段百科式展开。
- B-P1：Character Agent prompt 增加规模约束：最多输出 4 个核心人物，每个人物关系、转折和章节计划都按 scope 限制；字段名仍保持 `chapter_1_to_30_plan`，但内容只覆盖本轮 focus chapters。
- B-P1：Character Agent 默认 `max_tokens` 从 4000 降到 3000，减少 `gpt-5.5 + xhigh` 在 6 章链路中的输出和推理压力。
- B 协作：`docs/SCHEMAS.md` 已补充 Character Agent 输入 `scope` 字段说明。

压测观察：

```text
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 6 -OutlineChapters 6 -NewOutlineBatchSize 2 -OutlineBatchSize 2 -SkipOutline -SkipRewrite -StepRetries 0
失败：new 阶段在 3 个 Plot batch 均 ok 后、Character 未落库前出现 exit code -1。

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 6 -OutlineChapters 6 -NewOutlineBatchSize 3 -OutlineBatchSize 3 -SkipOutline -SkipRewrite -StepRetries 0
修复前失败：new 阶段在 2 个 Plot batch 均 ok 后、Character 未落库前出现 exit code -1。

Character scope 收敛后，同一命令推进到 review：
new ok; write ok; review 首次调用 exit code -1。
```

分段补验：

```text
cargo run --quiet -- --config <xhigh-6ch-config> review --novel-id e5cd0a65-1286-4736-9a20-73599517dea1 --chapter 1
审稿总分: 84
是否通过: 是

cargo run --quiet -- --config <xhigh-6ch-config> export --novel-id e5cd0a65-1286-4736-9a20-73599517dea1 --format markdown --output <export-after-review.md>
export_size=19347

cargo run --quiet -- --config <xhigh-6ch-config> runs --novel-id e5cd0a65-1286-4736-9a20-73599517dea1 --limit 80 --summary --fail-on-bad-status
agent_run_summary total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=700385
```

当前状态：

- 6 章数据链路和 AgentOutput 质量分段验证通过，未出现 fallback / parse_error。
- 6 章一次性 demo 脚本仍未稳定通过；当前失败更像本地 `cliproxyapi/gpt-5.5 xhigh` 偶发子进程中断，而非业务解析或存储问题。
- 下一步建议继续做 6 章一次性脚本复跑，或在 demo 层为真实 provider 子进程级 `exit code -1` 引入更明确的阶段日志/可恢复检查点。

## 46. 开发者 A 第三十一轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：新增 `POST /api/novels/{novel_id}/chapters/write/jobs`，支持 Web 工作台一次提交章节范围并后台顺序生成多章。
- A-P2：批量写作 job kind 为 `write_chapters`，`payload` 保存 `chapter_start`、`chapter_end` 和 `chapter_indexes`，成功 `result` 返回 `chapter_start`、`chapter_end` 和 `drafts`。
- A-P2：批量 job 复用单章 `ChapterGenerationWorkflow::write_chapter`，每章仍保留正文、版本、连续性报告、Style 结果和 facts 写入。
- A-P2：批量 job 支持 `retry` 和 `cancel`；retry 会用 `source_job_id` 指向源失败任务，cancel 后不会再开始下一章，也不会覆盖 `cancelled` 终态。
- A-P2：API smoke test 覆盖批量 job 成功、批量 job retry 成功和 `source_job_id` 来源关系。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 和 `scripts/api_demo.ps1` 已同步批量章节 job 接口。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 14 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / write / review / SSE / jobs(source_job_id) / batch-jobs / retry-completed-400 / cancel-completed-400 / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- Web 工作台可以直接创建、轮询、取消和重试批量章节写作任务；下一步可继续补任务进度字段或章节范围 UI 所需的更细粒度进度。

## 47. 开发者 A 第三十二轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：`api_jobs` 和 `JobRecord` 增加 `progress_current` / `progress_total`，用于 Web 工作台展示后台任务进度。
- A-P2：普通 job 默认 `0/1`，成功后自动写回 `1/1`；批量章节 job 创建时 `progress_total` 为章节数量，每完成一章推进 `progress_current`。
- A-P2：旧 `api_jobs` 表迁移会自动补 progress 列，并把既有 `succeeded` 任务回填为完成进度。
- A-P2：`JobRepository::set_progress` 只允许更新 `running` job；失败或取消会保留当前进度，完成写回不会覆盖 cancelled 终态。
- A-P2：API smoke test 覆盖单章 job、批量 job、retry job 的进度字段；Storage 单元测试覆盖批量进度推进和完成回填。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 和 `scripts/api_demo.ps1` 已同步 job progress 字段。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 14 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / write / review / SSE / jobs(progress) / batch-jobs(progress) / retry-completed-400 / cancel-completed-400 / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- Web 工作台可以直接用 `progress_current/progress_total` 展示单章、批量、重试和取消任务的进度。

## 48. 开发者 A 第三十三轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：`GET /api/jobs` 增加 `status` / `kind` 可选筛选参数，用于 Web 工作台任务面板查看运行中任务、批量任务或指定类型任务。
- A-P2：非法 `status` 会复用统一错误响应返回 `400 Bad Request`，避免 UI 静默拿到空列表。
- A-P2：`JobRepository` 增加 `list_recent_filtered`，底层查询支持 `status` / `kind` 组合筛选，并新增 `idx_api_jobs_status_kind_updated_at` 索引。
- A-P2：API smoke test 覆盖 `status=succeeded&kind=write_chapters` 和非法 status；Storage 单元测试覆盖 repository 层筛选。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 和 `scripts/api_demo.ps1` 已同步 jobs 列表筛选。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 14 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / write / review / SSE / jobs(filter) / batch-jobs / retry-completed-400 / cancel-completed-400 / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- Web 工作台可以直接按状态和任务类型筛选 job 列表；下一步可继续补任务面板的更细筛选，例如 novel_id 或 source_job_id。

## 49. 开发者 A 第三十四轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：`GET /api/jobs` 增加 `novel_id` / `source_job_id` 可选筛选参数，用于 Web 工作台任务面板查看当前作品任务和某个失败任务的 retry 链。
- A-P2：`JobRepository::list_recent_filtered` 扩展为支持 `status` / `kind` / `novel_id` / `source_job_id` 组合筛选。
- A-P2：SQLite 迁移增加 `idx_api_jobs_novel_id_updated_at` 索引，并继续保留 `idx_api_jobs_source_job_id`，支撑作品维度和来源维度查询。
- A-P2：API smoke test 覆盖 `novel_id` 筛选、`novel_id + source_job_id` 组合筛选；Storage 单元测试覆盖 repository 层作品筛选和 retry 来源筛选。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 和 `scripts/api_demo.ps1` 已同步 jobs 细粒度筛选能力。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 14 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / write / review / SSE / jobs(filter + novel_id) / batch-jobs / retry-completed-400 / cancel-completed-400 / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- Web 工作台可以直接按任务状态、类型、作品和 retry 来源筛选 job 列表；任务面板可以分别呈现“当前作品任务”和“某失败任务的重试链”。

## 50. 开发者 A 第三十五轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：新增 `GET /api/novels/{novel_id}/facts?limit=<n>`，用于 Web 工作台独立查询作品事实表和伏笔/事实侧栏数据。
- A-P2：新增 `GET /api/novels/{novel_id}/chapters/{chapter_index}/continuity`，返回指定章节最新 Continuity Agent 结构化报告。
- A-P2：新增 `FactsResponse` / `LatestContinuityResponse`，保持 facts 使用 domain `Fact`，continuity report 保持原始结构化 JSON，避免和 Prompt schema 分叉。
- A-P2：API smoke test 覆盖 facts 列表和章节最新连续性报告查询；API demo 覆盖 HTTP facts / continuity 调用。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md` 和 `docs/WORKPLAN.md` 已同步 facts / continuity 只读 API。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 14 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / facts / write / continuity / review / SSE / jobs(filter + novel_id) / batch-jobs / retry-completed-400 / cancel-completed-400 / export / runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23
```

当前状态：

- Web 工作台可以直接读取作品事实表和章节连续性报告；事实/伏笔面板与连续性侧栏不再必须依赖整份作品详情接口。

## 51. 开发者 A 第三十六轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：`GET /api/novels/{novel_id}/runs` 增加 `role` / `task` / `status` 可选筛选参数，用于 Web 工作台运行时间线按 Agent 角色、任务类型和运行状态筛选。
- A-P2：`AgentRunRepository::list_recent_filtered` 支持按 `role` / `task` 在 SQLite 层筛选；`status` 继续复用 `AgentRunRecord::status()` 的工程口径，保持 API 与 CLI 状态一致。
- A-P2：非法 `status` 返回 `400 Bad Request`，避免运行面板静默展示空数据。
- A-P2：API smoke test 覆盖 `role=writer&task=generate_chapter&status=ok` 和非法 status；API demo 覆盖 filtered AgentRun 查询。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 和 `scripts/api_demo.ps1` 已同步 AgentRun 筛选能力。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 14 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / facts / write / continuity / review / SSE / jobs(filter + novel_id) / batch-jobs / retry-completed-400 / cancel-completed-400 / export / runs / filtered-runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23
```

当前状态：

- Web 工作台运行面板可以直接按 Agent 角色、任务类型和状态筛选运行记录，并获得对应筛选后的 summary。

## 52. 开发者 A 第三十七轮处理记录

处理日期：2026-06-09

已处理：

- A-P2：新增全局 `GET /api/runs`，支持 Web 工作台运行面板不依赖逐作品聚合即可读取最近 AgentRun。
- A-P2：全局 runs 支持 `limit` / `novel_id` / `role` / `task` / `status` 可选筛选参数；作品内 `GET /api/novels/{novel_id}/runs` 保持兼容。
- A-P2：`AgentRunsQuery` 增加 `novel_id`，复用同一 `agent_runs_response` 生成筛选结果和 summary，保证全局入口与作品入口状态统计口径一致。
- A-P2：API smoke test 覆盖 `GET /api/runs?novel_id=...&role=writer&task=generate_chapter&status=ok`；API demo 覆盖 global filtered AgentRun 查询。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 和 `scripts/api_demo.ps1` 已同步全局 AgentRun 查询入口。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 14 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / facts / write / continuity / review / SSE / jobs(filter + novel_id) / batch-jobs / retry-completed-400 / cancel-completed-400 / export / runs / filtered-runs / global-filtered-runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
write / rewrite stream output observed
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23
```

当前状态：

- Web 工作台可直接通过 `GET /api/runs` 构建全局运行面板，也可继续通过作品内 runs 接口查看单作品运行记录。

## 53. 开发者 B xhigh 检查点续跑收口记录

处理日期：2026-06-09

已处理：

- B-P1 / C-P0-2：`new --resume-novel-id` 已进入验收口径，可复用同一作品下已成功的 Market / Plot / Character / Worldbuilding AgentRun，避免真实模型中断后从头重跑。
- B-P1：修正续跑时人物卡重复落库问题；如果作品已有人物卡，续跑会沿用既有人物，不再把同一份 Character 输出重复插入。
- B-P1：`write_chapter` 在复用 Writer 输出时会优先复用该章已持久化的 Continuity report，避免真实模型恢复时反复重跑 Continuity。
- B-P1 / C-P0-2：`scripts/mvp_demo.ps1` 会提前输出 `work_dir`，新书成功后输出 `resume_novel_id`，并已将 `-WorkDir` / `-ResumeNovelId` 写入 README、MVP 验收和接口冻结文档；本地 OpenAI-compatible 6 章验收可在 provider 子进程中断后复用临时库继续。
- B-P1：新增 smoke 回归 `resume_create_novel_reuses_completed_agent_runs_without_duplicate_characters` 和 `write_chapter_resume_reuses_persisted_continuity_report`，覆盖新书续跑不重复人物、章节续跑不重复 Continuity。
- 真实模型记录：本地 cliproxyapi `gpt-5.5 + reasoning_effort=xhigh` 6 章最新复跑中，一次性 demo 在 `write` 阶段仍出现 provider 子进程 `exit code -1`；用同一 `work_dir` / `resume_novel_id` 续跑后完成 `write -> review -> export -> runs`，最终 `agent_run_summary total=10 ok=10 fallback=0 parse_error=0 duration_ms_total=823868`、`export_size=19962`。

验证结果：

```text
cargo fmt
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 16 passed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
CORS / create / list / facts / write / continuity / review / SSE / jobs(filter + novel_id) / batch-jobs / retry-completed-400 / cancel-completed-400 / export / runs / filtered-runs / global-filtered-runs all ok

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 6 -OutlineChapters 6 -NewOutlineBatchSize 3 -OutlineBatchSize 3 -SkipOutline -SkipRewrite -StepRetries 0
write 阶段出现 provider 子进程 exit code -1；已输出 work_dir 和 resume_novel_id

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 6 -OutlineChapters 6 -NewOutlineBatchSize 3 -OutlineBatchSize 3 -SkipOutline -SkipRewrite -StepRetries 0 -WorkDir <work_dir> -ResumeNovelId <resume_novel_id>
agent_run_summary total=10 ok=10 fallback=0 parse_error=0 duration_ms_total=823868
export_size=19962

git diff --check
ok
```

当前状态：

- B 线真实模型验收已有可恢复路径；下一步应继续复跑 `gpt-5.5 + xhigh` 6 章一次性脚本，判断剩余失败是否完全来自本地 provider 进程稳定性。

## 54. 开发者 A 第三十八轮处理记录

处理日期：2026-06-09

已处理：

- A-P1 / C-P0-2：新增 `PUT /api/novels/{novel_id}/chapters/{chapter_index}/edit`，供 Web 编辑器真实模式保存人工编辑稿。
- A-P1：人工保存接口请求体固定为 `title?` / `content` / `summary?`，响应复用 `{ "draft": {} }`；空 `content` 返回 `400 Bad Request`。
- A-P1：接口复用 `ChapterGenerationWorkflow::save_manual_edit`，只保存新的 `ChapterDraft` / `chapter_versions`，不调用 Agent、不生成 `agent_runs`、不自动刷新 facts。
- A-P1：API smoke test 覆盖人工保存、版本递增、章节最新内容读取、版本列表包含人工版本和空内容 400。
- A/C 协作：README、`docs/API.md`、`docs/MVP_ACCEPTANCE.md`、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md` 和 `scripts/api_demo.ps1` 已同步人工保存章节接口。

验证结果：

```text
rustfmt --edition 2024 src\api.rs
ok

cargo check
Finished `dev` profile ... ok

cargo test api_can_create_and_read_smoke_project
1 passed

cargo test
api/storage unit: 3 passed
smoke tests: 15 passed

powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
manual edit chapter / manual edit versions observed
agent_run_total=21
```

当前状态：

- 后端已开放 Web 人工保存章节入口；C 侧真实 API 客户端可将 `saveChapterContent` 对接到 `PUT /api/novels/{novel_id}/chapters/{chapter_index}/edit`。

## 55. 开发者 B xhigh 同次检查点续跑处理记录

处理日期：2026-06-09

已处理：

- B-P1 / C-P0-2：`scripts/mvp_demo.ps1` 新增 `-CheckpointResumes`，在 `-UseRealModel` 下只对已写入检查点的 `new` / `write` / `review` 失败步骤做同次续跑。
- B-P1：`write` 步骤失败后，如果检测到同章章节版本、Writer、Continuity 或 Style 成功 AgentRun，会继续执行同一步；配合 workflow 复用 Writer / Continuity，可把 provider 子进程中断后的恢复留在同一次脚本里。
- B-P1：`review` 步骤失败后，如果检测到 Reviewer 成功 AgentRun，会跳过重复审稿，继续后续导出和 `runs --fail-on-bad-status`。
- B-P1 / C-P0-2：README、MVP 验收和接口冻结文档已把 6 章 xhigh 推荐命令补充为 `-CheckpointResumes 6`。

验证结果：

```text
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -CheckpointResumes 2
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 6 -OutlineChapters 6 -NewOutlineBatchSize 3 -OutlineBatchSize 3 -SkipOutline -SkipRewrite -StepRetries 0 -CheckpointResumes 3
new 阶段首次 exit code -1 后同次 new resume 成功；write 阶段连续检查点续跑后仍耗尽额度并返回 exit code -1

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 6 -OutlineChapters 6 -NewOutlineBatchSize 3 -OutlineBatchSize 3 -SkipOutline -SkipRewrite -StepRetries 0 -CheckpointResumes 6 -WorkDir <work_dir> -ResumeNovelId <resume_novel_id>
agent_run_summary total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=643718
export_size=19960
```

当前状态：

- `new` 阶段已验证同次检查点续跑可用；`write` 阶段在本地 provider 持续 `exit code -1` 时仍可能需要提高 `-CheckpointResumes` 或用同一 `work_dir` 手动续跑。

## 56. 开发者 B Style 检查点复用补强记录

处理日期：2026-06-09

已处理：

- B-P1：`write_chapter` 续跑现在会在复用同章 Writer 输出时同步复用已成功的 Style `polish_style` AgentRun，避免 Style 成功后进程在保存章节前中断导致下一次续跑重复调用 Style。
- B-P1：AgentRun 入库前会在输入含 `chapter_draft` 时补充 `_workflow.chapter_index` / `volume_index` / `chapter_id`，让 Continuity / Style 这类业务输出不自带章号的 AgentRun 也能被可靠定位。
- B-P1：回归测试 `write_chapter_resume_reuses_persisted_continuity_and_style` 已覆盖同章第二次写作复用 Writer、Continuity 和 Style，且不重复插入 Continuity 报告。

验证结果：

```text
cargo check
Finished `dev` profile ... ok

cargo test
unit/API/storage: 3 passed
smoke tests: 16 passed

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -CheckpointResumes 2
agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=0 tokenized_runs=23
export_size=9899

powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 6 -OutlineChapters 6 -NewOutlineBatchSize 3 -OutlineBatchSize 3 -SkipOutline -SkipRewrite -StepRetries 0 -CheckpointResumes 6
agent_run_summary total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=639102
export_size=20616
```

当前状态：

- `write` 检查点恢复链路已从 Writer / Continuity 扩展到 Style；本轮真实 `gpt-5.5 + xhigh` 6 章从头跑通，后续如果再遇到 provider 子进程中断，应能少一次不必要的 Style 重打。

## 57. 开发者 B UI 内容口径与返工类型处理记录

处理日期：2026-06-09

已处理：

- B-P0：新增 `docs/UI_CONTENT_GUIDE.md`，给 C 侧审稿面板、连续性侧栏、事实表、AgentRun 面板和空状态文案提供展示口径。
- B-P0：梳理每个 AgentRun role 的 UI 摘要来源，避免前端直接展示大段原始 JSON。
- B-P0：Reviewer Prompt 已要求 `issues` 按严重程度排序、带可定位 `location`，`suggestions` 必须能直接转成作者待办。
- B-P0：将 `rewrite_instruction.rewrite_type` 业务口径从 `none / partial / full` 扩展为 `none / partial / full / opening / ending / style`，覆盖整章重写、开头重写、结尾重写和语言润色。
- B-P0：`docs/UI_CONTENT_GUIDE.md` 明确 3 个 UI demo 项目来源：`examples/urban_rebirth.md`、`examples/fantasy_upgrade.md`、`examples/romance_comeback.md`。
- B-P1：Continuity / Style Prompt 已补充输出质量约束，重点约束 issues 排序、事实重要度、伏笔状态、Style changes 和 preserved_facts。
- B-P1：新增 `docs/HUMAN_EVAL.md`，提供 provider / prompt / 题材样例人工评测表和失败原因归类。
- B/C 协作：README、`docs/INTERFACE_FREEZE.md`、`docs/WORKPLAN.md`、`docs/SCHEMAS.md` 和 `prompts/reviewer_agent.md` 已同步 UI 内容和返工类型口径。

验证结果：

```text
文档/Prompt 口径更新，无 Rust 行为变更。
cargo check ok
git diff --check ok
```

当前状态：

- C 侧可以按 `docs/UI_CONTENT_GUIDE.md` 接审稿面板和运行面板；B 侧后续 provider 对比和 prompt 版本对比可按 `docs/HUMAN_EVAL.md` 做人工评测。若前端需要新增持久化字段，再回到 `docs/API.md` 和 Rust DTO 层处理。

## 58. 开发者 B 平台题材模板与钩子一致性处理记录

处理日期：2026-06-09

已处理：

- B-P2：新增 `docs/PLATFORM_TEMPLATES.md`，沉淀起点 / 番茄 / 通用的平台策略、章尾钩子类型和人物行为一致性检查。
- B-P2：为都市重生商业文、玄幻升级文、女性向逆袭复仇建立前三章节奏模板，供 Market / Plot / Writer / Reviewer 和 UI demo 共用。
- B-P2：Market Prompt 已要求按题材模板输出前三章读者承诺、第一冲突和 Plot handoff 钩子方向。
- B-P2：Plot Prompt 已补充章尾钩子自然延伸规则、人物变化可执行规则、重生信息差偏差和反派局部优势要求。
- B-P2：Writer Prompt 已补充开头 800 字、中段压力密度、主角主动选择、配角动机和章尾自然延伸要求。
- B-P2：Reviewer Prompt 已把非自然章尾钩子和人物行为不一致纳入扣分上限。
- B/C 协作：README 和 `docs/WORKPLAN.md` 已同步平台模板、钩子和人物一致性状态。

验证结果：

```text
文档/Prompt 口径更新，无 Rust 行为变更。
cargo check ok
git diff --check ok
```

当前状态：

- B 线 P0/P1/P2 的提示词、UI 展示口径、人工评测表和平台题材模板都已有首版；后续可以开始按 `docs/HUMAN_EVAL.md` 做 provider / prompt 版本对比。

## 59. 开发者 B 首条真实模型人工评测记录

处理日期：2026-06-09

已处理：

- B-P2：新增 `docs/EVAL_LOG.md`，按 `docs/HUMAN_EVAL.md` 记录首条真实模型人工评测。
- B-P2：评测对象为本地 OpenAI-compatible `gpt-5.5 + reasoning_effort=xhigh` 6 章短链路中的第 1 章《重生第一天，先撕罚款单》。
- B-P2：记录了验收命令、`agent_run_summary total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=639102`、`export_size=20616`、ReviewReport `total_score=88 passed=true` 和人工 10 维评分。
- B-P2：人工总分 44 / 50，结论为“可用，建议小修后进入 Web demo”；主要改法是压缩跑单中段、补清欠薪金额线、强化母亲押金倒计时、后续加入未来记忆偏差。
- B/C 协作：README、`docs/HUMAN_EVAL.md` 和 `docs/WORKPLAN.md` 已同步评测记录入口。

验证结果：

```text
文档记录更新，无 Rust 行为变更。
cargo check ok
git diff --check ok
```

当前状态：

- B 侧已有一条可复用的真实模型质量基线；下一条建议用同题材 DeepSeek 或新 Prompt 后的 `gpt-5.5 + xhigh` 结果做对照。

## 60. 开发者 B DeepSeek 人工评测对照记录

处理日期：2026-06-09

已处理：

- B-P2：`docs/EVAL_LOG.md` 新增 DeepSeek 历史真实输出评测，作为 `gpt-5.5 + xhigh` 都市重生第 1 章的 provider 对照。
- B-P2：评测对象为 `deepseek / deepseek-chat` 完整真实 demo 历史输出中的第 1 章《重生：五百块与一个未来》，novel_id 为 `ebc0233d-278f-436f-94f7-6935e089c6ae`。
- B-P2：记录历史链路 `agent_run_summary total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=613501 tokenized_runs=0` 和 `export_size=25792`。
- B-P2：人工总分 36 / 50，结论为“需要返工后再展示”；主要问题是即时压力不足、商业谈判偏顺、章尾钩子不够硬，适合作为 DeepSeek provider 质量基线。
- B/C 协作：README、`docs/HUMAN_EVAL.md` 和 `docs/WORKPLAN.md` 已同步评测记录已覆盖 `gpt-5.5 xhigh` 与 DeepSeek 对照。

验证结果：

```text
文档记录更新，无 Rust 行为变更。
fresh DeepSeek API run 未执行：当前审批系统拦截了新真实 API 调用，本条使用已存在的历史真实输出。
```

当前状态：

- B 侧已有两条真实模型人工质量基线：`gpt-5.5 + xhigh` 当前推荐样本 44 / 50，DeepSeek 历史对照样本 36 / 50。下一步可在最新 Prompt 下重新跑 DeepSeek，或做 `gpt-5.5 xhigh` prompt 版本对比。

## 61. 开发者 B DeepSeek 对照反馈反灌 Prompt 记录

处理日期：2026-06-09

已处理：

- B-P2：根据 DeepSeek 对照样本暴露出的“商业谈判过顺、未来趋势解释偏多、章尾只剩方向宣言”问题，补强 Market / Plot / Writer / Reviewer Prompt。
- B-P2：Market Prompt 要求都市重生商业文的第一冲突必须包含外部阻力、失败代价和主角当场选择，并在 `opening_strategy.avoid` 中规避长篇商业模式解释。
- B-P2：Plot Prompt 要求商业、创业、谈判类章节把机会写成冲突，落到订单、资金、合同、竞争压力或阻止损失，而不是只讲行业未来。
- B-P2：Writer Prompt 要求商业谈判写成对抗场面，未来信息必须转化为动作、对白、判断失误或临场补救。
- B-P2：Reviewer Prompt 对“谈判过顺”和“章尾方向宣言”设置 `pacing_score`、`payoff_score`、`cliffhanger_score` 扣分上限。
- B-P2：`docs/PLATFORM_TEMPLATES.md` 新增都市重生商业场景最低要求，`docs/RUBRIC.md` 同步番茄向扣分和分数上限，供后续人工评测和 Prompt 迭代复用。

验证结果：

```text
文档/Prompt 口径更新，无 Rust 行为变更。
```

当前状态：

- DeepSeek 历史样本不再只是评测记录，已转化为下一轮生成的 Prompt 约束；后续重新跑 DeepSeek 或做 prompt 版本对比时，应重点观察第一章是否出现明确外部阻力、资源代价和章尾具体压力。

## 62. 开发者 B 都市重生样例质量回归补强记录

处理日期：2026-06-09

已处理：

- B-P1 / B-P2：`examples/urban_rebirth.md` 的 `expected_checks` 已补充外部阻力、失败代价、下一步压力等可测试要求，覆盖 DeepSeek 对照样本暴露的问题。
- B-P1：都市样例 `writer.forbidden` 增加“合作无阻力谈成”“只讲趋势不写场景”“我要建立商业帝国”，避免商业文退化成方向宣言或行业分析。
- B-P1：`tests/smoke.rs` 的都市 fixture 已同步新增 `外部阻力` 关键事件，并在正文 fixture 中写入失败代价和章尾具体订单压力。
- B-P1：现有 `valid_json_model_outputs_match_fixture_expected_checks` 会消费这些新增断言，后续如果本地合法 JSON fixture 退化，会在 smoke test 中失败。

验证结果：

```text
cargo fmt
ok

cargo test valid_json_model_outputs_match_fixture_expected_checks
1 passed

git diff --check
ok，仅有 LF/CRLF 提示

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 16 passed
```

当前状态：

- 都市重生商业文的 Prompt 约束、人工评测口径和自动 fixture 回归已形成闭环：评测发现问题 -> Prompt/Rubric 修正 -> expected_checks 固化。

## 63. 开发者 B Provider 对照摘要与判读规则记录

处理日期：2026-06-09

已处理：

- B-P2：`docs/EVAL_LOG.md` 新增 Provider 对照摘要表，把 `gpt-5.5 xhigh` 44 / 50 和 DeepSeek 历史样本 36 / 50 的 demo 状态、优势、风险和后续动作放到顶部。
- B-P2：`docs/HUMAN_EVAL.md` 新增 Provider 对照判读规则，明确总分差、开篇抓力、冲突密度、章尾钩子和平台适配的判读口径。
- B-P2：`docs/EVAL_LOG.md` 中 `gpt-5.5 xhigh` 条目的“下一次对比 DeepSeek”旧文案已更新为“DeepSeek 对照已完成，并已反灌到 Prompt / Rubric / expected_checks”。
- B/C 协作：README 和 `docs/WORKPLAN.md` 已同步评测记录包含 provider 对照摘要和判读规则。

验证结果：

```text
git diff --check
ok，仅有 LF/CRLF 提示

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 16 passed
```

当前状态：

- 后续新增 provider 或 prompt 版本评测时，可以先看 `docs/EVAL_LOG.md` 顶部摘要判断是否进入 Web demo、人工小修或失败原因库。

## 64. 开发者 B Prompt 版本记录处理记录

处理日期：2026-06-09

已处理：

- B-P2：新增 `docs/PROMPT_CHANGELOG.md`，记录当前 Prompt bundle `b-quality-2026-06-09-r3`、覆盖文件、关键变化和评测使用规则。
- B-P2：`prompts/README.md` 已标明当前 Prompt bundle，并指向版本记录文档。
- B-P2：`docs/HUMAN_EVAL.md` 的元数据和记录模板新增 `prompt_bundle` 字段，后续 provider / prompt 版本对比必须记录。
- B-P2：`docs/EVAL_LOG.md` 已标注两条历史真实样本早于 `b-quality-2026-06-09-r3`，避免和新 Prompt 输出无标注直接比较。
- B/C 协作：README 和 `docs/WORKPLAN.md` 已同步 Prompt 版本记录入口。

验证结果：

```text
git diff --check
ok，仅有 LF/CRLF 提示

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 16 passed
```

当前状态：

- B 线质量评测现在具备三层追踪：provider 对照摘要、人工评分记录、Prompt bundle 版本记录。后续新增真实评测时可以明确区分“模型差异”和“Prompt 版本差异”。

## 65. 开发者 B 评测日志元数据回归记录

处理日期：2026-06-09

已处理：

- B-P2：`tests/smoke.rs` 新增 `eval_log_records_include_prompt_bundle_and_run_summary`，自动检查 `docs/EVAL_LOG.md` 中每条真实评测记录必须包含 `prompt_bundle`、AgentRun summary 和人工总分。
- B-P2：该测试把第 64 轮新增的 Prompt bundle 追踪从“文档约定”推进到“新增评测记录会被测试提醒”。
- B/C 协作：`docs/WORKPLAN.md` 已同步评测日志关键元数据纳入 smoke 测试。

验证结果：

```text
cargo fmt
ok

cargo test eval_log_records_include_prompt_bundle_and_run_summary
1 passed

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 3 passed
smoke tests: 17 passed

git diff --check
ok，仅有 LF/CRLF 提示
```

当前状态：

- 后续追加 provider 或 prompt 版本评测时，如果漏写关键元数据，smoke test 会失败，避免评测日志重新变成不可比较的散记。

## 66. 开发者 B Web demo 内容包与失败样例库记录

处理日期：2026-06-09

已处理：

- B/C 协作：新增 `docs/WEB_DEMO_CONTENT.md`，给 C 侧主工作台提供 `urban_rebirth_fanqie_demo` 内容包，包含作品信息、第 1 章展示正文、ReviewReport mock、Continuity/facts mock、AgentRun mock 和 DeepSeek 负向对照摘要。
- B-P2：新增 `docs/FAILURE_CASES.md`，沉淀 `quality_regression`、`parse_error`、`provider_error`、`eval_process` 四类失败样例。
- B-P2：失败样例库已覆盖 DeepSeek 历史弱输出、DeepSeek Plot 长 JSON 截断、本地 `gpt-5.5 xhigh` provider 子进程中断、评测日志缺元数据风险。
- B-P2：`tests/smoke.rs` 新增 `failure_cases_document_covers_known_failure_types`，防止失败样例库关键 case id 和类型丢失。
- B/C 协作：README、`docs/UI_CONTENT_GUIDE.md` 和 `docs/WORKPLAN.md` 已同步 Web demo 内容包与失败样例库入口。

验证结果：

```text
rustfmt --edition 2024 src/agents/mod.rs src/workflow/agent_runner.rs src/workflow/chapter_generation.rs src/workflow/novel_creation.rs tests/smoke.rs
ok

cargo check
Finished `dev` profile ... ok

cargo test
api/storage unit: 4 passed
smoke tests: 18 passed

scripts/mvp_demo.ps1
new -> outline -> write -> review -> rewrite -> versions -> edit -> export -> runs ok
```

当前状态：

- C 侧已有可直接 mock 的主 demo 内容包；B 侧真实失败与弱输出已有失败样例库，不再只散落在长审查记录里。
