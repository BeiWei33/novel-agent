# novel-agent 双人开发分工计划

版本：v0.1  
日期：2026-06-08  
团队规模：2 人  
目标：两人并行完成 CLI MVP，跑通从新书创建到单章生成、审稿、重写的闭环。

## 1. 分工原则

项目分成两条主线：

```text
开发者 A：Rust 核心工程线
开发者 B：小说 Agent 业务线
```

开发者 A 负责让系统稳定运行，包括项目骨架、CLI、存储、模型适配、工作流调度和基础设施。

开发者 B 负责让系统真正会写小说，包括 Agent 角色设计、Prompt、小说圣经、人物卡、大纲、章节生成、审稿标准和测试样例。

两人的交界面是：

```text
Agent trait
AgentInput
AgentOutput
Workflow
Domain Model
```

这些接口需要优先定义，后续两边围绕接口并行开发。

## 2. 开发者 A：Rust 核心工程线

### 2.1 主要职责

开发者 A 负责项目的工程骨架和运行时能力。

职责范围：

- 初始化 Rust 项目
- 设计 crate/module 结构
- 实现 CLI 命令
- 接入 Rig
- 封装模型调用层
- 实现 Agent trait 和 Orchestrator
- 实现 SQLite 存储
- 实现配置、日志、错误处理
- 实现基础工作流执行器
- 提供可被开发者 B 调用的 Agent 接口

### 2.2 第一阶段任务

优先级 P0：

- 初始化 `cargo` 项目
- 建立基础目录结构
- 添加依赖：`tokio`、`clap`、`serde`、`sqlx`、`tracing`、`thiserror`、`anyhow`、`rig`
- 定义 `NovelAgent` trait
- 定义 `AgentContext`、`AgentInput`、`AgentOutput`
- 定义核心错误类型
- 定义模型调用抽象 `ModelClient`
- 用 Rig 实现第一版 `RigModelClient`
- 实现 `novel-agent new` 命令骨架
- 实现 `novel-agent write` 命令骨架

优先级 P1：

- 实现 SQLite schema
- 实现 `NovelRepository`
- 实现 `ChapterRepository`
- 实现 `CharacterRepository`
- 实现 `ReviewReportRepository`
- 实现配置文件读取
- 实现日志和请求追踪
- 实现 Markdown 导出

优先级 P2：

- 支持模型 provider 切换
- 支持流式输出
- 支持任务运行记录
- 支持 Agent 执行耗时和 token 统计

### 2.3 交付物

开发者 A 需要交付：

```text
src/main.rs
src/config.rs
src/error.rs
src/domain/*
src/model/*
src/storage/*
src/workflow/*
src/agents/mod.rs
```

核心接口：

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

CLI 最小可用命令：

```bash
novel-agent new "<创意>"
novel-agent outline --novel-id <id> --chapters 30
novel-agent write --novel-id <id> --chapter 1
novel-agent review --novel-id <id> --chapter 1
novel-agent export --novel-id <id> --format markdown
```

## 3. 开发者 B：小说 Agent 业务线

### 3.1 主要职责

开发者 B 负责小说创作业务能力。

职责范围：

- 设计各 Agent 的职责边界
- 编写 Prompt 模板
- 定义小说圣经结构
- 定义人物卡结构
- 定义世界观结构
- 定义章节大纲结构
- 定义审稿评分标准
- 设计起点/番茄平台化策略
- 设计测试用小说样例
- 调整生成效果

### 3.2 第一阶段任务

优先级 P0：

- 编写 Market Agent Prompt
- 编写 Plot Agent Prompt
- 编写 Character Agent Prompt
- 编写 Chapter Writer Agent Prompt
- 编写 Reviewer Agent Prompt
- 定义 `NovelBible` 输出 JSON schema
- 定义 `CharacterCard` 输出 JSON schema
- 定义 `ChapterOutline` 输出 JSON schema
- 定义 `ReviewReport` 输出 JSON schema
- 准备 3 个测试题材样例

优先级 P1：

- 编写 Continuity Agent Prompt
- 编写 Style Agent Prompt
- 编写 Worldbuilding Agent Prompt
- 设计章节通过线
- 设计章节返工策略
- 设计伏笔和事实提取格式
- 设计起点向和番茄向的不同生成策略

优先级 P2：

- 建立 Prompt 版本管理
- 建立人工评测表
- 对不同模型输出做效果对比
- 优化网文语感
- 优化章尾钩子生成

### 3.3 交付物

开发者 B 需要交付：

```text
prompts/market_agent.md
prompts/plot_agent.md
prompts/character_agent.md
prompts/worldbuilding_agent.md
prompts/chapter_writer_agent.md
prompts/continuity_agent.md
prompts/style_agent.md
prompts/reviewer_agent.md
examples/urban_rebirth.md
examples/fantasy_upgrade.md
examples/romance_comeback.md
docs/RUBRIC.md
```

核心输出格式：

```text
NovelBible
CharacterCard
WorldSetting
ChapterOutline
ChapterDraft
ContinuityReport
ReviewReport
RewriteInstruction
```

## 4. 双方共同任务

以下任务必须两人一起定，不建议单独决定：

- `AgentInput` 和 `AgentOutput` 的结构
- Agent 返回结果是纯文本、JSON，还是二者混合
- 小说圣经字段
- 章节大纲字段
- 审稿评分标准
- MVP 命令行交互方式
- 第一批测试题材
- 默认目标平台策略

建议第一天先开一次接口对齐会，只定三件事：

1. Agent 的输入输出结构
2. 新书创建工作流的中间产物
3. 单章生成工作流的通过标准

## 5. 推荐开发节奏

### 第 1 天：项目骨架和接口

