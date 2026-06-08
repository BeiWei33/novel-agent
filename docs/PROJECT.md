# novel-agent 项目文档

版本：v0.1  
日期：2026-06-08  
语言：Rust  
定位：多 Agent 编排的中文长篇网文自动创作系统

## 1. 项目定位

`novel-agent` 是一个用 Rust 构建的多 Agent 小说创作系统，目标是面向中文网文场景，自动完成从立项、卖点分析、大纲设计、人物设定、世界观构建、章节生成、连续性检查、润色评审到重写迭代的完整创作流程。

项目对标起点、番茄等中文网文平台的内容生产需求，重点服务长篇连载型小说创作，而不是简单的一次性短文生成。

核心目标：

- 支持百万字级长篇连载规划
- 支持章节级自动生成与返工
- 支持人物、世界观、伏笔和事实一致性维护
- 支持平台化读者留存导向优化
- 支持人工作者随时介入和修改
- 支持多模型、多 Agent、多工作流扩展

一句话定位：

> `novel-agent` 不是一个简单的 AI 写作脚本，而是一个面向网文工业化生产的多 Agent 创作编排引擎。

## 2. Agent 框架选型

项目采用：

```text
Rig 作为底层 LLM Agent 能力层
novel-agent 自研小说领域 Orchestrator
```

也就是说，本项目不会完全依赖某个通用 Agent 框架来决定业务流程，而是将系统拆成两层：

```text
novel-agent 自研编排层
├── NovelWorkflow
├── AgentRole
├── ChapterPipeline
├── Memory / Facts / Outline
├── Review / Rewrite Loop
└── Platform Strategy

Rig 底层能力层
├── LLM provider
├── prompt / completion
├── tool calling
├── embeddings
├── RAG / vector store
└── streaming output
```

### 2.1 选择 Rig 的原因

Rig 是 Rust 原生的 LLM 应用开发框架，适合承担底层模型调用、工具调用、embedding、RAG 和向量检索等基础设施能力。

选择 Rig 的主要原因：

- Rust 原生，适合和本项目的主语言保持一致
- 面向模块化 LLM 应用开发，而不是只做简单 API wrapper
- 支持 Agent、tool calling、embedding、向量库和多模型 provider
- 可以作为稳定的底层能力层，减少重复造轮子
- 不强迫业务流程采用某种固定 Agent 范式

### 2.2 为什么自研 Orchestrator

小说创作不是通用聊天任务，也不是简单 ReAct Agent 能解决的问题。长篇网文创作需要稳定、可控、可追踪的生产流程。

自研 Orchestrator 负责：

- 定义小说创作工作流
- 调度不同专业 Agent
- 控制章节生成和返工逻辑
- 维护人物、世界观、事实、伏笔等长程记忆
- 判断质量评分是否达标
- 管理作品、卷、章节、摘要、审稿报告等结构化数据

项目核心价值在小说生产系统，而不是“让一个 Agent 自己想办法写”。

## 3. 系统架构

推荐模块结构：

```text
novel-agent
├── crates
│   ├── novel-agent-cli          CLI 入口
│   ├── novel-agent-core         核心领域模型与编排
│   ├── novel-agent-agents       专业 Agent 实现
│   ├── novel-agent-memory       记忆、事实、检索
│   ├── novel-agent-model        模型供应商适配
│   ├── novel-agent-evaluator    质量评估与审稿
│   └── novel-agent-storage      数据持久化
├── docs
│   └── PROJECT.md
├── examples
└── prompts
```

早期也可以先用单 crate 快速起步，等领域边界稳定后再拆 workspace。

推荐第一阶段结构：

