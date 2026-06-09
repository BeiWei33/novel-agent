# novel-agent REST API

版本：v0.1
日期：2026-06-09

本文档描述 A 侧提供给 Web 工作台的 P0 API。默认本地启动方式：

```powershell
cargo run -- serve --bind 127.0.0.1:3001
```

默认配置文件仍为 `novel-agent.toml`；可用 `--config <path>` 指定临时配置。

本地 API demo：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
```

脚本会自动启动临时 `serve` 进程并调用本文档的核心 P0/P1 接口。

## 通用约定

- Base URL：`http://127.0.0.1:3001`
- 请求和响应均为 JSON。
- 时间字段使用 RFC3339 字符串。
- `platform` 可传 `general`、`qidian`、`fanqie`。
- API router 已启用 permissive CORS，支持 Web 工作台本地开发跨端口访问。
- 错误响应格式：

```json
{
  "error": {
    "code": "not_found",
    "message": "chapter 1 for novel `...` was not found"
  }
}
```

## Health

```http
GET /health
```

响应：

```json
{
  "status": "ok",
  "service": "novel-agent",
  "version": "0.1.0",
  "checked_at": "2026-06-09T00:00:00Z",
  "sse": true
}
```

## 模型配置

```http
GET /api/model
PUT /api/model
```

`GET /api/model` 返回当前 API 进程后续 workflow 会使用的模型配置。`PUT /api/model` 会在线切换模型配置，影响切换之后触发的新建、生成、审稿、重写和后台 job；已经启动的后台任务继续使用任务启动时捕获的模型客户端。`provider` 支持 `smoke`、`openai`、`deepseek`，并兼容 `local` / `offline` 作为 `smoke` 别名。`reasoning_effort` 只对 `openai` provider 生效，其他 provider 会返回 `null`。
`pricing` 是可选的用户配置价格，单位为每百万 token 的 microUSD；系统不会内置或推断真实供应商价格。只有同时提供 prompt 和 completion 两个价格时，新产生的 AgentRun 才会保存价格并计算成本；未配置、配置不完整或历史旧记录的成本字段返回 `null`，汇总中按 0 计。

请求：

```json
{
  "provider": "smoke",
  "model": "smoke",
  "reasoning_effort": null,
  "pricing": {
    "prompt_cost_micro_usd_per_million_tokens": 1000000,
    "completion_cost_micro_usd_per_million_tokens": 2000000
  }
}
```

响应：

```json
{
  "model": {
    "provider": "smoke",
    "model": "smoke",
    "reasoning_effort": null,
    "pricing": {
      "prompt_cost_micro_usd_per_million_tokens": 1000000,
      "completion_cost_micro_usd_per_million_tokens": 2000000
    }
  }
}
```

## 后台任务

当前 jobs 记录写入 SQLite `api_jobs` 表，用于给 Web 工作台提供非阻塞调用入口。服务重启后已完成或失败的任务记录仍可查询；如果进程在任务运行中退出，下次 `serve` 启动会把遗留的 `queued` / `running` 任务标记为 `failed`，MVP 暂不自动恢复执行该任务。

```http
GET /api/jobs?limit=50
GET /api/jobs/{job_id}
POST /api/jobs/{job_id}/retry
POST /api/jobs/{job_id}/cancel
```

`GET /api/jobs` 支持可选筛选参数：

```http
GET /api/jobs?limit=50&status=running&kind=write_chapters&novel_id=...&source_job_id=...
```

`status` 必须是 `queued`、`running`、`succeeded`、`failed`、`cancelled` 之一；非法值返回 `400 Bad Request`。`kind` 可传 `create_novel`、`write_chapter`、`write_chapters`、`review_chapter`、`rewrite_chapter` 等任务类型。`novel_id` 用于筛选当前作品任务；`source_job_id` 用于筛选某个失败源任务创建出的 retry job。

响应分别为：

```json
{
  "jobs": []
}
```

```json
{
  "job": {
    "id": "...",
    "kind": "write_chapter",
    "status": "queued",
    "novel_id": "...",
    "chapter_index": 1,
    "source_job_id": null,
    "progress_current": 0,
    "progress_total": 1,
    "payload": {
      "novel_id": "...",
      "chapter_index": 1
    },
    "result": null,
    "error": null,
    "created_at": "2026-06-09T00:00:00Z",
    "updated_at": "2026-06-09T00:00:00Z"
  }
}
```

