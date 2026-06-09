# novel-agent 失败样例库

日期：2026-06-09
负责人：开发者 B
用途：沉淀真实模型或真实链路中出现过的失败模式，并明确它们已经进入哪些回归防线。

失败样例不是普通 bug 列表。它关注的是“下次怎么防止同类文本或链路被误判为可展示”。

## 1. 状态说明

| 状态 | 含义 |
| --- | --- |
| `covered` | 已有 Prompt、Rubric、文档或测试回归覆盖 |
| `partial` | 已有缓解策略，但仍需要真实复跑确认 |
| `blocked_external` | 需要真实 API / 本地代理稳定性，当前不能只靠离线测试完成 |

## 2. 样例总表

| case_id | 类型 | 来源 | 状态 | 当前防线 |
| --- | --- | --- | --- | --- |
| `quality-urban-deepseek-001` | `quality_regression` | DeepSeek 历史真实输出第 1 章 | `covered` | Prompt / Rubric / `expected_checks` / 评测日志元数据测试 |
| `quality-cross-artifact-001` | `quality_regression` | r3 真实样本中人物名、金额、合作状态跨产物不一致风险 | `covered` | Continuity / Reviewer Prompt、Rubric、`expected_checks`、v0.3 质量视图 |
| `parse-deepseek-plot-001` | `parse_error` | DeepSeek 30 章长 JSON 大纲截断 | `covered` | Plot 分批、真实验收 `runs --fail-on-bad-status` |
| `provider-openai-xhigh-001` | `provider_error` | 本地 cliproxyapi `gpt-5.5 xhigh` 子进程 `exit code -1` | `partial` | `-CheckpointResumes`、AgentRun 复用、work_dir / resume_novel_id；r3 6 章复跑未复现 |
| `eval-metadata-001` | `eval_process` | 人工评测记录缺少 Prompt 版本信息的风险 | `covered` | `eval_log_records_include_prompt_bundle_and_run_summary` |

## 3. 详细样例

### `quality-urban-deepseek-001`

类型：`quality_regression`
来源：DeepSeek 历史真实输出第 1 章《重生：五百块与一个未来》
评测记录：`docs/EVAL_LOG.md`

问题：

- 开头有死亡重生信息，但现实危机没有持续压住主角。
- 商业谈判推进偏顺，缺少对手反压、时间限制或失败代价。
- 章尾偏“未来方向宣言”，没有下一章必须处理的具体压力。

已落地防线：

- `prompts/market_agent.md`：第一冲突必须包含外部阻力、失败代价和主角当场选择。
- `prompts/plot_agent.md`：商业/创业/谈判章节必须把机会写成冲突。
- `prompts/chapter_writer_agent.md`：未来信息必须转化为动作、对白、判断失误或临场补救。
- `prompts/reviewer_agent.md`：谈判过顺和章尾方向宣言有扣分上限。
- `docs/RUBRIC.md`：番茄向扣分规则同步。
- `examples/urban_rebirth.md`：`expected_checks` 增加外部阻力、失败代价和下一步压力。
- `tests/smoke.rs`：`valid_json_model_outputs_match_fixture_expected_checks` 消费新增断言。

### `quality-cross-artifact-001`

类型：`quality_regression`
来源：`gpt-5.5 xhigh` r3 快速链路和 6 章链路人工评测
评测记录：`docs/EVAL_LOG.md`

问题：

- `gpt-5.5 xhigh` r3 2 章快速链路出现主角名和罚款金额口径不一致。
- `gpt-5.5 xhigh` r3 6 章链路中 NovelBible / opening_strategy 残留旧名，和正文主角名不一致。
- 这类问题不一定让单章正文不可读，但 Web 会同时展示 Bible、outline、draft、facts、ReviewReport，读者或作者很容易发现冲突。

已落地防线：

- `prompts/chapter_writer_agent.md`：写正文前必须自检姓名、金额、合同/合作状态，并把冲突写入 `continuity_notes`。
- `prompts/continuity_agent.md`：把姓名、金额、合作/敌对状态不一致列为 high severity 连续性问题。
- `prompts/reviewer_agent.md`：遇到未解释的一致性硬伤时，`continuity_score` 最高 6，且 `passed = false`。
- `docs/RUBRIC.md`：新增 v0.3 一致性硬门。
- `examples/*.md`：新增 `cross_artifact_consistency` expected checks。
- `docs/UI_CONTENT_GUIDE.md`：v0.3 质量视图优先展示一致性硬门卡片。