```text
src
├── main.rs
├── config.rs
├── error.rs
├── agents
│   ├── mod.rs
│   ├── market.rs
│   ├── plot.rs
│   ├── character.rs
│   ├── worldbuilding.rs
│   ├── writer.rs
│   ├── continuity.rs
│   ├── style.rs
│   └── reviewer.rs
├── workflow
│   ├── mod.rs
│   ├── novel_creation.rs
│   └── chapter_generation.rs
├── memory
│   ├── mod.rs
│   ├── facts.rs
│   ├── summary.rs
│   └── retrieval.rs
├── model
│   ├── mod.rs
│   └── rig_provider.rs
├── storage
│   ├── mod.rs
│   └── sqlite.rs
└── domain
    ├── mod.rs
    ├── novel.rs
    ├── chapter.rs
    ├── character.rs
    └── review.rs
```

## 4. 技术栈

核心技术：

- Rust：主开发语言
- Tokio：异步运行时
- Rig：LLM Agent、tool calling、embedding、RAG 基础能力
- Clap：CLI 命令行工具
- Serde：结构化数据序列化
- SQLx：数据库访问
- SQLite：MVP 本地数据库
- PostgreSQL：后续生产数据库
- pgvector / Qdrant：后续向量检索
- Tantivy：后续本地全文检索
- Tracing：日志、调试和可观测性
- Anyhow / thiserror：错误处理

第一阶段优先本地 CLI + SQLite，避免过早引入复杂服务端和 Web UI。

## 5. 核心领域对象

### 5.1 Novel

小说项目本体。

字段草案：

```rust
pub struct Novel {
    pub id: NovelId,
    pub title: String,
    pub genre: String,
    pub target_platform: TargetPlatform,
    pub status: NovelStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 5.2 NovelBible

小说圣经，保存全书核心设定。

包括：

- 一句话卖点
- 目标读者
- 主线冲突
- 情绪价值
- 题材标签
- 世界观规则
- 主角成长线
- 禁止事项
- 文风要求

### 5.3 Character

人物卡。

包括：

- 姓名
- 身份
- 性格
- 目标
- 动机
- 秘密
- 能力
- 关系网
- 当前状态
- 成长弧线

### 5.4 Chapter

章节实体。

包括：

- 章节序号
- 标题
- 大纲
- 正文
- 摘要
- 字数
- 状态
- 评分
- 生成版本

### 5.5 Fact

事实表，用来保证长篇一致性。

示例：

```text
主角 林舟 拥有能力 时间回溯
林舟 第12章 加入 青云商会
反派 陈启明 知道 林舟真实身份
```

### 5.6 ReviewReport

章节审稿报告。

包括：

- 总分
- 爽点评分
- 节奏评分
- 人物评分
- 对话评分
- 设定一致性评分
- 章尾钩子评分
- 问题列表
- 修改建议

## 6. Agent 设计

### 6.1 Agent 抽象

核心 trait 草案：

```rust
#[async_trait::async_trait]
pub trait NovelAgent {
    fn role(&self) -> AgentRole;