`status` 固定为 `queued`、`running`、`succeeded`、`failed`、`cancelled` 之一。`source_job_id` 对普通创建任务为 `null`；对 retry 创建的新任务为源失败任务 id，便于 UI 串起失败任务和重试任务。`progress_current` / `progress_total` 用于 UI 展示任务进度；普通任务默认 `0/1`，成功后为 `1/1`，批量任务会按已完成章节数推进。`payload` 保存创建任务时的请求参数，便于 UI 展示和重试预填。任务成功后 `result` 会包含同步接口同形结果，例如写作和重写任务为 `{ "draft": {} }`，审稿任务为 `{ "report": {} }`。

`POST /api/jobs/{job_id}/retry` 仅允许重试 `failed` 状态的任务。重试不会覆盖原任务，而是基于原任务 `payload` 创建一个新的 `queued` job，新 job 的 `source_job_id` 指向源任务，并返回 `202 Accepted`。非 `failed` 任务会返回 `400 Bad Request`。

`POST /api/jobs/{job_id}/cancel` 仅允许取消 `queued` / `running` 状态的任务，成功后返回当前 job，`status` 为 `cancelled`、`result` 为 `null`、`error` 为取消原因。取消后的 job 是终态；后台任务即使稍后完成，也不会覆盖 `cancelled` 状态。对 `succeeded`、`failed` 或 `cancelled` 任务再次取消会返回 `400 Bad Request`。

可异步创建的任务：

```http
POST /api/novels/jobs
POST /api/novels/{novel_id}/chapters/write/jobs
POST /api/novels/{novel_id}/chapters/{chapter_index}/write/jobs
POST /api/novels/{novel_id}/chapters/{chapter_index}/review/jobs
POST /api/novels/{novel_id}/chapters/{chapter_index}/rewrite/jobs
```

所有任务创建成功均返回 `202 Accepted`：

```json
{
  "job": {
    "id": "...",
    "kind": "create_novel",
    "status": "queued",
    "novel_id": null,
    "chapter_index": null,
    "source_job_id": null,
    "progress_current": 0,
    "progress_total": 1,
    "payload": {
      "idea": "...",
      "platform": "fanqie",
      "chapters": 30,
      "outline_batch_size": 5
    },
    "result": null,
    "error": null,
    "created_at": "2026-06-09T00:00:00Z",
    "updated_at": "2026-06-09T00:00:00Z"
  }
}
```

批量章节写作 job 使用范围请求，最多一次 50 章：

```http
POST /api/novels/{novel_id}/chapters/write/jobs
```

请求：

```json
{
  "chapter_start": 4,
  "chapter_end": 8
}
```

返回 job 的 `kind` 为 `write_chapters`，`chapter_index` 为 `null`，`progress_total` 为章节数，`payload` 会保存 `chapter_start`、`chapter_end` 和展开后的 `chapter_indexes`。成功后 `progress_current == progress_total`，`result` 为：

```json
{
  "chapter_start": 4,
  "chapter_end": 8,
  "drafts": []
}
```

批量 job 会顺序复用单章写作 workflow，因此每章仍会保存章节正文、版本、连续性报告和 facts。每完成一章，`progress_current` 会加一。任一章节失败时，job 会进入 `failed`，并保留当前进度；已成功写入的前序章节会保留。`cancel` 对批量 job 生效后，不会再开始下一章，且后台完成结果不会覆盖 `cancelled` 终态。`retry` 支持 `write_chapters`，会按原 `payload` 重新创建同范围的新 job，并用 `source_job_id` 指向源失败任务。

## 作品

```http
GET /api/novels?limit=50
```

返回最近更新的作品列表：

```json
{
  "novels": []
}
```

```http
POST /api/novels
```

请求：

```json
{
  "idea": "都市重生商业文，主角回到十年前从外卖站逆袭",
  "platform": "fanqie",
  "chapters": 30,
  "outline_batch_size": 6
}
```

