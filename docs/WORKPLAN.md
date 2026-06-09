# novel-agent 三人开发分工计划

版本：v0.2
日期：2026-06-09
团队规模：3 人
目标：在现有 CLI MVP 基础上，继续完善 Agent 编排能力，并启动 B 端 Web 创作工作台。

## 1. 当前阶段判断

项目当前不再是空项目阶段，已经具备 CLI MVP 的基础工程链路：

- Rust CLI 骨架已存在。
- 核心 domain、workflow、agents、model、storage 模块已存在。
- Prompt、schema、rubric、examples 和 smoke demo 已存在。
- SQLite、本地 smoke provider、真实 provider 配置、章节版本和导出链路已经进入验收口径。

下一阶段的重点是：

```text
A：把后端能力产品化为稳定 API。
B：继续提升小说生成、审稿、返工质量。
C：把 CLI 能力做成 B 端创作工作台。
```

## 2. 分工原则

项目分成三条主线：

```text
开发者 A：Rust 核心工程与 API 线
开发者 B：小说 Agent 业务与质量线
开发者 C：B 端 Web 创作工作台线
```

三人的边界：

- A 负责系统能跑、能存、能调度、能通过 API 暴露。
- B 负责系统会写、会评、会改、像网文。
- C 负责系统好用、好看、可视化、适合长时间创作。

共同接口：

```text
Domain Model
AgentInput / AgentOutput
Workflow Result
REST API DTO
SSE Event
ReviewReport
ChapterVersion
```

## 3. 开发者 A：Rust 核心工程与 API 线

### 3.1 主要职责

开发者 A 负责后端工程、API 和系统稳定性。

职责范围：

- 维护 Rust 核心模块
- 维护 Agent trait 和 workflow
- 维护 SQLite 存储和 repository
- 维护 Rig / smoke / OpenAI / DeepSeek provider
- 增加 axum API
- 增加 REST DTO
- 增加 SSE 流式输出
- 支持 Web 工作台调用现有 CLI 能力
- 保证 `cargo check`、`cargo test`、demo 脚本稳定

### 3.2 第一阶段任务

优先级 P0：

- 已增加 `src/api.rs` 模块
- 已接入 `axum`
- 已提供作品列表 API
- 已提供小说详情 API
- 已提供章节列表 API
- 已提供章节详情 API
- 已提供章节版本 API
- 已提供审稿报告 API
- 已提供小说 Bible、人物卡和世界设定拆分只读 API
- 已提供 facts 和 continuity report 只读 API
- 已提供 AgentRun API
- 已提供新建小说 API
- 已提供写章节、人工保存、审稿、重写 API
- 已给 C 提供 OpenAPI 风格接口说明：`docs/API.md`

优先级 P1：

- 已提供章节生成 SSE
- 已提供重写 SSE
- 已提供导出 API
- 增加统一错误响应
- 已增加 CORS 配置
- 已增加 API smoke test
- 已增加 API demo 配置

优先级 P2：

- 已支持 SQLite 持久化后台任务记录、payload、retry 来源关系、进度字段、status/kind/novel_id/source_job_id 列表筛选、遗留任务收口、失败任务重试、queued/running 任务取消和批量章节写作 job
- 已支持 AgentRun token / duration 汇总统计、全局查询、详情查询、SSE 快照、兼容别名和 role/task/status 筛选
- 支持 AgentRun cost 统计
- 支持 provider 在线切换
- 准备 workspace 拆分

### 3.3 交付物

```text
src/api/mod.rs
src/api/routes.rs
src/api/dto.rs
src/api/error.rs
src/api/stream.rs
docs/API.md
```

验收方式：

```powershell
cargo check
cargo test
novel-agent serve --config novel-agent.toml
```

## 4. 开发者 B：小说 Agent 业务与质量线

### 4.1 主要职责

开发者 B 负责小说创作质量、Prompt 和业务策略。

职责范围：

- 维护各 Agent Prompt
- 维护小说圣经 schema
- 维护人物卡、世界观、章节大纲 schema
- 维护审稿 Rubric
- 优化起点 / 番茄平台化策略
- 设计返工策略
- 维护测试题材样例
- 和 C 一起决定审稿结果在 UI 中如何呈现

### 4.2 第一阶段任务

优先级 P0：

