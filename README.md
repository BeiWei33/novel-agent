# novel-agent

`novel-agent` 是一个用 Rust 构建的多 Agent 中文网文创作编排系统。MVP 目标是跑通从创意输入、新书设定、人物和大纲、章节正文、连续性检查、润色、审稿、返工到 Markdown 导出的闭环。

当前项目同时支持离线 smoke provider 和真实模型 provider。离线 smoke provider 不访问网络，用于本地 demo、CI 和接口回归；真实模型 provider 目前支持 OpenAI-compatible 和 DeepSeek。

## 当前状态

- Rust CLI + SQLite 本地持久化。
- Agent 输出统一使用 `AgentOutputEnvelope`。
- 新书创建链路：Market -> Plot -> Character -> Worldbuilding。
- 章节写作链路：Writer -> Continuity -> Style。
- 审稿返工链路：Reviewer -> Writer(rewrite) -> Continuity -> Style -> Reviewer。
- 支持章节版本快照、facts、world settings、continuity reports、review reports、agent runs。
- 支持 `provider = "smoke" | "openai" | "deepseek"`；`openai` 可通过 `OPENAI_BASE_URL` 指向本地 OpenAI-compatible 代理。
- 支持 `reasoning_effort` 透传到 OpenAI-compatible provider，已验证 `gpt-5.5 + xhigh` 2 章快速无重试真实链路；6 章链路已用 `-CheckpointResumes 6` 从头跑通，且仍可在本地代理中断后复用检查点续跑。
- 支持 `write --stream` 和 `rewrite --stream` 将已生成正文分块输出到终端。
- 支持 `versions` 查看章节版本快照并做基础对比。
- 支持 `edit` 从本地文件保存人工编辑稿为新的章节版本。
- 支持通过 REST API 保存人工编辑稿为新的章节版本，便于 Web 编辑器真实模式落库。
- 支持 Plot Agent 分批生成大纲，`new --outline-batch-size` 和 `outline --batch-size` 可降低真实模型长 JSON 截断风险。
- 支持 `new --resume-novel-id` 从已落库的新书 AgentRun 续跑；`scripts/mvp_demo.ps1` 支持 `-WorkDir` / `-ResumeNovelId` 复用临时库恢复真实模型验收。
- 支持 `serve` 启动本地 REST API，给 Web 工作台提供作品、章节、版本、审稿、导出、SSE、后台任务和 AgentRun 接口。
- 支持通过 API 查询作品 facts 和章节最新连续性报告，便于工作台展示伏笔/事实表和连续性侧栏。
- 后台任务记录已写入 SQLite，包含创建 payload、retry 来源关系和进度字段；完成/失败任务可在服务重启后继续查询，遗留未完成任务会在下次 `serve` 启动时标为失败。
- 支持按 `status` / `kind` / `novel_id` / `source_job_id` 筛选后台任务列表，便于工作台任务面板展示运行中任务、批量任务、当前作品任务和重试链。
- 支持通过 API 基于失败任务的 payload 创建重试 job，新 job 会用 `source_job_id` 指向源任务。
- 支持通过 API 取消 `queued` / `running` 后台任务，取消终态不会被后台完成结果覆盖。
- 支持批量章节写作 job，可一次提交章节范围并顺序生成多章，任务进度会按已完成章节数推进。
- AgentRun summary 已包含状态、耗时和 token 汇总，并支持全局或按作品以 `role` / `task` / `status` 筛选，便于运行面板展示。
- 已提供 Web 工作台内容展示指南，覆盖审稿面板、返工类型、连续性侧栏、事实表、AgentRun 摘要和 3 个 UI demo 项目来源。
- 已提供人工评测表，用于对比 provider、prompt 版本和题材样例的生成质量。
- 已记录真实模型人工评测样例和 provider 对照摘要，覆盖 `gpt-5.5 + xhigh` 和 DeepSeek 都市重生第 1 章。
- 已提供 Prompt 版本记录，当前 bundle 为 `b-quality-2026-06-09-r3`。
- 已提供 Web demo 内容包，主展示项目为都市重生商业文《重回外卖站》。
- 已提供失败样例库，用于沉淀 provider error、parse error 和质量退化回归。
- 已将 DeepSeek 对照样本暴露出的商业谈判过顺、章尾压力不足问题反灌到 Market / Plot / Writer / Reviewer Prompt。
- 已沉淀平台题材模板，覆盖起点/番茄/通用策略、章尾钩子类型和人物行为一致性检查。