响应 `201 Created`：

```json
{
  "novel": {},
  "bible": {},
  "characters": [],
  "outlines": [],
  "used_fallback": false
}
```

```http
GET /api/novels/{novel_id}
```

返回作品详情、Bible、角色、章节、世界设定和事实：

```json
{
  "novel": {},
  "bible": {},
  "characters": [],
  "chapters": [],
  "world_setting": {},
  "facts": []
}
```

作品资料拆分查询：

```http
GET /api/novels/{novel_id}/bible
GET /api/novels/{novel_id}/characters
GET /api/novels/{novel_id}/world-settings
```

响应分别为：

```json
{
  "bible": {}
}
```

```json
{
  "characters": []
}
```

```json
{
  "world_setting": {}
}
```

作品事实：

```http
GET /api/novels/{novel_id}/facts?limit=100
```

响应：

```json
{
  "facts": [
    {
      "id": "...",
      "novel_id": "...",
      "chapter_id": null,
      "subject": "林舟",
      "predicate": "确认",
      "object": "自己回到命运转折点",
      "importance": 3,
      "created_at": "2026-06-09T00:00:00Z"
    }
  ]
}
```

## 大纲

```http
GET /api/novels/{novel_id}/outline
POST /api/novels/{novel_id}/outline
```

`GET` 只读取当前已保存章节大纲，不调用模型；`POST` 会按请求体重新生成大纲。

请求：

```json
{
  "chapters": 30,
  "batch_size": 6
}
```

响应：

```json
{
  "outlines": []
}
```

## 章节

```http
GET /api/novels/{novel_id}/chapters
GET /api/novels/{novel_id}/chapters/{chapter_index}
```

响应分别为：

```json
{
  "chapters": []
}
```

```json
{
  "chapter": {}
}
```

生成章节：

```http
POST /api/novels/{novel_id}/chapters/{chapter_index}/write
POST /api/novels/{novel_id}/chapters/{chapter_index}/write/stream
```

普通响应：

```json
{
  "draft": {}
}
```

SSE 响应使用 `text/event-stream`，事件顺序为：

```text
event: started
data: {"operation":"write","chapter_index":1,"version":1}

event: chapter_chunk
data: {"operation":"write","chapter_index":1,"chunk_index":0,"text":"..."}

event: completed
data: {"operation":"write","draft":{}}
```

人工保存章节：

```http
PUT /api/novels/{novel_id}/chapters/{chapter_index}/edit
PUT /api/novels/{novel_id}/chapters/{chapter_index}/content
```

`/content` 是 Web 工作台早期交接文档的兼容别名，请求和响应与 `/edit` 完全一致。

请求：

```json
{
  "title": "第一章 人工修订",
  "content": "人工编辑后的章节正文",
  "summary": "人工保存后补强目标和伏笔"
}
```

`title` 和 `summary` 可省略；`content` 不能为空，否则返回 `400 Bad Request`。该接口不调用 Agent、不生成 `agent_runs`、不自动刷新 facts；它会保存新的章节版本，并清空旧审稿分数，等待后续复审。

响应：

```json
{
  "draft": {}
}
```

审稿：

```http
POST /api/novels/{novel_id}/chapters/{chapter_index}/review
GET /api/novels/{novel_id}/chapters/{chapter_index}/review
GET /api/novels/{novel_id}/chapters/{chapter_index}/continuity
```

审稿响应分别为：

```json
{
  "report": {}
}
```

```json
{
  "chapter": {},
  "report": {}
}
```

连续性报告响应：

```json
{
  "chapter": {},
  "report": {
    "passed": true,
    "issues": [],
    "new_facts": [],
    "character_state_updates": [],
    "foreshadowing_updates": []
  }
}
```

重写：

```http
POST /api/novels/{novel_id}/chapters/{chapter_index}/rewrite
POST /api/novels/{novel_id}/chapters/{chapter_index}/rewrite/stream
```

普通响应：

```json
{
  "draft": {}
}
```

SSE 响应与写章节一致，`operation` 为 `rewrite`。

## 版本

```http
GET /api/novels/{novel_id}/chapters/{chapter_index}/versions
GET /api/novels/{novel_id}/chapters/{chapter_index}/versions/{version}
```