- 稳定 Market / Plot / Character / Writer / Reviewer 输出结构
- 已给每个 Agent 输出梳理 UI 可展示摘要字段，见 `docs/UI_CONTENT_GUIDE.md`
- 已优化 `ReviewReport` 的问题列表和修改建议输出规则
- 已补充章节返工类型：整章重写、开头重写、结尾重写、语言润色
- 已给 C 提供审稿面板字段说明，见 `docs/UI_CONTENT_GUIDE.md`
- 已准备 3 个 UI demo 小说项目样例来源：`examples/urban_rebirth.md`、`examples/fantasy_upgrade.md`、`examples/romance_comeback.md`

优先级 P1：

- 已优化 Continuity Agent 输出约束
- 已优化 Style Agent 输出约束
- 已设计伏笔表 UI 字段，见 `docs/UI_CONTENT_GUIDE.md`
- 已设计事实表 UI 字段，见 `docs/UI_CONTENT_GUIDE.md`
- 已增加人工评测表，见 `docs/HUMAN_EVAL.md`

优先级 P2：

- 已准备不同 provider 的生成效果对比表和判读规则，见 `docs/HUMAN_EVAL.md`；`gpt-5.5 xhigh` 与 DeepSeek 对照摘要见 `docs/EVAL_LOG.md`
- 已提供 Prompt 版本记录，见 `docs/PROMPT_CHANGELOG.md`；当前 bundle 为 `b-quality-2026-06-10-v0.3-guard`
- 已将评测日志关键元数据纳入 smoke 测试，新增记录必须包含 `prompt_bundle`、AgentRun summary 和人工总分
- 已提供 Web demo 内容包，见 `docs/WEB_DEMO_CONTENT.md`；主 demo 为 `urban_rebirth_fanqie_demo`
- 已提供失败样例库，见 `docs/FAILURE_CASES.md`
- 已优化章尾钩子规则，见 `docs/PLATFORM_TEMPLATES.md`
- 已优化人物行为一致性检查，见 `docs/PLATFORM_TEMPLATES.md`
- 已沉淀平台题材模板，见 `docs/PLATFORM_TEMPLATES.md`
- 已根据 DeepSeek 对照评测补强商业谈判阻力、未来信息场景化和章尾硬压力规则
- 已把都市重生样例的外部阻力、失败代价和下一步压力写入 `expected_checks`，并纳入 smoke fixture 回归

### 4.3 交付物

```text
prompts/*.md
docs/RUBRIC.md
docs/SCHEMAS.md
examples/*.md
docs/UI_CONTENT_GUIDE.md
docs/HUMAN_EVAL.md
docs/EVAL_LOG.md
docs/FAILURE_CASES.md
docs/PROMPT_CHANGELOG.md
docs/WEB_DEMO_CONTENT.md
docs/PLATFORM_TEMPLATES.md
```

## 5. 开发者 C：B 端 Web 创作工作台线

### 5.1 主要职责

开发者 C 负责把 `novel-agent` 从 CLI 工具变成可视化创作工作台。

职责范围：

- 初始化 `apps/web`
- 搭建 React + TypeScript + Vite 工程
- 接入 Tailwind CSS、shadcn/ui、lucide-react
- 接入 TanStack Query、Zustand、React Router
- 接入 CodeMirror 6
- 实现 B 端信息架构
- 实现作品列表
- 实现小说工作台
- 实现章节编辑器
- 实现审稿面板
- 实现 Agent 运行面板
- 实现版本列表和版本查看
- 实现生成、审稿、重写、保存人工编辑稿、导出等主流程

### 5.2 第一阶段任务

优先级 P0：

- 创建 `apps/web`
- 搭建全局布局：左侧导航、中间主内容、右侧 Agent 面板
- 实现 API client
- 实现作品列表页
- 实现新建小说表单
- 实现小说详情页
- 实现章节树
- 实现章节编辑器
- 实现生成章节按钮
- 实现审稿按钮
- 实现重写按钮
- 实现 ReviewReport 面板
- 实现章节版本列表
- 实现人工编辑保存

优先级 P1：

- 接入 SSE 流式输出
- 实现生成中状态
- 实现 AgentRun 时间线
- 实现错误提示和重试
- 实现版本正文查看
- 实现 v1 / v2 文本对比
- 实现 Markdown 导出入口
- 做桌面和笔记本宽度适配

优先级 P2：

- 增加快捷键
- 增加批量生成入口
- 增加伏笔表和事实表可视化
- 增加 Tauri 桌面版预研
- 增加主题和密度设置

### 5.3 交付物