交付验收入口见 [docs/MVP_ACCEPTANCE.md](docs/MVP_ACCEPTANCE.md)，REST API 见 [docs/API.md](docs/API.md)，接口冻结和协作规则见 [docs/INTERFACE_FREEZE.md](docs/INTERFACE_FREEZE.md)，Web 内容展示口径见 [docs/UI_CONTENT_GUIDE.md](docs/UI_CONTENT_GUIDE.md)，Web demo 内容包见 [docs/WEB_DEMO_CONTENT.md](docs/WEB_DEMO_CONTENT.md)，人工评测口径见 [docs/HUMAN_EVAL.md](docs/HUMAN_EVAL.md)，评测记录见 [docs/EVAL_LOG.md](docs/EVAL_LOG.md)，失败样例库见 [docs/FAILURE_CASES.md](docs/FAILURE_CASES.md)，Prompt 版本见 [docs/PROMPT_CHANGELOG.md](docs/PROMPT_CHANGELOG.md)，平台题材模板见 [docs/PLATFORM_TEMPLATES.md](docs/PLATFORM_TEMPLATES.md)。

## 快速开始

### 1. 验证工程

```powershell
cargo check
cargo test
```

当前期望结果：

```text
cargo check 通过
cargo test: api/storage unit 3 passed; smoke tests 14 passed
```

### 2. 运行离线 MVP demo

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
```

默认使用 `provider = "smoke"`，不需要 API key，也不会访问网络。

可选：流式输出正文。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -StreamWrite
```

### 3. 真实模型验收

OpenAI：

```powershell
$env:OPENAI_API_KEY = "..."
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -UseRealModel
```

本地 OpenAI-compatible 代理（cliproxyapi / gpt-5.5 xhigh）：

```powershell
$env:OPENAI_BASE_URL = "http://127.0.0.1:8317/v1"
$env:OPENAI_API_KEY = "<cliproxy bearer token>"
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel
```

本地 OpenAI-compatible 快速无重试验收：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 2 -OutlineChapters 2 -NewOutlineBatchSize 1 -OutlineBatchSize 1 -SkipOutline -SkipRewrite -StepRetries 0
```

本地 OpenAI-compatible 6 章验收建议直接开启检查点续跑。首次运行失败时保留脚本输出的 `work_dir` 和 `resume_novel_id`，再用同一临时库恢复：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 6 -OutlineChapters 6 -NewOutlineBatchSize 3 -OutlineBatchSize 3 -SkipOutline -SkipRewrite -StepRetries 0 -CheckpointResumes 6
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 6 -OutlineChapters 6 -NewOutlineBatchSize 3 -OutlineBatchSize 3 -SkipOutline -SkipRewrite -StepRetries 0 -CheckpointResumes 6 -WorkDir <work_dir> -ResumeNovelId <novel_id>
```

DeepSeek：

```powershell
$env:DEEPSEEK_API_KEY = "..."
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel
```

DeepSeek 快速真实链路可先用短大纲降低长 JSON 截断风险：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel -NewChapters 6 -SkipOutline -SkipRewrite
```

`-UseRealModel` 会在缺少 key 时提前失败，并在 `runs --fail-on-bad-status` 检测到 fallback 或 parse error 时判失败。

### 4. 启动 REST API

```powershell
cargo run -- serve --bind 127.0.0.1:3001
```

API 文档见 [docs/API.md](docs/API.md)。默认 `smoke` provider 下，新建作品、章节生成、审稿和重写接口均可离线跑通。

可直接运行 API demo：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
```

该脚本会使用临时 SQLite 配置启动本地 API 服务，调用 CORS、作品创建、章节生成、审稿、SSE、后台任务、Markdown 导出和 AgentRun 查询，然后自动停止服务。

## 配置

配置文件默认为 `novel-agent.toml`，可通过 `--config` 指定。示例：

```toml
[model]
provider = "smoke"
model = "smoke"

[storage]
database_url = "sqlite://novel-agent.db"
```

真实模型示例：

```toml
[model]
provider = "deepseek"
model = "deepseek-chat"

[storage]
database_url = "sqlite://novel-agent.db"
```

`novel-agent.toml` 已被 `.gitignore` 忽略；提交仓库时请使用 `novel-agent.toml.example` 作为模板。

## CLI

创建新书：

