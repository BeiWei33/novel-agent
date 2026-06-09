# MVP 交付验收说明

验收日期：2026-06-09

本文档用于作为 `novel-agent` CLI MVP 的交付入口。接口冻结规则见 `docs/INTERFACE_FREEZE.md`，详细问题处理记录见 `docs/STAFF_REVIEW_ISSUES.md`。

## 1. 当前结论

当前 MVP 离线工程链路已可验收：

- 能创建小说项目。
- 能生成小说圣经、核心人物、世界观和前 30 章大纲。
- 能生成第 1 章正文，并执行 Continuity 和 Style。
- 能审稿、给出评分和返工指令。
- 能低分返工、保存章节版本快照，并返工后自动复审。
- 能查看章节版本列表、单个版本正文和 v1/v2 基础对比。
- 能通过 `edit` 从本地文件保存人工编辑稿为新版本，并继续进入 `versions` / `review` 闭环。
- 能分批调用 Plot Agent 生成 30 章大纲，降低真实模型长 JSON 截断风险。
- 能写入 SQLite：novels、novel_bibles、characters、chapters、chapter_versions、facts、world_settings、continuity_reports、review_reports、agent_runs、api_jobs。
- 能导出 Markdown。
- 能查看最近 AgentRun。
- 能查看 AgentRun 状态、耗时和 token 汇总。
- 能启动本地 REST API，并通过 API 创建作品、生成章节、审稿、导出 Markdown、查看后台任务、查看 AgentRun 和接收 SSE 章节正文事件。
- 能使用本地 `smoke` provider 做稳定离线验收。
- 能配置 `openai` / `deepseek` 真实 provider，并具备 key preflight 和 fallback 误通过防护。
- 能通过 `OPENAI_BASE_URL` 接入本地 OpenAI-compatible 代理；已验证 `gpt-5.5 + reasoning_effort=xhigh` 最小真实链路。

## 2. 已验证命令

```powershell
& $HOME\.cargo\bin\cargo.exe check
& $HOME\.cargo\bin\cargo.exe test
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
```

最近一次已确认结果：

```text
cargo check ok
cargo test: api/storage unit 3 passed; smoke tests 14 passed; 0 failed
mvp_demo.ps1: new / outline / write / review / rewrite / versions / edit / versions / export / runs all ok
mvp_demo.ps1 -StreamWrite: write / rewrite stream output observed; edit and v2/v3 compare observed
api_demo.ps1: CORS / create / list / write / review / SSE / jobs / export / runs all ok
```

API smoke test 已覆盖：

- `POST /api/novels`
- `GET /api/novels`
- `POST /api/novels/{novel_id}/chapters/1/write`
- `GET /api/novels/{novel_id}/facts?limit=10`
- `GET /api/novels/{novel_id}/chapters/1/continuity`
- `POST /api/novels/{novel_id}/chapters/1/review`
- `POST /api/novels/{novel_id}/chapters/2/write/stream`
- `POST /api/novels/{novel_id}/chapters/3/write/jobs`
- `POST /api/novels/{novel_id}/chapters/write/jobs`
- `GET /api/jobs/{job_id}`
- `GET /api/jobs?limit=10`
- `GET /api/jobs?limit=10&status=succeeded&kind=write_chapters&novel_id=<novel_id>`
- API 单元测试额外覆盖重建 router 后仍可查询已完成 job。
- API 单元测试额外覆盖 facts 列表和章节最新连续性报告查询。
- API 单元测试额外覆盖 jobs 列表按 `status` / `kind` / `novel_id` / `source_job_id` 筛选，以及非法 `status` 返回 400。
- API 单元测试额外覆盖批量章节 job 成功返回 `drafts` 和 `progress_current/progress_total`、failed 批量章节 job retry 创建带 `source_job_id` 的新 job 并执行成功、queued job cancel 进入 `cancelled`、已完成 job cancel 返回 400；API demo 覆盖批量章节 job 进度、非 failed job retry 和 completed job cancel 返回 400。
- Storage 单元测试额外覆盖旧 `api_jobs` 表补 `payload` / `source_job_id` / progress 列、批量进度推进、取消终态不被 running/complete 写回覆盖，以及 `serve` 启动前可将遗留未完成 job 标为 `failed`。
- `GET /api/novels/{novel_id}/export/markdown`
- `GET /api/runs?limit=20&novel_id=<novel_id>&role=writer&task=generate_chapter&status=ok`
- `GET /api/novels/{novel_id}/runs?limit=50`
- `GET /api/novels/{novel_id}/runs?limit=20&role=writer&task=generate_chapter&status=ok`
- CORS preflight