```text
apps/web/package.json
apps/web/src/main.tsx
apps/web/src/App.tsx
apps/web/src/routes/*
apps/web/src/components/*
apps/web/src/features/novels/*
apps/web/src/features/chapters/*
apps/web/src/features/agent-runs/*
apps/web/src/lib/api.ts
apps/web/src/lib/store.ts
```

第一版页面：

```text
/novels
/novels/new
/novels/:novelId
/novels/:novelId/chapters/:chapterIndex
/agent-runs
```

## 6. 三方协作边界

### A 和 B

共同决定：

- Agent 输入输出结构
- schema 字段
- workflow 中间产物
- 返工策略
- provider 错误处理

### A 和 C

共同决定：

- REST API 路径
- DTO 字段
- SSE event 格式
- 错误响应格式
- 本地开发启动方式

### B 和 C

共同决定：

- 审稿报告展示方式
- Agent 建议文案展示
- 小说圣经、人物卡、世界观字段布局
- 起点 / 番茄平台策略在 UI 中如何切换

### A、B、C 共同决定

- 是否冻结接口
- 是否拆 workspace
- Web demo 验收链路
- 第一批真实用户测试样例

## 7. 推荐开发节奏

### 第 1-2 天：API 和 Web 骨架

A：

- 增加 axum serve 命令
- 提供作品、章节、版本、审稿报告只读 API
- 写 `docs/API.md` 初稿

B：

- 给 ReviewReport 增加 UI 展示说明
- 准备 3 个 demo 项目样例
- 标注审稿面板的关键字段

C：

- 初始化 `apps/web`
- 搭建布局、路由、API client
- 做作品列表和章节编辑器静态页面

阶段验收：

- 后端能启动 API 服务
- Web 能打开
- Web 能展示 smoke 数据或 mock 数据

### 第 3-4 天：打通主链路

A：

- 提供新建小说、写章节、审稿、重写 API
- 提供统一错误响应
- 支持 CORS

B：

- 调整 Agent 输出摘要字段
- 优化审稿问题和返工指令

C：

- 接入真实 API
- 在页面完成新建小说、生成章节、审稿、重写
- 显示加载态和错误态

阶段验收：

```text
Web 页面能完成：
新建小说 → 生成第 1 章 → 审稿 → 重写
```

### 第 5-7 天：版本、流式输出和体验打磨

A：

- 提供 SSE 流式输出
- 提供导出 API
- 补 API smoke test

B：

- 检查 UI demo 的文本质量
- 调整章节通过线和返工建议

C：

- 接入 SSE
- 实现 AgentRun 时间线
- 实现版本列表和正文查看
- 实现人工编辑保存
- 实现 Markdown 导出入口

阶段验收：

```text
Web 页面能完成：
打开工作台 → 查看小说 → 编辑章节 → 生成 → 审稿 → 重写 → 查看版本 → 保存人工编辑 → 导出
```

## 8. UI MVP 验收标准

UI MVP 完成时，系统必须支持：

- 打开 Web 工作台
- 查看小说列表
- 创建新小说
- 查看小说圣经
- 查看章节列表
- 打开章节正文
- 生成章节
- 审稿
- 重写
- 查看审稿报告
- 查看章节版本
- 保存人工编辑稿
- 导出 Markdown
- 查看最近 AgentRun

质量要求：

- 长文本编辑区域足够大
- 生成中有明确状态
- 错误信息可读
- 审稿建议可操作
- 版本之间能区分
- 页面布局不拥挤、不遮挡正文
- 用户不需要理解 CLI 命令

## 9. 风险和边界

主要风险：

- A 的 API 没稳定，C 容易反复改 client。
- B 的 schema 变动过快，A 和 C 会同步成本高。
- Web UI 过早追求复杂功能，会拖慢主链路。
- SSE 和长文本编辑同时接入时，状态管理容易混乱。
- 真实模型延迟较高，UI 必须处理等待和失败。

应对策略：

- 先冻结 `docs/API.md` 的 P0 接口。
- C 先用 mock 数据并行开发。
- 所有 P0 页面只服务主链路。
- ReviewReport、ChapterVersion、AgentRun 字段变动需要同步三方。
- 真实模型失败时保留 smoke provider demo 路径。

## 10. 每日同步模板

每天同步只回答 5 个问题：

```text
昨天完成了什么？
今天准备完成什么？
接口或 schema 有没有变化？
有没有阻塞别人？
今天的 demo 链路是否还能跑通？
```

A 重点同步：

