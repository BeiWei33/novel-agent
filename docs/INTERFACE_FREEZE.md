# MVP 接口冻结说明

冻结日期：2026-06-09

本文档用于冻结 `novel-agent` MVP 阶段 A/B 协作接口。后续字段变更必须同步更新本文档、`docs/SCHEMAS.md`、Prompt、Rust domain/storage/workflow 和回归测试。

## 1. 权威来源

| 内容 | 权威来源 | 工程实现 |
| --- | --- | --- |
| 业务输出结构 | `docs/SCHEMAS.md` | Rust domain model 或 `serde_json::Value` 映射 |
| Prompt 输入/输出格式 | `prompts/*.md` 和 `prompts/README.md` | `PromptAgent` 组装输入，`parse_agent_output` 解析输出 |
| 评分标准 | `docs/RUBRIC.md` | `ReviewScores::passes_default_line` 和 Reviewer Prompt |
| 测试样例验收 | `examples/*.md` 中的 `expected_checks` | `tests/smoke.rs` |
| 持久化结构 | Rust storage schema | `SqliteStorage::migrate` |
| REST API | `docs/API.md` | `src/api.rs` 和 `serve` CLI |

`docs/SCHEMAS.md` 是业务字段权威；Rust 可以为了工程需要增加 ID、时间戳、版本号、状态和存储索引字段，但不得改变业务字段含义。

## 2. Agent 输入输出

Prompt 文件的输入示例只描述运行时 `payload`。工程侧调用模型时统一使用：

```json
{
  "task": "create_novel | generate_outline | generate_chapter | review_chapter | rewrite_chapter | extract_facts | polish_style | check_continuity",
  "instructions": "本次调用的格式要求或重试要求",
  "payload": {},
  "context": []
}
```

所有 Agent 输出必须使用：

```json
{
  "role": "market | plot | character | worldbuilding | writer | continuity | style | reviewer",
  "structured": {},
  "raw_notes": ""
}
```

`raw_text`、`parse_error`、`attempt`、`will_fallback`、`duration_ms`、`token_usage` 是工程侧生成和保存的字段，不由模型输出。

MVP 阶段支持三个模型 provider：

- `openai`：真实模型 provider，依赖 `OPENAI_API_KEY`。
- `deepseek`：真实模型 provider，依赖 `DEEPSEEK_API_KEY`。
- `smoke`：本地确定性 provider，不访问网络，输出合法 AgentOutput envelope，用于 demo、CI 和离线链路验证。

`openai` provider 可通过 `OPENAI_BASE_URL` 指向 OpenAI-compatible 代理；`model.reasoning_effort` 为可选配置，会透传为 provider additional params，当前已验证 `gpt-5.5 + xhigh`。

`ModelClient::complete_stream` 是模型层流式接口。当前 MVP 为保证 Agent JSON 可解析，workflow 仍等待完整 `AgentOutput` 后再保存结构化结果；CLI `write --stream` 和 `rewrite --stream` 用于将已生成章节正文分块输出到终端。

## 3. 冻结结构

MVP 阶段冻结以下结构名和字段语义：

- `AgentOutputEnvelope`
- `AgentInputEnvelope`
- `FactTriple`
- `PlatformProfile`
- `NovelBible`
- `ChapterOutline`
- `CharacterCard`
- `WorldSetting`
- `ChapterDraft`
- `ContinuityReport`
- `StyleOutput.styled_chapter`
- `ReviewReport`
- `RewriteInstruction`

事实字段统一使用 `FactTriple { subject, predicate, object, importance }`。`ChapterOutline.new_facts` 表示计划事实，`ChapterDraft.new_facts` 和 `ContinuityReport.new_facts` 表示正文实际产生或复核确认的事实。

`platform_profile` 的权威持久化位置是 `NovelBible.platform_profile`。Market Agent 负责生成初稿，Plot/Writer/Reviewer 从 `NovelBible` 读取。

## 4. 工作流顺序

新书创建：

```text
Market -> Plot(batched outlines) -> Character -> Worldbuilding -> save NovelBible/Characters/Outlines/WorldSetting/Facts
```

Plot Agent 可按批次调用。工程侧会在 payload 中传入 `target_chapters`、`total_chapters`、`chapter_start`、`chapter_end`、`existing_plot_plan` 和 `previous_chapter_outlines`，并把多批 `chapter_outlines` 合并为连续章节；输出 schema 仍使用 `PlotOutput`。

章节写作：

```text
Writer -> Continuity -> Style -> save ChapterDraft/Facts/ChapterVersion
```

审稿返工：

```text
Reviewer -> RewriteNeeded? -> Writer(rewrite) -> Continuity -> Style -> save ChapterVersion -> Reviewer
```

人工编辑：

```text
read edit input file -> save ChapterDraft/ChapterVersion -> optional Review
```

版本查看：

