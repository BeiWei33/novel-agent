# 开发者 C 交接文档

日期：2026-06-09
角色：B 端 Web 创作工作台负责人
目标：把现有 CLI MVP 做成可视化创作工作台。

## 1. C 的核心任务

开发者 C 不负责模型调用、不负责 Prompt、不负责 Rust workflow 内部逻辑。C 的核心任务是：

```text
把 A 暴露的后端能力和 B 定义的小说业务内容，组织成一个好用的 B 端创作界面。
```

第一版工作台需要服务真实写作流程，而不是展示型页面。

## 2. 技术栈

```text
React
TypeScript
Vite
Tailwind CSS
shadcn/ui
lucide-react
TanStack Query
Zustand
React Router
CodeMirror 6
TanStack Table
```

暂不使用 Next.js，暂不做 SSR。

## 3. 首批页面

需要先做 5 个路由：

```text
/novels
/novels/new
/novels/:novelId
/novels/:novelId/chapters/:chapterIndex
/agent-runs
```

### 3.1 作品列表页

目标：让用户看到已有作品并进入工作台。

字段：

- 书名
- 题材
- 目标平台
- 状态
- 章节数
- 更新时间
- 最近评分

操作：

- 新建作品
- 打开作品
- 导出

### 3.2 新建作品页

目标：把 CLI 的 `new "<创意>"` 变成表单。

字段：

- 创意描述
- 题材
- 目标平台：起点 / 番茄 / 通用
- 目标字数
- 章节字数

操作：

- 创建小说
- 显示生成中状态
- 创建成功后进入小说工作台

### 3.3 小说工作台

目标：集中查看小说核心资料。

标签页：

- 小说圣经
- 人物卡
- 世界观
- 大纲
- 事实表
- 伏笔表

### 3.4 章节编辑器

目标：日常写作主页面。

布局：

```text
左侧：章节树
中间：CodeMirror 正文编辑器
右侧：大纲 / 审稿 / Agent 面板
底部：版本记录
```

操作：

- 生成章节
- 审稿
- 重写
- 保存人工编辑稿
- 查看版本
- 导出

### 3.5 AgentRun 页面

目标：查看最近 Agent 执行情况。

字段：

- 运行时间
- Agent 角色
- provider
- 状态
- 耗时
- 错误信息
- 输出摘要

## 4. 第一版 API 依赖

C 可以先按以下接口做 mock，等 A 实现后替换真实 client。

作品：

```http
GET  /api/novels
POST /api/novels
GET  /api/novels/:novel_id
GET  /api/novels/:novel_id/bible
```

章节：

```http
GET  /api/novels/:novel_id/chapters
GET  /api/novels/:novel_id/chapters/:chapter_index
POST /api/novels/:novel_id/chapters/:chapter_index/write
POST /api/novels/:novel_id/chapters/:chapter_index/review
POST /api/novels/:novel_id/chapters/:chapter_index/rewrite
PUT  /api/novels/:novel_id/chapters/:chapter_index/content
GET  /api/novels/:novel_id/chapters/:chapter_index/versions
```

运行记录：

```http
GET /api/agent-runs
GET /api/agent-runs/:run_id
```

## 5. Mock 数据要求

在 A 的 API 未完成前，C 需要用 mock 数据并行开发。

至少准备：

- 3 本小说
- 每本 30 章目录
- 1 个小说圣经
- 3 张人物卡
- 1 章正文
- 2 个章节版本
- 1 份 ReviewReport
- 5 条 AgentRun

Mock 数据字段要贴近 `docs/SCHEMAS.md`，不要随意发明一套前端专用结构。

## 6. UI 设计原则

这个 UI 是 B 端生产工具，不是营销页。

设计原则：

- 信息密度要高，但不要拥挤。
- 正文编辑器必须是页面中心。
- 操作按钮要明确：生成、审稿、重写、保存、导出。
- Agent 输出要可读，不要堆大段原始 JSON。
- 审稿报告必须能直接指导重写。
- 失败状态必须明确显示。
- 长时间生成时必须有进度反馈。

不要做：

- 首页大 hero
- 装饰性插画
- 花哨背景
- 复杂动画
- 需要用户阅读说明才能使用的交互

## 7. 第一周验收链路

第一周 C 的验收目标：

```text
打开 Web
→ 查看作品列表
→ 新建作品
→ 进入小说工作台
→ 打开第 1 章
→ 生成正文
→ 审稿
→ 重写
→ 查看版本
→ 保存人工编辑稿
→ 导出 Markdown
```

如果 A 的真实 API 未完全就绪，C 可以先用 mock 完成页面和交互，再切 API。

## 8. 与 A 的对接清单

C 需要向 A 确认：

- API 基础地址
- 统一错误响应格式
- 日期格式
- 章节 ID 和章节序号的使用规则
- 版本 ID 的使用规则
- `write` / `review` / `rewrite` 是否同步返回
- SSE event 格式
- 导出 API 返回文件路径还是文件内容

## 9. 与 B 的对接清单

C 需要向 B 确认：

- ReviewReport 每个评分的展示名称
- 问题列表是否有严重等级
- 修改建议是否可拆成按钮动作
- 小说圣经哪些字段放首屏
- 人物卡哪些字段最重要
- 起点 / 番茄策略如何展示
- 返工类型有哪些

## 10. 完成标准

C 第一版完成后，用户应该能不打开命令行，也完成一次完整创作闭环。

最低完成标准：

- 页面能跑
- 主链路能通
- 长文本能编辑
- 审稿能看懂
- 重写能触发
- 版本能查看
- 错误能定位
- 布局适合长时间使用