- API 路径是否变了
- DTO 字段是否变了
- SSE event 是否变了
- 存储字段是否变了

B 重点同步：

- Prompt 输出格式是否变了
- 审稿标准是否变了
- 测试样例是否变了
- UI 展示字段是否变了

C 重点同步：

- 页面路由是否变了
- 需要新增哪些 API
- 哪些字段无法支撑 UI
- 哪些交互影响 workflow

## 11. 推荐任务归属汇总

| 模块 | 负责人 | 备注 |
| --- | --- | --- |
| Rust 核心 workflow | A | P0 |
| Agent trait | A 主导，B 参与 | P0 |
| Rig / provider 接入 | A | P0 |
| SQLite 存储 | A | P0 |
| CLI | A | P0 |
| axum API | A | P0 |
| REST DTO | A 主导，C 参与 | P0 |
| SSE 流式输出 | A 主导，C 接入 | P1 |
| Prompt 模板 | B | P0 |
| NovelBible schema | B 主导，A 参与 | P0 |
| ReviewReport schema | B 主导，A/C 参与 | P0 |
| 章节质量 Rubric | B | P0 |
| 测试题材样例 | B | P0 |
| Web 工程初始化 | C | P0 |
| B 端布局和设计系统 | C | P0 |
| 作品列表和新建作品 | C | P0 |
| 章节编辑器 | C | P0 |
| 审稿面板 | C 主导，B 参与 | P0 |
| Agent 运行面板 | C 主导，A 参与 | P1 |
| 版本列表和版本查看 | C 主导，A 参与 | P1 |
| Markdown 导出入口 | C 接入，A 提供 API | P1 |
| UI demo 验收 | C 主导，A/B 参与 | P1 |

## 12. 结论

三人分工建议：

```text
A：后端和 API，把能力稳定暴露出来。
B：小说业务和质量，让输出像能连载的网文。
C：B 端工作台，让作者愿意每天用它写作。
```

下一阶段的里程碑是：

```text
CLI MVP
→ axum API
→ React Web 工作台
→ 页面完成新建、生成、审稿、重写、版本、导出闭环
```

## 13. v0.3 下一版本合并执行计划

来源：开发者 B / C 的 v0.3 建议合并稿。
建议版本：v0.3 Web 工作台真实闭环版。
核心判断：v0.3 不继续优先堆 Agent、真实模型压测或外延功能，而是把现有 CLI / API / Prompt / Web demo 能力压成一条稳定可演示、可复测、可真实使用的浏览器工作台闭环。

一句话目标：

```text
作者能在浏览器里完成：
创建作品 -> 生成章节 -> 审稿返工 -> 人工编辑 -> 查看版本 -> 导出 Markdown -> 复盘 AgentRun / jobs / 质量证据
```

### 13.1 开发前必须先收口

以下事项先于 v0.3 功能开发处理：

- 明确 `novel-agent-c/` 的归属。它当前是 C 线 worktree，不应作为普通子目录提交进主仓库；需要把 C 线最新提交合并回主工作区，或将 worktree 移出项目目录。
- 已将 B / C 的 v0.3 建议合并到本节；后续只以 `docs/WORKPLAN.md` 第 13 节作为权威 v0.3 开发计划，独立 `docs/NEXT_VERSION_PLAN.md` 不再保留。
- 修正 Web 新建作品默认章节数。当前前端按 `target_words / chapter_words` 计算，默认会把 `1_200_000 / 2600` 转成约 462 章并发给 API；v0.3 应新增明确的“规划章节数”字段，默认 30 或 demo 用 6，总字数只作为展示元数据。
- 本地测试命令固定独立 target，例如 `$env:CARGO_TARGET_DIR = "target\codex-test"` 后再执行 `cargo test`，避免运行中的 `target/debug/novel-agent.exe` 阻塞测试覆盖。
- v0.3 的“真实 API 模式优先”默认解释为 `smoke provider + real REST API` 优先；OpenAI / DeepSeek 真实 provider 作为带 key 的专项验收，不要求每次本地或 CI 都调用外部模型。

### 13.2 P0 主线

#### P0-1：冻结 v0.3 API / DTO

目标：让 Web 真实 API 模式稳定接入，不被字段和事件格式反复打断。

冻结对象：

- `Novel`
- `NovelBible`
- `CharacterCard`
- `WorldSetting`
- `Chapter`
- `ChapterVersion`
- `ReviewReport`
- `ContinuityReport`
- `Fact`
- `AgentRun`
- `ApiJob`

