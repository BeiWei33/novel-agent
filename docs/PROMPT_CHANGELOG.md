# novel-agent Prompt 版本记录

本文档记录开发者 B 维护的 Prompt bundle 版本。它用于人工评测、provider 对比和 demo 内容验收，避免把不同 Prompt 口径下的输出混在一起比较。

## 当前版本

| 字段 | 值 |
| --- | --- |
| prompt_bundle | `b-quality-2026-06-09-r3` |
| 日期 | 2026-06-09 |
| 负责人 | B |
| schema 兼容性 | 不变，继续使用 `AgentOutputEnvelope` |
| Rust 行为变更 | 无 |
| 适用评测 | 新一轮 provider / prompt 版本对比 |

### 版本范围

本版本覆盖以下 Prompt 和业务口径：

- `prompts/market_agent.md`
- `prompts/plot_agent.md`
- `prompts/chapter_writer_agent.md`
- `prompts/continuity_agent.md`
- `prompts/style_agent.md`
- `prompts/reviewer_agent.md`
- `docs/RUBRIC.md`
- `docs/PLATFORM_TEMPLATES.md`
- `examples/urban_rebirth.md`

### 关键变化

- 都市重生商业文第一冲突必须包含外部阻力、失败代价和主角当场选择。
- 商业、创业、谈判类章节必须把机会写成冲突，不能只讲行业趋势。
- Writer 必须把未来信息转化为动作、对白、判断失误或临场补救。
- Reviewer 对“商业谈判过顺”和“章尾方向宣言”设置扣分上限。
- 番茄向评测新增“外部阻力、资源代价、下一步具体压力”判断。
- `examples/urban_rebirth.md` 的 `expected_checks` 已纳入外部阻力、失败代价和下一步压力的自动回归。

### 使用规则

新增人工评测记录时必须写入：

- `prompt_bundle`
- provider / model / reasoning effort
- 题材样例和平台
- 章节范围
- AgentRun summary
- 是否出现 fallback / parse_error

如果评测样本来自旧输出或历史临时库，必须标明：

- `prompt_bundle` 是否可确定。
- 样本是否早于当前 Prompt bundle。
- 是否允许和当前版本输出做直接质量对比。

## 历史记录

### `b-quality-2026-06-09-r2`

- 新增 `docs/EVAL_LOG.md`，记录 `gpt-5.5 xhigh` 和 DeepSeek 都市重生第 1 章对照。
- 新增 `docs/HUMAN_EVAL.md` provider 对照判读规则。
- 新增 `docs/PLATFORM_TEMPLATES.md`，沉淀平台策略、章尾钩子和题材模板。
- 该版本的真实输出样本可用于 provider 基线对照，但 DeepSeek 样本暴露的问题已在 r3 中修正。

### `b-quality-2026-06-09-r1`

- 新增 UI 内容口径、人工评测表和平台题材模板首版。
- Reviewer Prompt 扩展返工类型：`none | partial | full | opening | ending | style`。
- Continuity / Style Prompt 补充 issues 排序、事实重要度和 style changes 输出要求。
