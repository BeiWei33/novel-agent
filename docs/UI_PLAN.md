# novel-agent B 端创作工作台方案

版本：v0.1
日期：2026-06-09
定位：面向作者、编辑和内容工作室的小说生产工作台

## 1. 产品判断

`novel-agent` 的 UI 第一阶段做 B 端，不做 C 端。

原因：

- 当前核心能力是多 Agent 编排、审稿、重写、记忆和版本管理，天然属于生产工具。
- 长篇小说创作需要频繁查看、编辑、对比、返工，不适合只用 CLI。
- C 端会过早引入登录、会员、支付、增长和移动端适配，容易稀释 Agent 编排主线。
- B 端更适合先验证真实创作效率，再决定是否包装成面向个人作者的轻量 C 端产品。

第一阶段 UI 目标不是做营销网站，而是做一个能长期写作、审稿和管理作品的创作后台。

## 2. 技术栈

推荐技术栈：

```text
后端 API：Rust + axum + tokio + sqlx + Rig
前端应用：React + TypeScript + Vite
UI 样式：Tailwind CSS + shadcn/ui + lucide-react
状态管理：TanStack Query + Zustand
路由：React Router
编辑器：CodeMirror 6
表格：TanStack Table
通信：REST + SSE
本地数据库：SQLite
生产数据库：PostgreSQL + pgvector
桌面版：Tauri
```

短期不引入 Next.js。这个项目第一阶段不需要 SSR，Vite 更轻，和 Tauri 后续集成也更直接。

## 3. 推荐目录结构

当前项目已经有单 crate CLI MVP。短期建议先在现有结构上增量增加 UI/API，不急着拆大 workspace。

短期结构：

```text
novel-agent
├── src
│   ├── api                  axum API 模块
│   ├── agents               Agent 抽象与执行
│   ├── domain               领域模型
│   ├── model                模型 provider
│   ├── storage              SQLite 存储
│   ├── workflow             工作流
│   ├── main.rs              CLI 入口
│   └── lib.rs
├── apps
│   └── web                  React B 端创作工作台
├── docs
├── prompts
├── examples
└── scripts
```

中期结构：

```text
novel-agent
├── crates
│   ├── novel-agent-core
│   ├── novel-agent-api
│   ├── novel-agent-cli
│   └── novel-agent-storage
├── apps
│   ├── web
│   └── desktop
├── prompts
└── docs
```

拆 workspace 的时机：

- API 路由超过 20 个
- Web 已经稳定使用核心领域能力
- CLI 和 API 共享代码开始显得拥挤
- 准备引入 Tauri 桌面版

## 4. 第一版页面

### 4.1 作品列表

用途：管理所有小说项目。

核心信息：

- 书名
- 题材
- 目标平台
- 状态
- 章节数
- 最新更新时间
- 最近评分

核心操作：

- 新建作品
- 打开作品
- 导出 Markdown
- 查看最近 Agent 运行记录

### 4.2 小说工作台

用途：集中查看小说圣经、人物、世界观、事实和伏笔。

布局建议：

```text
左侧：作品内导航
中间：当前资料面板
右侧：Agent 建议和运行记录
```

第一版标签页：

- 小说圣经
- 人物卡
- 世界观
- 事实表
- 伏笔表
- 大纲

### 4.3 章节编辑器

用途：日常写作的主工作区。

布局建议：

```text
左侧：卷 / 章节树
中间：正文编辑器
右侧：大纲 / 审稿 / Agent 面板
底部：版本记录 / 运行日志
```

核心操作：

- 生成章节
- 审稿
- 润色
- 重写
- 保存人工编辑版本
- 查看版本对比
- 导出当前章节

### 4.4 Agent 运行面板

用途：让用户知道系统正在做什么，避免黑盒感。

显示内容：

- 当前 Agent
- 运行状态
- 输入摘要
- 输出摘要
- 耗时
- provider
- 错误信息

第一版不展示完整 prompt，避免页面过载；提供“查看原始响应”入口即可。

### 4.5 审稿与返工面板

用途：把 Reviewer Agent 的输出变成可操作建议。

显示内容：

- 总分
- 开头吸引力
- 情节推进
- 爽点强度
- 人物表现
- 对话自然度
- 设定一致性
- 章尾钩子
- 平台适配度
- 问题列表
- 修改建议

核心操作：

- 按建议重写
- 只重写开头
- 只重写结尾
- 只润色语言
- 标记为通过

## 5. 第一版 API

第一版 API 以 CLI 已有能力为边界，不额外发散。

作品：

```http
GET    /api/novels
POST   /api/novels
GET    /api/novels/:novel_id
GET    /api/novels/:novel_id/bible
GET    /api/novels/:novel_id/characters
GET    /api/novels/:novel_id/world-settings
GET    /api/novels/:novel_id/facts
```

章节：

```http
GET    /api/novels/:novel_id/chapters
GET    /api/novels/:novel_id/chapters/:chapter_index
POST   /api/novels/:novel_id/chapters/:chapter_index/write
POST   /api/novels/:novel_id/chapters/:chapter_index/review
POST   /api/novels/:novel_id/chapters/:chapter_index/rewrite
PUT    /api/novels/:novel_id/chapters/:chapter_index/content
GET    /api/novels/:novel_id/chapters/:chapter_index/versions
GET    /api/novels/:novel_id/chapters/:chapter_index/versions/:version_id
```

Agent 运行：

```http
GET    /api/agent-runs
GET    /api/agent-runs/:run_id
GET    /api/agent-runs/stream
```

导出：

```http
POST   /api/novels/:novel_id/export
```

流式输出：

```http
GET    /api/novels/:novel_id/chapters/:chapter_index/write/stream
GET    /api/novels/:novel_id/chapters/:chapter_index/rewrite/stream
```

## 6. 第一版不做

第一版 UI 不做：

- 多用户权限
- 在线协作
- 支付和套餐
- 移动端深度适配
- 模板市场
- 自动发布到起点或番茄
- 花哨 landing page
- 复杂富文本排版

原因：先把“小说生产效率”做扎实，再考虑商业化和平台化。

## 7. 开发者 C 第一阶段目标

开发者 C 负责 B 端 UI 和 API 协作层，第一阶段目标是把 CLI MVP 变成可视化创作工作台。

第一个可验收版本必须能完成：

```text
打开 Web 工作台
→ 查看小说列表
→ 创建新小说
→ 打开章节编辑器
→ 生成第 1 章
→ 审稿
→ 重写
→ 查看版本
→ 保存人工编辑稿
→ 导出 Markdown
```

## 8. UI 验收标准

第一版 UI 达标条件：

- 能通过页面完成 CLI demo 的核心链路
- 长文本编辑不卡顿
- Agent 运行有明确状态反馈
- 失败时能看到错误信息
- 审稿结果可读、可操作
- 章节版本能查看和区分
- 页面不需要用户理解底层命令
- 不出现信息重叠、按钮挤压或正文编辑区域过小的问题

UI 的目标不是“看起来像 AI 产品”，而是让作者愿意每天在里面工作数小时。