规则：

- 以 `docs/API.md` 和 `docs/INTERFACE_FREEZE.md` 为 v0.3 权威接口文档。
- 新字段只能后向兼容新增。
- 删除字段、改字段类型、改枚举语义必须进入下一版本。
- `ReviewReport`、`ChapterVersion`、`AgentRun`、`ApiJob` 字段变动必须 A/B/C 同步确认。
- v0.3 内除非阻塞 Web 主链路，不再扩张 API 面。

验收：

```powershell
cargo test
powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
```

#### P0-2：Web 真实 API 主闭环

目标：用户不需要理解 CLI 命令，也能完成主要创作流程。

必须支持：

- 打开作品列表。
- 创建新作品。
- 进入小说工作台。
- 打开章节编辑器。
- 生成第 1 章。
- 审稿并查看 ReviewReport。
- 按建议重写。
- 保存人工编辑稿。
- 查看版本列表和版本正文。
- 导出 Markdown。
- 查看本次 AgentRun。

体验要求：

- 第一屏就是工作台，不做营销首页。
- 长文本编辑区足够大。
- 生成、审稿、重写必须有清楚的进行中状态。
- API 失败时显示作者能理解的错误和重试入口。
- mock demo 保留为离线展示和 UI 回归兜底，但 v0.3 主验收切到 `VITE_USE_MOCK=false`。

验收：

```powershell
npm.cmd --prefix apps/web run typecheck
npm.cmd --prefix apps/web run build
```

并在浏览器真实 API 模式完成：

```text
新建 -> 生成 -> 审稿 -> 重写 -> 人工保存 -> 版本查看 -> 导出 -> AgentRun 查看
```

#### P0-3：后台任务体验补齐

目标：把已有 jobs API 做成 Web 工作台可用能力。

必须支持：

- 任务列表按状态、类型、作品、retry 来源筛选。
- 任务详情展示 `payload`、`result`、`error`、进度和 `source_job_id`。
- failed 任务支持重试。
- queued / running 任务支持取消。
- 批量生成章节时显示章节范围和完成进度。
- 从小说工作台跳转到当前作品任务队列。

验收：

```text
Web 能创建后台写作任务
jobs 页面能看到进度
失败任务能重试
排队或运行中任务能取消
按 novel_id / status / kind / source_job_id 筛选有效
```

#### P0-4：轻量质量视图

目标：让系统不只是“能生成”，还要能让作者看懂为什么过线、为什么返工、下一步改什么。

第一版展示：

- 当前章节 ReviewReport 总分和分项分数。
- 是否通过当前平台通过线。
- `rewrite_instruction` 的类型、优先级和目标。
- Continuity / facts / 伏笔更新的关键摘要。
- provider、model、reasoning_effort、prompt_bundle。
- fallback / parse_error / 质量退化样例的明显提示。

B 线要求：

- v0.3 接入期暂不大改 Prompt bundle，避免主链路不稳定。
- v0.3 推荐 prompt_bundle 为 `b-quality-2026-06-10-v0.3-guard`，只做一致性和质量视图守门补强，不改变 `AgentOutputEnvelope`。
- demo 内容优先使用 `docs/WEB_DEMO_CONTENT.md` 的 `urban_rebirth_fanqie_demo`。
- 真实模型展示候选使用 `gpt-5.5 xhigh` r3 6 章链路样本，但展示前必须修正旧名残留、权限说明和少量行业解释。
- 补强一致性约束：Bible / outline / draft 的主角名、金额、合作状态必须一致。

### 13.3 一键端到端验收脚本

目标：减少 A/B/C 各自手工验证，形成一条可复测的 v0.3 demo 链路。

建议新增脚本能力：

```text
启动临时 SQLite API
-> 启动 Web 或静态预览
-> 创建 demo 小说
-> 生成章节
-> 审稿
-> 重写
-> 保存人工编辑稿
-> 导出 Markdown
-> 查询 AgentRun 和 jobs
-> 输出通过 / 失败摘要
```

验收输出至少包含：

- `novel_id`
- 生成章节数
- 最新 ReviewReport 总分
- 导出文件大小
- AgentRun 总数、失败数、总耗时、总 token、总成本
- jobs 成功 / 失败 / 取消数量
- 失败时的 API 路径和错误消息

脚本要求：