开发者 A：

- 初始化 Rust 项目
- 加基础依赖
- 建目录结构
- 定义核心 trait 和 domain model 初稿

开发者 B：

- 编写 Agent 职责说明
- 编写 NovelBible schema
- 编写前 3 个核心 Prompt
- 准备测试题材样例

当天共同验收：

- `cargo check` 通过
- Agent trait 确定
- NovelBible 字段确定
- 新书创建流程确定

### 第 2-3 天：新书创建流程

开发者 A：

- 实现 Rig 模型调用
- 实现 `new` 命令
- 实现 Orchestrator 调用链
- 实现结果保存

开发者 B：

- 完成 Market / Plot / Character Prompt
- 调整输出 JSON
- 验证 3 个题材生成效果

阶段验收：

```bash
novel-agent new "都市重生商业文，主角回到十年前，从外卖站开始逆袭"
```

需要成功输出：

- 书名候选
- 简介
- 核心卖点
- 主角设定
- 前 30 章大纲

### 第 4-5 天：单章生成流程

开发者 A：

- 实现 `write` 命令
- 实现章节存储
- 实现章节上下文组装
- 实现 Markdown 导出

开发者 B：

- 完成 Chapter Writer Prompt
- 完成 Reviewer Prompt
- 完成章节评分 Rubric
- 调试章尾钩子和网文节奏

阶段验收：

```bash
novel-agent write --novel-id <id> --chapter 1
novel-agent review --novel-id <id> --chapter 1
```

需要成功输出：

- 章节标题
- 章节正文
- 章节摘要
- 审稿评分
- 修改建议

### 第 6-7 天：审稿与返工闭环

开发者 A：

- 实现 `review` 命令
- 实现 `rewrite` 命令
- 实现 ReviewReport 存储
- 实现章节版本记录

开发者 B：

- 完成返工策略
- 完成 Style Agent Prompt
- 完成 Continuity Agent Prompt
- 调整评分通过线

阶段验收：

```bash
novel-agent review --novel-id <id> --chapter 1
novel-agent rewrite --novel-id <id> --chapter 1
```

需要成功完成：

- 审稿报告可保存
- 低分章节可自动返工
- 重写版本可对比
- 最终章节可导出

## 6. MVP 验收标准

MVP 完成时，系统必须支持：

- 创建一个小说项目
- 自动生成小说圣经
- 自动生成主角和核心人物
- 自动生成前 30 章大纲
- 自动生成第 1 章正文
- 自动审稿并给出评分
- 低分章节可以触发重写
- 所有结果保存到本地数据库
- 可以导出 Markdown

推荐演示命令：

```bash
novel-agent new "都市重生商业文，主角回到十年前，从外卖站开始逆袭"
novel-agent write --novel-id <id> --chapter 1
novel-agent review --novel-id <id> --chapter 1
novel-agent rewrite --novel-id <id> --chapter 1
novel-agent export --novel-id <id> --format markdown
```

## 7. 风险和边界

主要风险：

- Rust LLM 生态变化较快，Rig 版本 API 可能调整
- Agent 输出 JSON 不稳定，需要容错解析
- 章节质量不只取决于工程，也强依赖 Prompt 和模型
- 长篇连续性不能在 MVP 一次性解决
- 两人同时改领域模型容易冲突

应对策略：

- A 负责接口稳定，B 通过 schema 提需求
- Prompt 输出先强制 JSON，失败时保留原始文本
- MVP 只做最近章节摘要和基础事实表
- 每天固定同步一次 domain model 变更
- 所有 Agent 输出都保存原始响应，便于调试

## 8. 每日同步模板

每天同步只回答 4 个问题：

```text
昨天完成了什么？
今天准备完成什么？
接口或数据结构有没有变化？
有没有阻塞对方的事情？
```

开发者 A 重点同步：

- trait 是否变了
- schema 是否变了
- CLI 命令是否变了
- 存储字段是否变了

开发者 B 重点同步：

- Prompt 输出格式是否变了
- 审稿标准是否变了
- 测试样例是否变了
- Agent 需要的新上下文字段是否变了

## 9. 推荐任务归属汇总

| 模块 | 负责人 | 备注 |
| --- | --- | --- |
| Rust 项目初始化 | A | P0 |
| CLI | A | P0 |
| Rig 接入 | A | P0 |
| Agent trait | A 主导，B 参与 | P0 |
| Orchestrator | A | P0 |
| SQLite 存储 | A | P1 |
| Domain model | A 主导，B 参与 | P0 |
| NovelBible schema | B 主导，A 参与 | P0 |
| Prompt 模板 | B | P0 |
| Market Agent | B 设计，A 接入 | P0 |
| Plot Agent | B 设计，A 接入 | P0 |
| Character Agent | B 设计，A 接入 | P0 |
| Chapter Writer Agent | B 设计，A 接入 | P0 |
| Reviewer Agent | B 设计，A 接入 | P0 |
| Continuity Agent | B 设计，A 接入 | P1 |
| Style Agent | B 设计，A 接入 | P1 |
| Markdown 导出 | A | P1 |
| 测试题材样例 | B | P0 |
| MVP 演示脚本 | A 主导，B 参与 | P1 |

## 10. 结论

两人分工建议：

```text
A 负责系统能跑、能存、能调度、能导出。
B 负责系统会写、会评、会改、像网文。
```

第一周的目标不是做完完整平台，而是跑通一条可信闭环：

```text
创意输入
→ 小说圣经
→ 人物和大纲
→ 第一章正文
→ 审稿评分
→ 自动重写
→ Markdown 导出
```

只要这条链路稳定，后续再扩展记忆、向量检索、Web 创作台和团队协作系统。
