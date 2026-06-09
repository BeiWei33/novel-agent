# novel-agent 人工评测表

日期：2026-06-09
负责人：开发者 B
用途：对比不同 provider、prompt 版本和题材样例的生成质量。

评测记录写入 `docs/EVAL_LOG.md`。当前已记录 `gpt-5.5 xhigh` 和 DeepSeek 的都市重生第 1 章对照样本。

## 1. 使用方式

每次评测选择同一创意、同一平台、同一章节范围和同一验收命令。建议至少记录：

- provider / model / reasoning effort
- prompt bundle 或 git commit，例如 `b-quality-2026-06-09-r3`
- 题材样例
- 章节编号
- 是否出现 fallback / parse_error
- 人工评分和备注

真实模型输出如果出现 fallback 或 parse_error，本轮质量评分可以记录，但不能算作通过。

## 2. 评测元数据

| 字段 | 示例 |
| --- | --- |
| 日期 | 2026-06-09 |
| 评测人 | B |
| provider | openai / deepseek / smoke |
| model | gpt-5.5 / deepseek-v4-flash / smoke |
| reasoning_effort | xhigh / high / none |
| prompt_bundle | b-quality-2026-06-09-r3 |
| 题材 | 都市重生商业文 |
| 平台 | fanqie |
| novel_id | ... |
| chapter_index | 1 |
| 命令 | `scripts/mvp_demo.ps1 ...` |
| AgentRun 状态 | total=9 ok=9 fallback=0 parse_error=0 |

## 3. 人工评分维度

每项 1 到 5 分，3 分为可用，4 分为良好，5 分为可以进入 demo 展示。

| 维度 | 评分 | 判断问题 |
| --- | --- | --- |
| 开篇抓力 | 1-5 | 前 800 字是否快速进入目标、压力或悬念 |
| 主角主动性 | 1-5 | 主角是否主动做选择，而不是被剧情推着走 |
| 冲突密度 | 1-5 | 每 800 到 1200 字是否有新压力、反制或信息差 |
| 爽点/情绪回报 | 1-5 | 本章是否给读者明确回报 |
| 人物区分度 | 1-5 | 主要角色说话和行动是否有差异 |
| 连续性 | 1-5 | 是否违背既有事实、人物状态或世界规则 |
| 章尾钩子 | 1-5 | 结尾是否自然推动下一章 |
| 平台适配 | 1-5 | 是否符合起点/番茄/通用的节奏偏好 |
| 可编辑性 | 1-5 | 人类作者是否容易在此基础上继续改 |
| 文风自然度 | 1-5 | 是否少 AI 腔、少空泛解释、有网文语感 |

总分建议：

| 总分 | 结论 |
| --- | --- |
| 45-50 | 可作为高质量 demo |
| 38-44 | 可用，建议小修 |
| 30-37 | 需要返工后再展示 |
| < 30 | 不进入 demo，回收失败原因 |

## 4. 快速评测记录模板

```markdown
## 评测记录

- 日期：
- 评测人：
- provider / model：
- reasoning_effort：
- prompt_bundle：
- 题材 / 平台：
- novel_id：
- chapter_index：
- AgentRun summary：
- export_size：

| 维度 | 分数 | 备注 |
| --- | --- | --- |
| 开篇抓力 |  |  |
| 主角主动性 |  |  |
| 冲突密度 |  |  |
| 爽点/情绪回报 |  |  |
| 人物区分度 |  |  |
| 连续性 |  |  |
| 章尾钩子 |  |  |
| 平台适配 |  |  |
| 可编辑性 |  |  |
| 文风自然度 |  |  |

结论：

主要问题：

下一步改法：
```

## 5. 失败原因归类

| 类型 | 说明 | 下一步 |
| --- | --- | --- |
| `provider_error` | 子进程中断、超时、网络失败 | 使用检查点续跑，保留 work_dir / resume_novel_id |
| `parse_error` | JSON 不合法或 envelope 不匹配 | 收集 raw_text，调整 prompt/schema |
| `weak_opening` | 开头慢、目标不清 | 调整 Market/Writer 开头约束 |
| `weak_conflict` | 冲突弱、主角被动 | 调整 Plot/Writer 事件链 |
| `weak_payoff` | 爽点或情绪回报不足 | 调整平台策略和 Reviewer 返工标准 |
| `continuity_break` | 事实、人物状态或世界规则冲突 | 调整 Continuity Prompt 和事实输入 |
| `style_ai_tone` | AI 腔、解释堆叠、对白同质 | 调整 Style Prompt |

## 6. Demo 入选标准

进入 Web demo 的章节必须同时满足：

- `runs --fail-on-bad-status` 通过。
- ReviewReport `passed = true`。
- 人工评测总分 >= 38。
- 连续性人工分 >= 4。
- 章尾钩子人工分 >= 4。
- 没有明显抄袭、套壳或在世作者风格模仿风险。

## 7. Provider 对照判读

同题材 provider 对照时，优先比较以下差异：

| 差异 | 判读 |
| --- | --- |
| 总分差 >= 5 | 可以判定为明显质量差距，应进入 Prompt 或 provider 策略复盘 |
| 开篇抓力差 >= 2 | 检查 Market / Writer 是否给出足够强的第一冲突 |
| 冲突密度差 >= 2 | 检查 Plot / Writer 是否把机会写成阻力和反制 |
| 章尾钩子差 >= 2 | 检查结尾是否有下一章具体压力，而不是方向宣言 |
| 平台适配差 >= 2 | 检查平台模板是否被执行，而不只是写在分析里 |

对照结论必须落到下一步动作：

- 高分样本进入 Web demo 候选。
- 中分样本进入人工小修或 prompt 微调。
- 低分样本进入失败原因库，用来补 Prompt、Rubric 或 `expected_checks`。