后续建议：

- 真实 Web demo 展示前，先人工修正 `gpt-5.5 xhigh` r3 6 章样本中的旧名、权限说明和行业解释。
- 下次真实复跑时，重点检查 Bible / outline / draft / facts 的主角名、关键金额和合作状态是否统一。

最新复跑：

- `b-quality-2026-06-10-v0.3-guard` 下已完成 `gpt-5.5 xhigh` 2 章快速链路。
- novel_id：`eac2af4a-e21c-483b-b684-42be44fed943`
- `agent_run_summary total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=1164764`
- ReviewReport：`total_score=80 passed=false`
- 本轮一致性硬门生效：Reviewer 明确要求统一费用期限、暴雨/爆单时间和责任单/押金/罚款状态。

### `parse-deepseek-plot-001`

类型：`parse_error`
来源：DeepSeek 真实链路中 Plot Agent 30 章长 JSON 曾出现截断
记录位置：`docs/STAFF_REVIEW_ISSUES.md`

问题：

- 长 JSON 输出在章节大纲阶段容易截断。
- 截断会导致 JSON 解析失败，如果被 fallback 掩盖，会误判真实模型通过。

已落地防线：

- `new --outline-batch-size` 和 `outline --batch-size` 支持分批生成大纲。
- `scripts/mvp_demo.ps1` 真实模式默认用 `runs --fail-on-bad-status`。
- `runs --fail-on-bad-status` 遇到 fallback 或 parse_error 会失败。
- DeepSeek 完整真实链路历史上已复跑通过 `total=23 ok=23 fallback=0 parse_error=0`。

后续建议：

- 新 Prompt bundle 下重跑 DeepSeek 时，继续保留分批参数和 `runs --fail-on-bad-status`。
- 若再次出现截断，把 raw_text 摘要写入本文件并补对应 schema / prompt 约束。

### `provider-openai-xhigh-001`

类型：`provider_error`
来源：本地 OpenAI-compatible `cliproxyapi` + `gpt-5.5 xhigh` 长链路
记录位置：`docs/STAFF_REVIEW_ISSUES.md`

问题：

- 历史上 `write` 阶段出现 provider 子进程 `exit code -1`。
- 这类失败更像本地 provider / 代理稳定性问题，不一定是业务解析问题。

已落地防线：

- `scripts/mvp_demo.ps1` 支持 `-CheckpointResumes`。
- `new --resume-novel-id` 可复用 Market / Plot / Character / Worldbuilding AgentRun。
- `write_chapter` 续跑可复用 Writer / Continuity / Style 检查点。
- demo 输出 `work_dir` 和 `resume_novel_id`，便于手动续跑。

后续建议：

- 新 Prompt bundle 下复跑 `gpt-5.5 xhigh` 时继续使用 `-CheckpointResumes 6`。
- 如果仍出现 `exit code -1`，记录失败阶段、work_dir、resume_novel_id 和最终续跑结果。
- 不把 provider 子进程失败当成 Prompt 质量失败，除非 AgentRun 出现 parse_error 或 fallback。

最新复跑：

- `b-quality-2026-06-09-r3` 下已完成 `gpt-5.5 xhigh` 6 章链路。
- novel_id：`ea942e57-abea-42cf-8fff-287b64017b41`
- `agent_run_summary total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=733844`
- 本轮未复现 provider 子进程 `exit code -1`。

### `eval-metadata-001`

类型：`eval_process`
来源：人工评测记录容易漏写 Prompt bundle 或 AgentRun summary
记录位置：`docs/HUMAN_EVAL.md`、`docs/EVAL_LOG.md`

问题：

- 如果评测记录缺少 Prompt 版本，就无法判断差异来自模型还是 Prompt。
- 如果缺少 AgentRun summary，就无法确认本轮是否有 fallback / parse_error。

已落地防线：

- `docs/HUMAN_EVAL.md` 模板新增 `prompt_bundle`。
- `docs/PROMPT_CHANGELOG.md` 记录当前 Prompt bundle。
- `tests/smoke.rs` 新增 `eval_log_records_include_prompt_bundle_and_run_summary`。

## 4. 新增失败样例流程

新增失败样例时必须填写：

- `case_id`
- 类型：`provider_error` / `parse_error` / `quality_regression` / `eval_process`
- 来源：命令、novel_id、章节或评测记录
- 是否影响真实验收
- 已落地防线
- 下一步动作

如果样例来自真实 API，不要写入任何 key、token 或完整敏感配置。
