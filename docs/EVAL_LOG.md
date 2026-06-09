# novel-agent 人工评测记录

本文档记录按 `docs/HUMAN_EVAL.md` 执行过的人工评测样例。评分为人工主观判断，用于 prompt 迭代和 provider 对比，不替代 `runs --fail-on-bad-status`。

## Provider 对照摘要

| 样本 | 人工总分 | Demo 状态 | 主要优势 | 主要风险 | 后续动作 |
| --- | --- | --- | --- | --- | --- |
| `gpt-5.5 xhigh` 都市重生第 1 章 | 44 / 50 | 可用，小修后进入 Web demo | 开篇压力、主角主动性、章尾事故钩子强 | 跑单中段略重复，欠薪金额线可更清楚 | 作为当前推荐质量基线 |
| DeepSeek 历史都市重生第 1 章 | 36 / 50 | 需要返工后再展示 | 主线清楚，商业目标和结构可编辑 | 即时压力不足，谈判偏顺，章尾偏方向宣言 | 作为 provider 对照和 Prompt 回归反例 |

当前判断：

- `gpt-5.5 xhigh` 样本更适合直接给 C 侧做 Web demo 内容基线。
- DeepSeek 历史样本更适合做负向回归样本，重点检查外部阻力、失败代价和章尾具体压力。
- 新一轮 provider 对比必须继续使用同题材、同平台、同章节范围，并单独记录是否使用了新版 Prompt。
- 当前 Prompt bundle 为 `b-quality-2026-06-09-r3`；本页两条历史真实样本早于 r3，不应和 r3 新输出做无标注直接对比。

## 2026-06-09 gpt-5.5 xhigh 都市重生 6 章短链路

- 评测人：B
- provider / model：openai-compatible / gpt-5.5
- reasoning_effort：xhigh
- prompt_bundle：历史样本，早于 `b-quality-2026-06-09-r3`
- 题材 / 平台：都市重生商业文 / fanqie
- novel_id：cb7e206a-9fae-4ef0-9471-05f055e4ff4f
- chapter_index：1
- 章节标题：重生第一天，先撕罚款单
- 验收命令：`powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 6 -OutlineChapters 6 -NewOutlineBatchSize 3 -OutlineBatchSize 3 -SkipOutline -SkipRewrite -StepRetries 0 -CheckpointResumes 6`
- AgentRun summary：`total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=639102`
- export_size：20616
- ReviewReport：`total_score=88 passed=true`

| 维度 | 分数 | 备注 |
| --- | --- | --- |
| 开篇抓力 | 5 | 车祸残响、母亲押金、罚款表和重生时间点连得很快，前 800 字内压力明确。 |
| 主角主动性 | 5 | 主角主动撕罚款单、立赌约、规划短单路线，并明确阻止周强接危险单。 |
| 冲突密度 | 4 | 赵德旺克扣、押金催缴、冲榜赌约、危险远单连续推进；中段跑单过程略长。 |
| 爽点/情绪回报 | 4 | 两小时冲到第一、逼赵德旺先吐出一千八，有现实爽点；欠薪线可再算清。 |
| 人物区分度 | 4 | 林川、赵德旺、周强有清晰立场；部分骑手和商家仍偏功能性。 |
| 连续性 | 4 | 重生信息、押金、周强事故、外卖补贴规则自洽；未来记忆偏差还可更早出现。 |
| 章尾钩子 | 5 | 周强收到上一世事故路线远单，赵德旺逼单，下一章行动方向很硬。 |
| 平台适配 | 5 | 番茄向开篇冲突快、现实压力强、每段目标清楚，适合短周期追读。 |
| 可编辑性 | 4 | 结构完整，人工主要做压缩和补压力线，不需要大改主事件。 |
| 文风自然度 | 4 | 表达顺，场景清楚，少明显 AI 腔；跑单段可减少重复动作描写。 |

总分：44 / 50

结论：可用，建议小修后进入 Web demo。该章符合都市重生商业文模板，尤其是现实压力、主角主动选择、短周期回报和章尾事故压力都较稳。

主要问题：