响应分别为：

```json
{
  "novel_id": "...",
  "chapter_id": "...",
  "chapter_index": 1,
  "versions": [1, 2]
}
```

```json
{
  "novel_id": "...",
  "chapter_id": "...",
  "chapter_index": 1,
  "version": 1,
  "content": "..."
}
```

## 导出

```http
GET /api/novels/{novel_id}/export/markdown
POST /api/novels/{novel_id}/export
```

`POST /export` 是 Web 工作台导出动作的兼容入口，当前默认导出 Markdown，响应与 `/export/markdown` 相同。

响应：

```json
{
  "novel_id": "...",
  "format": "markdown",
  "filename": "....md",
  "markdown": "# 标题\n\n## 第一章\n\n..."
}
```

## AgentRun

```http
GET /api/runs?limit=50
GET /api/runs?limit=50&novel_id=...&role=writer&task=generate_chapter&status=ok
GET /api/runs?limit=50&provider=openai&model=gpt-5&reasoning_effort=xhigh
GET /api/runs/{run_id}
GET /api/runs/stream?limit=50
GET /api/agent-runs?limit=50
GET /api/agent-runs/{run_id}
GET /api/agent-runs/stream?limit=50
GET /api/novels/{novel_id}/runs?limit=20
GET /api/novels/{novel_id}/runs?limit=20&role=writer&task=generate_chapter&status=ok
```

全局 `GET /api/runs` 可选传 `novel_id`；`GET /api/agent-runs` 是给 Web 工作台保留的兼容别名，查询参数和响应与 `/api/runs` 相同。`/stream` 入口使用同一组查询参数，返回当前筛选结果的 SSE 快照事件，适合 Web 工作台用 fetch-SSE 或重连方式刷新运行面板。作品内 `GET /api/novels/{novel_id}/runs` 固定查当前作品。`role` / `task` / `status` / `provider` / `model` / `reasoning_effort` 均为可选筛选参数。`status` 必须是 `ok`、`fallback`、`parse_error` 之一；非法值返回 `400 Bad Request`。`summary` 基于筛选后的 `runs` 计算。`provider` / `model` / `reasoning_effort` 是模型运行元数据；旧记录如果没有保存该元数据，会返回 `null`，也不会命中对应模型元数据筛选。`prompt_tokens` / `completion_tokens` / `total_tokens` 来自 provider usage 或 smoke provider 估算；provider 未返回 usage 时会返回 `null`。`prompt_cost_micro_usd` / `completion_cost_micro_usd` / `total_cost_micro_usd` 只在该 run 同时有 token usage 和完整 `_model.pricing` 时计算；否则返回 `null`。

响应：

```json
{
  "runs": [
    {
      "id": "...",
      "novel_id": "...",
      "role": "writer",
      "task": "generate_chapter",
      "provider": "openai",
      "model": "gpt-5",
      "reasoning_effort": null,
      "status": "ok",
      "attempt": 1,
      "duration_ms": 12,
      "prompt_tokens": 800,
      "completion_tokens": 400,
      "total_tokens": 1200,
      "prompt_cost_micro_usd": 800,
      "completion_cost_micro_usd": 800,
      "total_cost_micro_usd": 1600,
      "output_summary": "生成第 1 章《旧账重开》，2600 字。",
      "structured": {},
      "raw_text": "...",
      "raw_notes": "...",
      "parse_error": null,
      "created_at": "2026-06-09T00:00:00Z"
    }
  ],
  "summary": {
    "total": 1,
    "ok": 1,
    "fallback": 0,
    "parse_error": 0,
    "duration_ms_total": 12,
    "tokenized_runs": 1,
    "prompt_tokens": 800,
    "completion_tokens": 400,
    "total_tokens": 1200,
    "priced_runs": 1,
    "prompt_cost_micro_usd": 800,
    "completion_cost_micro_usd": 800,
    "total_cost_micro_usd": 1600
  }
}
```

详情响应：

```json
{
  "run": {}
}
```

流式快照：

```text
event: snapshot
data: {"runs":[],"summary":{}}

event: completed
data: {"total":0}
```
