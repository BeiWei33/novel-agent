# novel-agent Prompt 版本记录

本文档记录开发者 B 维护的 Prompt bundle 版本。它用于人工评测、provider 对比和 demo 内容验收，避免把不同 Prompt 口径下的输出混在一起比较。

## 当前版本

| 字段 | 值 |
| --- | --- |
| prompt_bundle | `b-quality-2026-06-10-v0.3-guard` |
| 日期 | 2026-06-10 |
| 负责人 | B |
| schema 兼容性 | 不变，继续使用 `AgentOutputEnvelope` |
| Rust 行为变更 | 无 |
| 适用评测 | 已完成 `gpt-5.5 xhigh` 2 章快速真实复跑；v0.3 Web 工作台接入期的新生成和质量视图守门 |

### 版本范围

本版本覆盖以下 Prompt 和业务口径：

- `prompts/market_agent.md`
- `prompts/plot_agent.md`
- `prompts/chapter_writer_agent.md`
- `prompts/continuity_agent.md`
- `prompts/style_agent.md`
- `prompts/reviewer_agent.md`
- `docs/RUBRIC.md`
- `docs/UI_CONTENT_GUIDE.md`
- `docs/FAILURE_CASES.md`
- `docs/PLATFORM_TEMPLATES.md`
- `examples/urban_rebirth.md`
- `examples/fantasy_upgrade.md`
- `examples/romance_comeback.md`

### 关键变化

- Writer / Continuity / Reviewer 增加跨产物一致性守门：主角姓名、关键金额、债务、罚款、押金、合同状态、合作关系和敌对状态必须统一。
- Reviewer 遇到未解释的一致性硬伤时，`continuity_score` 最高 6，且 `passed = false`。
- `docs/RUBRIC.md` 新增 v0.3 一致性硬门，供 Web 质量视图和人工评测共用。
- `docs/UI_CONTENT_GUIDE.md` 新增 v0.3 轻量质量视图：通过线、一致性硬门、链路健康、来源标识和返工待办分组。
- `examples/*.md` 新增 `cross_artifact_consistency` expected checks。
- `docs/FAILURE_CASES.md` 新增 `quality-cross-artifact-001`，覆盖人物名、金额和合作状态跨产物不一致风险。

### 当前 bundle 真实复跑

| provider / model | 范围 | novel_id | AgentRun summary | 质量结论 |
| --- | --- | --- | --- | --- |
| openai-compatible / gpt-5.5 xhigh | 2 章快速链路 | `eac2af4a-e21c-483b-b684-42be44fed943` | `total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=1164764` | 42 / 50，链路通过，ReviewReport `80 passed=false`，一致性硬门生效 |

说明：该样本使用本地 `localhost:3001/v1` OpenAI-compatible API。首次运行在 `write` 阶段超过工具 15 分钟超时，随后用同一 `work_dir` / `resume_novel_id` 检查点续跑成功，没有 fallback 或 parse_error。

### v0.3 真实模型展示基线

| provider / model | 范围 | novel_id | AgentRun summary | 质量结论 |
| --- | --- | --- | --- | --- |
| openai-compatible / gpt-5.5 xhigh | 6 章链路 | `ea942e57-abea-42cf-8fff-287b64017b41` | `total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=733844` | 43 / 50，可用，小修后进入 Web demo |
| openai-compatible / gpt-5.5 xhigh | 2 章快速链路 | `9a81ecfe-e740-4625-9ade-f27ccd866a95` | `total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=635468` | 41 / 50，需先修一致性 |
| deepseek / deepseek-chat | 2 章快速链路 | `2f286b6b-1ad8-4cff-8e6a-2866a48079ff` | `total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=189155` | 38 / 50，可用，小修 |

说明：以上展示基线样本产自 `b-quality-2026-06-09-r3`。v0.3 接入期可把它们作为展示基线和回归反例；如果要宣称新 bundle 的 6 章展示效果，需要按同题材重新复跑并写入 `docs/EVAL_LOG.md`。

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

### `b-quality-2026-06-09-r3`

- 都市重生商业文第一冲突必须包含外部阻力、失败代价和主角当场选择。
- 商业、创业、谈判类章节必须把机会写成冲突，不能只讲行业趋势。
- Writer 必须把未来信息转化为动作、对白、判断失误或临场补救。
- Reviewer 对“商业谈判过顺”和“章尾方向宣言”设置扣分上限。
- 番茄向评测新增“外部阻力、资源代价、下一步具体压力”判断。
- `examples/urban_rebirth.md` 的 `expected_checks` 已纳入外部阻力、失败代价和下一步压力的自动回归。
- 已完成 `gpt-5.5 xhigh` / DeepSeek 快速对照，以及 `gpt-5.5 xhigh` 6 章链路压测。

### `b-quality-2026-06-09-r2`

- 新增 `docs/EVAL_LOG.md`，记录 `gpt-5.5 xhigh` 和 DeepSeek 都市重生第 1 章对照。
- 新增 `docs/HUMAN_EVAL.md` provider 对照判读规则。
- 新增 `docs/PLATFORM_TEMPLATES.md`，沉淀平台策略、章尾钩子和题材模板。
- 该版本的真实输出样本可用于 provider 基线对照，但 DeepSeek 样本暴露的问题已在 r3 中修正。

### `b-quality-2026-06-09-r1`

- 新增 UI 内容口径、人工评测表和平台题材模板首版。
- Reviewer Prompt 扩展返工类型：`none | partial | full | opening | ending | style`。
- Continuity / Style Prompt 补充 issues 排序、事实重要度和 style changes 输出要求。