- 跑单中段动作略重复，医院三单、学校两单、写字楼四单可以保留最精彩的一组，其余用榜单跳动和群聊反应带过。
- 欠薪金额线可以更清楚，例如补一句林川估算他和周强至少被扣三千六，赵德旺只先吐出一半。
- 母亲押金倒计时可以在区域群通知前再压一次，让赚钱线和救亲线绑定更紧。
- 未来记忆目前较准，下一章应加入路线封路、后台改派或商家出餐慢等偏差，避免主角只像背答案。

下一步改法：

- Writer Prompt 已补充“重生/预知类主角必须出现变量偏差或临场补救”。
- Reviewer Prompt 已要求对章尾自然延伸和人物行为一致性设扣分上限。
- DeepSeek 同题材历史样本已完成对照，差异已反灌到 Prompt、Rubric 和 `examples/urban_rebirth.md` 回归断言。

## 2026-06-09 DeepSeek 历史真实输出 都市重生完整链路

- 评测人：B
- provider / model：deepseek / deepseek-chat
- reasoning_effort：none
- prompt_bundle：历史样本，早于 `b-quality-2026-06-09-r3`
- 题材 / 平台：都市重生商业文 / fanqie
- novel_id：ebc0233d-278f-436f-94f7-6935e089c6ae
- chapter_index：1
- 章节标题：重生：五百块与一个未来
- 样本来源：历史真实 DeepSeek 完整 demo 输出；本轮未重新发起 DeepSeek API 调用
- 验收命令：`powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel`
- AgentRun summary：`total=23 ok=23 fallback=0 parse_error=0 duration_ms_total=613501 tokenized_runs=0`
- export_size：25792
- ReviewReport：未重新读取；本条以人工质量评测为主

| 维度 | 分数 | 备注 |
| --- | --- | --- |
| 开篇抓力 | 4 | 被害后重生、旧手机日期和五百块余额能建立悬念；进入商业规划后张力下降。 |
| 主角主动性 | 4 | 主角主动联系站长、盘下外卖站、谈奶茶店合作，目标明确。 |
| 冲突密度 | 3 | 电话、谈判和资金压力都有，但中段偏计划说明，缺少连续升级的即时阻力。 |
| 爽点/情绪回报 | 3 | 拿到站点转让口头机会有商业爽点，但本章回报偏软，还没有形成强反杀或强收益。 |
| 人物区分度 | 3 | 刘胖子、站长和奶茶店老板有基本功能差异，但对白和动机还可更鲜明。 |
| 连续性 | 4 | 重生日期、资金、站点转让和未来外卖趋势自洽，暂未看到明显事实冲突。 |
| 章尾钩子 | 3 | “五百块起家”的方向清楚，但下一章必须立刻解决的外部压力不够硬。 |
| 平台适配 | 3 | 都市重生商业题材适配番茄，但开篇节奏和短周期冲突不如 `gpt-5.5 xhigh` 样本紧。 |
| 可编辑性 | 5 | 结构完整，主线清楚，适合通过补压力、补对手和补即时回报快速改进。 |
| 文风自然度 | 4 | 叙述顺滑，少明显格式腔；商业解释段需要压短并转成行动场面。 |

总分：36 / 50

结论：需要返工后再展示。该章主线可用，商业目标清楚，但番茄向第一章的即时压力、外部冲突和章尾硬钩子不足；更适合作为 DeepSeek provider 的质量对照基线，而不是直接进入 Web demo。

主要问题：

- 开头有“死亡重生”强信息，但后续较快转入未来趋势解释，现实危机没有持续压住主角。
- 谈站点和谈合作的过程偏顺，缺少对手反压、时间限制或失败代价。
- 五百块余额是好钩子，但结尾更像方向宣言，缺少下一章必须处理的具体事件。
- 配角多承担推进功能，语言风格和利益诉求还不够分明。

下一步改法：

- Writer Prompt 已补充“开头 800 字必须持续给目标、压力或悬念”，该样本可作为回归对照。
- Platform Templates 已补充番茄向前三章节奏，后续 DeepSeek 新跑应重点观察第一章即时冲突和章尾事故/订单/资金压力。
- 本轮已把“商业谈判不能过顺、未来趋势不能替代场景压力、章尾不能只做方向宣言”反灌到 Market / Plot / Writer / Reviewer Prompt。
- 若重新跑 DeepSeek，应使用同题材、同章节数和 `runs --fail-on-bad-status`，再把新样本与本条历史基线分开记录。
