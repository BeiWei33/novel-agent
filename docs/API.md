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
  "status": "ok"
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
POST /api/novels/{novel_id}/outline
```

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
```

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
GET /api/novels/{novel_id}/runs?limit=20
GET /api/novels/{novel_id}/runs?limit=20&role=writer&task=generate_chapter&status=ok
```

`role` / `task` / `status` 均为可选筛选参数。`status` 必须是 `ok`、`fallback`、`parse_error` 之一；非法值返回 `400 Bad Request`。`summary` 基于筛选后的 `runs` 计算。

响应：

```json
{
  "runs": [
    {
      "id": "...",
      "novel_id": "...",
      "role": "writer",
      "task": "generate_chapter",
      "status": "ok",
      "attempt": 1,
      "duration_ms": 12,
      "total_tokens": 1200,
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
    "total_tokens": 1200
  }
}
```