```text
chapter_versions -> versions --novel-id <id> --chapter <n> [--show <v> | --from <a> --to <b>]
```

导出：

```text
list chapters -> export markdown
```

运行记录：

```text
agent_runs -> runs --novel-id <id> --limit <n> [--summary] [--fail-on-bad-status]
```

REST API：

```text
serve --bind 127.0.0.1:3001 -> docs/API.md 中的 /api/novels/... 路径
```

REST Jobs API：

```text
POST /api/novels/jobs
GET /api/novels/{novel_id}/facts?limit=<n>
POST /api/novels/{novel_id}/chapters/write/jobs
POST /api/novels/{novel_id}/chapters/{chapter_index}/write/jobs
GET /api/novels/{novel_id}/chapters/{chapter_index}/continuity
POST /api/novels/{novel_id}/chapters/{chapter_index}/review/jobs
POST /api/novels/{novel_id}/chapters/{chapter_index}/rewrite/jobs
GET /api/jobs/{job_id}
GET /api/jobs?limit=<n>&status=<status>&kind=<kind>&novel_id=<novel_id>&source_job_id=<job_id>
POST /api/jobs/{job_id}/retry
POST /api/jobs/{job_id}/cancel
```

当前 jobs 写入 SQLite `api_jobs` 表，状态固定为 `queued`、`running`、`succeeded`、`failed`、`cancelled`。`payload` 字段保存任务创建参数，`source_job_id` 对普通任务为 `null`、对 retry 新任务为源失败任务 id，`progress_current` / `progress_total` 表示任务进度，任务成功后的 `result` 字段复用同步接口同形 DTO，例如 `{ "draft": {} }` 或 `{ "report": {} }`。MVP 会保留历史任务记录；进程退出时尚未完成的任务不会自动恢复执行，下次 `serve` 启动会把遗留 `queued` / `running` 任务标记为 `failed`。
`GET /api/jobs` 的 `status` / `kind` / `novel_id` / `source_job_id` 为可选筛选参数；`status` 非法时必须返回 `400 Bad Request`。
`retry` 只接受 `failed` 源任务，并基于源任务 `payload` 创建新的 job，不覆盖原任务记录；新 job 的 `source_job_id` 必须指向源任务。
`cancel` 只接受 `queued` / `running` 源任务，并把任务标记为终态 `cancelled`；后续后台完成或失败写回不得覆盖取消终态。
批量章节写作 job 路径为 `POST /api/novels/{novel_id}/chapters/write/jobs`，请求字段固定为 `chapter_start` / `chapter_end`。该 job 的 `kind` 为 `write_chapters`，`chapter_index` 为 `null`，`progress_total` 为章节数量，`payload` 保存 `chapter_start`、`chapter_end` 和 `chapter_indexes`，成功 `result` 固定包含 `chapter_start`、`chapter_end` 和 `drafts`。批量写作按章节顺序复用单章 workflow，任一章节完成后推进 `progress_current`；任一章节失败则 job 进入 `failed` 并保留当前进度，已保存章节保留。
`GET /api/novels/{novel_id}/facts` 返回作品事实列表，支持 `limit`；`GET /api/novels/{novel_id}/chapters/{chapter_index}/continuity` 返回该章节最新连续性报告，报告内容保持 Continuity Agent 结构化 JSON。
`GET /api/novels/{novel_id}/runs` 支持 `limit` / `role` / `task` / `status` 可选筛选参数；`status` 固定为 `ok`、`fallback`、`parse_error`，非法值必须返回 `400 Bad Request`，`summary` 按筛选后的 runs 计算。

SSE API：

```text
write/stream 与 rewrite/stream 事件名固定为 started、chapter_chunk、completed
chapter_chunk.data 固定包含 operation、chapter_index、chunk_index、text
completed.data 固定包含 operation、draft
```

## 5. 持久化约定

MVP 阶段必须持久化：

- `novels`
- `novel_bibles`
- `characters`
- `chapters`
- `chapter_versions`
- `facts`
- `world_settings`
- `continuity_reports`
- `review_reports`
- `agent_runs`

同一章节保存草稿时，`chapters` 保存最新版本，`chapter_versions` 保存版本快照，并清空旧审稿分数，等待新一轮审稿写回。章节事实采用“删除同章旧 facts 后插入新 facts”的替换策略，避免重复污染上下文。`edit` 保存人工编辑稿时不调用 Agent、不生成 `agent_runs`、不自动刷新 facts；它只复用 `ChapterDraft` 和 `chapter_versions` 版本通道。

## 6. 变更规则

任一冻结字段发生变化时，必须同时完成：

1. 更新 `docs/SCHEMAS.md`。
2. 更新相关 `prompts/*.md`。
3. 更新 Rust domain 或 JSON 映射。
4. 更新 SQLite schema 或迁移逻辑。
5. 更新 `tests/smoke.rs` 和 `examples/*.md` 的 `expected_checks`。
6. 更新 `docs/STAFF_REVIEW_ISSUES.md` 处理记录。
7. 运行验证命令：

