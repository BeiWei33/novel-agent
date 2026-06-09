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
- 已提供 facts 和 continuity report 只读 API
- 已提供 AgentRun API
- 已提供新建小说 API
- 已提供写章节、审稿、重写 API
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
- 已支持 AgentRun token / duration 汇总统计
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
- 给每个 Agent 输出增加 UI 可展示摘要字段
- 优化 `ReviewReport` 的问题列表和修改建议
- 补充章节返工类型：整章重写、开头重写、结尾重写、语言润色
- 给 C 提供审稿面板字段说明
- 准备 3 个 UI demo 小说项目样例

优先级 P1：

- 优化 Continuity Agent 输出
- 优化 Style Agent 输出
- 设计伏笔表 UI 字段
- 设计事实表 UI 字段
- 增加人工评测表

优先级 P2：

- 做不同 provider 的生成效果对比
- 优化章尾钩子
- 优化人物行为一致性检查
- 沉淀平台题材模板

### 4.3 交付物

```text
prompts/*.md
docs/RUBRIC.md
docs/SCHEMAS.md
examples/*.md
docs/UI_CONTENT_GUIDE.md
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