## 3. Demo 口径

默认离线 demo：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
```

默认使用 `provider = "smoke"`，不需要 API key，不访问网络。该路径用于工程验收、CI 和接口回归。

流式正文演示：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
```

OpenAI fallback 分支：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai
```

DeepSeek fallback 分支：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek
```

## 4. 真实模型验收

OpenAI：

```powershell
$env:OPENAI_API_KEY = "..."
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -UseRealModel
```

DeepSeek：

```powershell
$env:DEEPSEEK_API_KEY = "..."
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel
```

本地 OpenAI-compatible 代理（cliproxyapi / gpt-5.5 xhigh）：

```powershell
$env:OPENAI_BASE_URL = "http://127.0.0.1:8317/v1"
$env:OPENAI_API_KEY = "<cliproxy bearer token>"
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel
```

本地 OpenAI-compatible 快速真实链路：

```powershell
$env:OPENAI_BASE_URL = "http://127.0.0.1:8317/v1"
$env:OPENAI_API_KEY = "<cliproxy bearer token>"
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 2 -OutlineChapters 2 -NewOutlineBatchSize 1 -OutlineBatchSize 1 -SkipOutline -SkipRewrite -StepRetries 0
```

DeepSeek 快速真实链路：

```powershell
$env:DEEPSEEK_API_KEY = "..."
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel -NewChapters 6 -SkipOutline -SkipRewrite
```

真实模型验收必须满足：

- 当前进程能读取对应 API key。
- CLI 输出中不能出现 fallback、Agent 调用失败或解析失败提示。
- demo 默认用 `runs --limit 80 --summary --fail-on-bad-status` 覆盖本次链路主要 AgentRun，真实模式下脚本会检查 fallback 或 parse error 并判失败。
- `runs` 输出中本次链路状态应为 `ok`。
- `agent_runs.parse_error` 不应出现本次真实链路的新错误。
- 完整验收中，`versions --from 1 --to 2` 能看到重写前后版本对比。
- 完整验收中，`edit` 后 `versions --from 2 --to 3` 能看到人工编辑版本对比。
- 完整验收默认使用 `-NewOutlineBatchSize 5` 和 `-OutlineBatchSize 5` 分批生成大纲。
- 快速真实链路可跳过重复 `outline` 和 `rewrite`，用于先验证真实 provider 能稳定返回合法 AgentOutput。

当前环境说明：DeepSeek key 已可见；完整 DeepSeek demo 已通过，`agent_run_summary total=23 ok=23 fallback=0 parse_error=0`。本地 cliproxyapi 已验证 `/v1/models`、`chat/completions`、`gpt-5.5 + reasoning_effort=xhigh`，并用 novel-agent 2 章快速无重试链路跑通，`agent_run_summary total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=799126`。6 章链路已完成 `new -> write -> review -> export -> runs` 分段验证，最终 `agent_run_summary total=9 ok=9 fallback=0 parse_error=0 duration_ms_total=700385`、`export_size=19347`；但一次性 demo 脚本在首次 review 调用处出现过 provider 子进程 `exit code -1`，因此 6 章严格一次性验收仍需继续压测。

## 5. 已知边界

- `smoke` provider 是确定性本地 provider，只用于工程链路和接口回归，不代表真实模型文本质量。
- `openai` / `deepseek` 的 token usage 当前仍由 provider 能力决定；本地 `smoke` provider 会写入粗略估算。
- `ModelClient::complete_stream` 已提供统一流式接口；真实 provider 当前可先通过完整响应拆块输出，后续可替换为 Rig 原生 streaming。
- `edit` 保存人工编辑稿时不会自动抽取新 facts；需要继续执行 `review` / `rewrite` 或后续专门的事实抽取流程来刷新质量状态。
- Plot 分批能降低单次输出长度，但真实模型仍可能因 provider 波动、格式偏移或提示理解偏差产生 parse error；完整真实验收以 `agent_runs.parse_error = 0` 为准。
- MVP 只做基础事实表、连续性报告和章节版本；复杂向量检索、Web UI、多用户和自动发布不在当前范围。

## 6. 下一步建议

- 对 `gpt-5.5 + reasoning_effort=xhigh` 继续执行 6 章一次性脚本或完整默认链路压测，确认 provider 子进程中断是否仍会复现。
- 将真实模型输出失败样例整理进 Prompt/Schema 回归。
- 做一次提交前文件清单复核，确认本轮 A/B 改动均应纳入同一提交或拆分提交。