```powershell
cargo run -- new "都市重生商业文，主角回到十年前，从外卖站开始逆袭" --platform fanqie --chapters 30 --outline-batch-size 5
cargo run -- new "都市重生商业文，主角回到十年前，从外卖站开始逆袭" --platform fanqie --chapters 30 --outline-batch-size 5 --resume-novel-id <id>
```

生成或更新大纲：

```powershell
cargo run -- outline --novel-id <id> --chapters 30 --batch-size 5
```

生成章节：

```powershell
cargo run -- write --novel-id <id> --chapter 1
cargo run -- write --novel-id <id> --chapter 1 --stream
```

审稿：

```powershell
cargo run -- review --novel-id <id> --chapter 1
```

返工：

```powershell
cargo run -- rewrite --novel-id <id> --chapter 1
cargo run -- rewrite --novel-id <id> --chapter 1 --stream
```

保存人工编辑稿：

```powershell
cargo run -- edit --novel-id <id> --chapter 1 --input edits\chapter-1.md --summary "人工编辑后补强目标和伏笔"
```

查看章节版本：

```powershell
cargo run -- versions --novel-id <id> --chapter 1
cargo run -- versions --novel-id <id> --chapter 1 --show 2
cargo run -- versions --novel-id <id> --chapter 1 --from 1 --to 2
```

导出 Markdown：

```powershell
cargo run -- export --novel-id <id> --format markdown --output exports\demo.md
```

查看 Agent 调用记录：

```powershell
cargo run -- runs --novel-id <id> --limit 20
cargo run -- runs --novel-id <id> --limit 80 --summary --fail-on-bad-status
```

## 目录结构

```text
docs/       项目文档、接口冻结、schema、rubric、审查记录
prompts/    各业务 Agent 的 Prompt 模板
examples/   三个题材样例和 expected_checks
src/        Rust CLI、domain、model、storage、workflow
tests/      smoke 和回归测试
scripts/    MVP demo 脚本
```

## A/B 协作边界

开发者 A 负责 Rust 工程线：

- CLI
- workflow
- model provider
- SQLite storage
- Agent 调用、重试、fallback 和运行记录
- smoke tests 和 demo 脚本

开发者 B 负责小说业务线：

- Prompt 模板
- `docs/SCHEMAS.md`
- `docs/RUBRIC.md`
- 平台策略
- 测试题材样例和 `expected_checks`
- 网文质量标准和返工策略

字段变更必须同步更新：

- `docs/INTERFACE_FREEZE.md`
- `docs/SCHEMAS.md`
- `prompts/*.md`
- Rust domain/storage/workflow
- `tests/smoke.rs`
- `examples/*.md`
- `docs/STAFF_REVIEW_ISSUES.md`

## 验收口径

离线工程验收：

```powershell
cargo check
cargo test
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1
```

真实模型验收：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -UseRealModel
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 2 -OutlineChapters 2 -NewOutlineBatchSize 1 -OutlineBatchSize 1 -SkipOutline -SkipRewrite -StepRetries 0
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider openai -Model gpt-5.5 -ReasoningEffort xhigh -UseRealModel -NewChapters 6 -OutlineChapters 6 -NewOutlineBatchSize 3 -OutlineBatchSize 3 -SkipOutline -SkipRewrite -StepRetries 0 -CheckpointResumes 6
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel
powershell -ExecutionPolicy Bypass -File .\scripts\mvp_demo.ps1 -Provider deepseek -UseRealModel -NewChapters 6 -SkipOutline -SkipRewrite
```

真实模型验收必须满足：

- 对应 API key 可见。
- CLI 不出现 fallback 或解析失败提示。
- demo 会用 `runs --limit 80 --summary --fail-on-bad-status` 覆盖本次链路主要 AgentRun；真实模式下脚本会检查 fallback 和 parse error 并判失败。
- `agent_runs` 中无本次真实链路 parse error。
- 完整验收应让 `new -> outline -> write -> review -> rewrite -> versions -> edit -> versions -> export -> runs` 通过。
- 完整验收默认会用 5 章一批生成/刷新大纲，必要时可调低 `-NewOutlineBatchSize` 和 `-OutlineBatchSize`。
- 快速真实链路可用 `-NewChapters 6 -SkipOutline -SkipRewrite` 先确认 provider 能稳定返回合法 AgentOutput。
- 本地代理发生子进程级中断时，可用 `-CheckpointResumes` 在同次脚本运行中从已落库检查点继续；章节写作续跑会复用同章 Writer / Continuity / Style 检查点，也可用同一 `-WorkDir` 和 `-ResumeNovelId` 手动恢复。