    async fn run(
        &self,
        ctx: AgentContext,
        input: AgentInput,
    ) -> Result<AgentOutput, AgentError>;
}
```

Agent 不直接访问所有系统资源，而是通过 `AgentContext` 获取必要上下文。

```rust
pub struct AgentContext {
    pub novel_id: NovelId,
    pub memory: MemoryHandle,
    pub model: ModelHandle,
    pub storage: StorageHandle,
    pub constraints: AgentConstraints,
}
```

### 6.2 Orchestrator Agent

职责：

- 解析用户目标
- 选择工作流
- 调度专业 Agent
- 管理中间产物
- 判断是否需要返工
- 汇总最终输出

Orchestrator 不负责直接写正文，它负责决定谁来写、写什么、写完后谁检查。

### 6.3 Market Agent

职责：

- 分析题材潜力
- 设计平台标签
- 生成书名候选
- 生成简介
- 判断开篇卖点
- 给出起点/番茄向优化建议

输出：

- 目标读者画像
- 核心爽点
- 平台标签
- 书名候选
- 简介候选
- 风险提示

### 6.4 Plot Agent

职责：

- 生成全书主线
- 设计分卷结构
- 生成前若干章大纲
- 安排高潮、反转、伏笔、回收
- 控制剧情节奏

重点：

- 前 3 章必须强冲突
- 前 10 章必须建立核心期待
- 每章必须有推进
- 每个小剧情周期必须有明确回报

### 6.5 Character Agent

职责：

- 生成人物卡
- 维护人物目标、性格、动机
- 设计人物关系
- 检查人物行为是否符合设定
- 维护人物成长线

重点：

- 主角欲望必须清晰
- 反派不能只作为工具人
- 重要配角要有独立目标
- 人物行为要和当前信息状态一致

### 6.6 Worldbuilding Agent

职责：

- 生成世界观
- 维护势力、地图、等级、职业、规则
- 检查设定冲突
- 输出可复用世界观词条

适用题材：

- 玄幻
- 仙侠
- 都市异能
- 科幻
- 末世
- 游戏
- 无限流

### 6.7 Chapter Writer Agent

职责：

- 根据章节大纲生成正文
- 控制字数
- 控制叙事视角
- 保持网文语感
- 在章尾制造钩子

章节生成输入：

- 小说圣经
- 人物状态
- 世界观规则
- 最近章节摘要
- 当前章大纲
- 本章目标
- 禁止事项

章节生成输出：

- 章节标题
- 章节正文
- 章节摘要
- 新增事实候选
- 伏笔候选

### 6.8 Continuity Agent

职责：

- 检查前后矛盾
- 检查人物状态
- 检查能力、物品、关系变化
- 检查伏笔是否遗忘
- 提取新增事实

输出：

- 一致性问题列表
- 新增事实
- 需要更新的人物状态
- 需要更新的世界观条目

### 6.9 Style Agent

职责：

- 润色语言
- 统一叙事口吻
- 提升可读性
- 减少机械 AI 味
- 保持目标平台语感

Style Agent 不应大幅改变剧情，只做表达层优化。

### 6.10 Reviewer Agent

职责：

- 对章节评分
- 判断是否达标
- 找出问题段落
- 给出修改建议
- 决定是否进入重写流程

评分维度：

```text
开头吸引力        0-10
情节推进          0-10
爽点强度          0-10
人物表现          0-10
对话自然度        0-10
设定一致性        0-10
章尾钩子          0-10
平台适配度        0-10
```

默认章节通过线：

```text
总分 >= 75
章尾钩子 >= 7
设定一致性 >= 8
情节推进 >= 7
```

## 7. 工作流设计

### 7.1 新书创建工作流

```text
用户输入题材/创意
→ Market Agent 分析卖点
→ Plot Agent 生成主线和分卷
→ Character Agent 生成核心人物
→ Worldbuilding Agent 生成世界观
→ Reviewer Agent 评估可写性
→ Orchestrator 汇总 NovelBible
→ 保存项目
```

输出：

- 书名候选
- 简介
- 平台标签
- 目标读者
- 核心卖点
- 全书主线
- 分卷规划
- 核心人物卡
- 世界观设定
- 前 30 章大纲

### 7.2 章节生成工作流

```text
加载 NovelBible
→ 加载最近章节摘要
→ 加载人物当前状态
→ 加载相关事实和伏笔
→ Plot Agent 补全本章目标
→ Chapter Writer Agent 生成初稿
→ Continuity Agent 检查一致性
→ Style Agent 润色
→ Reviewer Agent 评分
→ 分数不足则重写
→ 保存终稿、摘要、事实变更
```

### 7.3 章节返工工作流

触发条件：

- 总分低于通过线
- 设定一致性严重失败
- 章尾没有钩子
- 人物行为明显不合理
- 用户手动要求重写

返工策略：

```text
Reviewer Agent 输出问题
→ Orchestrator 判断返工类型
→ 局部重写或整章重写
→ Continuity Agent 复查
→ Reviewer Agent 重新评分
```

### 7.4 连载更新工作流

```text
选择小说项目
→ 设置今日更新目标
→ 批量生成章节大纲
→ 逐章生成正文
→ 逐章审稿
→ 保存可发布版本
→ 生成今日更新报告
```

## 8. 记忆系统

长篇小说的核心难点是连续性。项目需要同时维护短期记忆、中期记忆和长期记忆。

### 8.1 短期记忆

包括：

- 最近 3-5 章全文
- 最近 10 章摘要
- 当前剧情目标
- 当前人物状态

用于生成当前章节。

### 8.2 中期记忆

包括：

- 当前卷大纲
- 当前卷伏笔
- 当前卷人物关系变化
- 当前卷核心矛盾

用于控制一个剧情阶段的连续性。

### 8.3 长期记忆

包括：

- 小说圣经
- 世界观规则
- 人物初始设定
- 已确认事实表
- 关键伏笔和回收记录

用于保证整本书不崩。

### 8.4 检索策略

生成章节时，系统应检索：

- 和本章人物相关的角色卡
- 和本章地点相关的世界观条目
- 和本章冲突相关的历史事实
- 未回收伏笔
- 最近章节摘要

MVP 可先使用 SQLite + 关键词检索，后续升级到 embedding + vector store。

## 9. 数据模型草案

### 9.1 novels

```sql
CREATE TABLE novels (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    genre TEXT NOT NULL,
    target_platform TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### 9.2 novel_bibles

```sql
CREATE TABLE novel_bibles (
    novel_id TEXT PRIMARY KEY,
    premise TEXT NOT NULL,
    selling_points TEXT NOT NULL,
    target_reader TEXT NOT NULL,
    tone TEXT NOT NULL,
    world_rules TEXT NOT NULL,
    constraints TEXT NOT NULL,
    FOREIGN KEY (novel_id) REFERENCES novels(id)
);
```

### 9.3 characters

```sql
CREATE TABLE characters (
    id TEXT PRIMARY KEY,
    novel_id TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    personality TEXT NOT NULL,
    motivation TEXT NOT NULL,
    current_state TEXT NOT NULL,
    relationship_map TEXT NOT NULL,
    arc TEXT NOT NULL,
    FOREIGN KEY (novel_id) REFERENCES novels(id)
);
```

### 9.4 chapters

```sql
CREATE TABLE chapters (
    id TEXT PRIMARY KEY,
    novel_id TEXT NOT NULL,
    volume_index INTEGER NOT NULL,
    chapter_index INTEGER NOT NULL,
    title TEXT NOT NULL,
    outline TEXT NOT NULL,
    content TEXT,
    summary TEXT,
    status TEXT NOT NULL,
    score INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (novel_id) REFERENCES novels(id)
);
```

### 9.5 facts

```sql
CREATE TABLE facts (
    id TEXT PRIMARY KEY,
    novel_id TEXT NOT NULL,
    chapter_id TEXT,
    subject TEXT NOT NULL,
    predicate TEXT NOT NULL,
    object TEXT NOT NULL,
    importance INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (novel_id) REFERENCES novels(id),
    FOREIGN KEY (chapter_id) REFERENCES chapters(id)
);
```

### 9.6 review_reports

```sql
CREATE TABLE review_reports (
    id TEXT PRIMARY KEY,
    chapter_id TEXT NOT NULL,
    total_score INTEGER NOT NULL,
    hook_score INTEGER NOT NULL,
    pacing_score INTEGER NOT NULL,
    character_score INTEGER NOT NULL,
    continuity_score INTEGER NOT NULL,
    issues TEXT NOT NULL,
    suggestions TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (chapter_id) REFERENCES chapters(id)
);
```

## 10. CLI 设计

MVP 先做 CLI。

### 10.1 创建新书

```bash
novel-agent new "都市重生商业文，主角回到十年前，从外卖站开始逆袭"
```

输出：

- 小说项目 ID
- 书名候选
- 核心卖点
- 简介
- 前 30 章大纲

### 10.2 生成大纲

```bash
novel-agent outline --novel-id <id> --chapters 30
```

### 10.3 生成章节

```bash
novel-agent write --novel-id <id> --chapter 1
```

### 10.4 审稿

```bash
novel-agent review --novel-id <id> --chapter 1
```

### 10.5 重写

```bash
novel-agent rewrite --novel-id <id> --chapter 1
```

### 10.6 导出

```bash
novel-agent export --novel-id <id> --format markdown
```

## 11. MVP 范围

第一阶段必须完成：

- Rust CLI 骨架
- 配置文件读取
- Rig 模型调用封装
- Agent trait
- Orchestrator 基础调度
- 新书创建流程
- 小说圣经生成
- 人物卡生成
- 前 30 章大纲生成
- 单章正文生成
- 章节审稿
- 简单返工
- SQLite 持久化
- Markdown 导出

第一阶段暂不做：

- Web UI
- 多用户系统
- 自动发布到真实平台
- 复杂向量数据库部署
- 账号矩阵管理
- 收费系统

## 12. 安全与合规

项目需要避免：

- 直接复制已有小说正文
- 仿写指定在世作者的独特文风
- 生成侵权 IP 同人并用于商业发布
- 伪造真实作者身份
- 绕过平台审核机制
- 生成违法违规内容

系统应保留：

- Prompt 记录
- 生成版本
- 人工修改记录
- 审稿报告
- 内容来源说明

目标是辅助原创创作，而不是洗稿或侵权生产。

## 13. 质量标准

章节质量默认标准：

- 每章必须有明确剧情推进
- 每章必须有情绪回报或期待强化
- 章尾必须有钩子
- 主角目标必须清晰
- 人物行为必须符合当前信息状态
- 世界观规则不能随意变动
- 重要事实必须进入事实表
- 伏笔必须有记录和回收计划

平台化倾向：

- 起点向：更重视设定、升级线、长期期待、体系感
- 番茄向：更重视开篇速度、情绪反馈、短周期爽点、易读性

## 14. 后续路线图

### 阶段 1：CLI MVP

目标：跑通从新书创建到单章生成的完整流程。

交付：

- CLI
- SQLite
- Rig provider
- 基础 Agent
- 基础工作流
- Markdown 导出

### 阶段 2：记忆增强

目标：解决长篇连续性问题。

交付：

- 事实表
- 伏笔表
- 人物状态追踪
- 摘要压缩
- embedding 检索

### 阶段 3：质量评估

目标：让系统能判断章节是否适合连载。

交付：

- 评分 Rubric
- 审稿报告
- 自动返工
- 多版本对比
- 平台适配评分

### 阶段 4：Web 创作台

目标：提供可视化小说生产工作台。

交付：

- 作品管理
- 章节编辑器
- 人物面板
- 世界观面板
- 审稿面板
- 重写对比

### 阶段 5：生产级系统

目标：支持团队化、多作品、多模型生产。

交付：

- 任务队列
- 多模型路由
- 成本统计
- 权限管理
- 团队协作
- API 服务

## 15. 第一版实现顺序

推荐开发顺序：

1. 初始化 Rust 项目
2. 定义领域模型
3. 定义 Agent trait
4. 接入 Rig provider
5. 实现 Market Agent
6. 实现 Plot Agent
7. 实现 Character Agent
8. 实现 Chapter Writer Agent
9. 实现 Reviewer Agent
10. 实现 SQLite 存储
11. 实现 CLI 命令
12. 跑通新书创建和单章生成

第一版不要追求 Agent 完全自治，先追求流程稳定、数据结构清晰、输出可控。

## 16. 关键设计判断

本项目最重要的设计判断是：

```text
通用 LLM 能力交给 Rig
小说业务流程由 novel-agent 自己掌控
```

这样可以同时获得 Rust LLM 生态的基础设施能力，以及对中文网文生产流程的强控制力。

长期来看，`novel-agent` 的竞争力不在于“能不能调用模型”，而在于是否能把网文创作拆成稳定、可评估、可返工、可持续连载的生产系统。