- 使用临时 SQLite 配置，不依赖默认 `novel-agent.db`。
- 使用独立 `CARGO_TARGET_DIR`。
- 默认使用 `provider = "smoke"` 跑真实 REST API。
- 可选参数再启用 OpenAI / DeepSeek 真实 provider。

### 13.4 三方任务分工

A 侧重点：真实 API 稳定性和验收脚本。

- 合并 C 线代码后，冻结 v0.3 API / DTO。
- 修 Web 真实 API 模式暴露的接口缺口。
- 保持 `api_demo.ps1`、API smoke test、jobs、AgentRun、SSE、导出稳定。
- 提供推荐本地启动命令、独立测试 target 说明和一键端到端验收脚本。

B 侧重点：质量标准和展示内容守门。

- 明确 v0.3 推荐 prompt_bundle。
- 守住 Web demo 内容质量。
- 补齐 expected checks 与失败样例分类。
- 给质量视图提供展示口径。
- 输出 1-2 条真实模型可复测样本，作为 v0.3 demo 基线。

C 侧重点：真实模式 Web 闭环和任务体验。

- 按冻结接口跑通 Web 真实 API 模式。
- 修正新建作品章节数输入和默认值。
- 完善 jobs 页面、任务详情、重试、取消和当前作品任务跳转。
- 增加轻量质量视图。
- 优化真实模型等待、失败、重试、取消时的 UI 状态。
- 保留 mock demo 作为离线展示路径。

### 13.5 v0.3 不做清单

本版本明确不做：

- Tauri 桌面版。
- 多用户、权限、团队协作。
- PostgreSQL / pgvector / Qdrant 生产化切换。
- 复杂 RAG 和向量检索。
- 自动发布到真实平台。
- 收费系统。
- 大规模 UI 重设计。
- 大规模真实模型压测。
- 新增大量 Agent。
- 营销型官网首页。

这些能力后续重要，但 v0.3 的核心价值是把现有能力变成可用产品，而不是扩大范围。

### 13.6 v0.3 完成定义

工程验收：

```powershell
$env:CARGO_TARGET_DIR = "target\codex-test"
cargo check
cargo test
powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
npm.cmd --prefix apps/web run typecheck
npm.cmd --prefix apps/web run build
```

产品验收：

```text
浏览器真实 API 模式完成：
打开作品列表
创建新作品
进入小说工作台
生成第 1 章
查看生成中状态
审稿
查看 ReviewReport
执行重写
保存人工编辑稿
查看版本列表和版本正文
导出 Markdown
查看本次 AgentRun
查看相关 jobs
```

质量验收：

```text
Web demo 至少有一部可展示作品
第 1 章内容不是占位文本、乱码或空文本
审稿建议能转成作者下一步动作
质量视图能解释是否过线和为什么返工
fallback / parse_error 在 UI 中显眼
错误提示对用户可读
一次模型失败不会让页面进入不可恢复状态
```

v0.3 完成时，项目应从“CLI MVP + API + 文档齐全”推进到“可演示、可试用、可复测的 Web 创作工作台”。

### 13.7 v0.3-rc1 A 最后一轮验收安排

负责人：开发者 A

目标：在 `v0.3-rc1` 提交后，确认后端 API、SQLite 迁移、AgentRun/jobs、导出和端到端验收脚本仍然稳定，给 B/C 最终 Web 和质量验收提供可信底座。

验收命令：

```powershell
$env:CARGO_TARGET_DIR = "target\codex-rc1"
cargo check
cargo test
powershell -ExecutionPolicy Bypass -File .\scripts\api_demo.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\v03_e2e_demo.ps1
```

通过标准：

- `cargo check` 和 `cargo test` 全部通过。
- `api_demo.ps1` 覆盖 health、CORS、作品、章节、审稿、SSE、jobs、导出、AgentRun 和成本统计，退出码为 0。
- `v03_e2e_demo.ps1` 默认 smoke provider 下完成真实 REST API 闭环，输出 `status=ok`。
- AgentRun 汇总中 `fallback=0`、`parse_error=0`。
- jobs 汇总中失败数为 0，批量写作 job 至少有 1 个 succeeded。

验收产物：

- 记录 `v03_e2e_demo.ps1` 输出的 `novel_id`、`work_dir`、`review_score`、`agent_run_total`、`agent_run_total_cost_micro_usd` 和 jobs 汇总。
- 若任一命令失败，开发者 A 只修阻塞 Web 主闭环的 API / storage / workflow 问题；Prompt 质量和 UI 表现问题分别回给 B/C。