```powershell
& $HOME\.cargo\bin\cargo.exe check
& $HOME\.cargo\bin\cargo.exe test
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek
cargo run -- serve --bind 127.0.0.1:3001
powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
```

模型真实链路验收时使用：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -UseRealModel
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 2 -OutlineChapters 2 -NewOutlineBatchSize 1 -OutlineBatchSize 1 -SkipOutline -SkipRewrite -StepRetries 0
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel -NewChapters 6 -SkipOutline -SkipRewrite
```

真实模型验收要求环境中存在对应 provider 的 key：OpenAI 使用 `OPENAI_API_KEY`，OpenAI-compatible 代理可额外设置 `OPENAI_BASE_URL`，DeepSeek 使用 `DEEPSEEK_API_KEY`。`-UseRealModel` 模式会拒绝在缺少 key 时继续运行，并且一旦 AgentRun 出现 fallback 或 parse error，本次真实模型验收应视为失败。完整 demo 默认按 5 章一批调用 Plot Agent，并用 `runs --limit 80 --summary --fail-on-bad-status` 检查本次链路主要 AgentRun；DeepSeek 可先用 `-NewChapters 6 -SkipOutline -SkipRewrite` 验证短链路，OpenAI-compatible `gpt-5.5 + xhigh` 可先用 2 章无重试快速链路验证。

## 7. 当前验收状态

当前 MVP 离线 smoke 验收已覆盖：

- 非法 JSON fallback 和 `parse_error` 保存。
- 三个题材样例的语义 `expected_checks`。
- Worldbuilding、Continuity、Style 真实 workflow 接入。
- 低分审稿、重写、版本快照、返工后复审。
- AgentOutput envelope 严格校验，裸 JSON 或角色不匹配会进入 parse fallback。
- AgentRun `_engineering` 已记录 `duration_ms`，并预留 `token_usage`。
- `provider = "smoke"` 已支持本地合法 envelope 链路，不依赖 fallback。
- `provider = "deepseek"` 已接入 Rig DeepSeek provider，真实验收使用 `DEEPSEEK_API_KEY`。
- `provider = "openai"` 已支持 `OPENAI_BASE_URL` 指向本地 OpenAI-compatible 代理；`reasoning_effort` 可配置为 `xhigh`。
- `ModelClient::complete_stream` 已提供流式 chunks 接口，`write/rewrite --stream` 已支持正文分块输出。
- `runs` CLI 已可查看最近 AgentRun，展示角色、任务、尝试次数、状态、耗时和 token；`--summary` 会输出状态、耗时和 token 汇总，`--fail-on-bad-status` 会在存在 fallback 或 parse_error 时返回失败；真实 demo 默认检查最近 80 条运行记录。
- `serve` CLI 已可启动本地 REST API；API smoke test 覆盖作品创建、列表、章节生成、审稿、SSE、后台任务、Markdown 导出、CORS preflight 和 AgentRun 查询。
- `scripts/api_demo.ps1` 已可启动临时 API 服务并调用 CORS、作品创建、章节生成、审稿、SSE、后台任务、Markdown 导出和 AgentRun 查询。
- `versions` CLI 已可查看章节版本快照，并输出基础版本对比。
- `edit` CLI 已可从本地文件保存人工编辑稿为新版本，并可继续用 `versions` 对比、用 `review` 复审。
- `scripts/mvp_demo.ps1 -UseRealModel` 已具备 key preflight 和 `runs --fail-on-bad-status` 坏状态检测。
- CLI `new` 支持 `--chapters` 和 `--outline-batch-size`，`outline` 支持 `--batch-size`，默认 5 章一批，用于降低 Plot Agent 长 JSON 截断风险。
- Plot Agent 分批输出已支持区间过滤、绝对章号矫正和缺章 fallback，默认 30 章会拆为多次短输出。
- DeepSeek 短链路 `-NewChapters 6 -SkipOutline -SkipRewrite` 已完成真实 API 验证，`agent_runs.parse_error = 0`。
- DeepSeek 完整真实链路已通过，`agent_run_summary total=23 ok=23 fallback=0 parse_error=0`。
- 本地 cliproxyapi 已验证 `gpt-5.5 + reasoning_effort=xhigh` 2 章快速无重试链路，`agent_run_summary total=9 ok=9 fallback=0 parse_error=0`。
- 本地 cliproxyapi 的 6 章链路已分段验证到 `new -> write -> review -> export -> runs`，最终 `agent_run_summary total=9 ok=9 fallback=0 parse_error=0`；一次性 demo 脚本仍观察到 provider 子进程 `exit code -1`，需继续复跑确认稳定性。
- CLI `new -> outline -> write -> review -> rewrite -> versions -> edit -> versions -> export -> runs` 闭环。
