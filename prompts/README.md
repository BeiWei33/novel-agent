# novel-agent Prompt 目录

本目录由开发者 B 维护，负责小说业务侧 Agent 的提示词模板。

约定：

- 所有 Agent 默认输出 JSON，不在 JSON 外追加解释。
- 当模型无法严格满足 schema 时，必须在 `raw_notes` 或 `issues` 字段保留原始说明，方便工程侧容错。
- Prompt 中的字段名称应和 `docs/SCHEMAS.md` 保持一致。
- 起点向更重设定、升级、长线期待；番茄向更重开篇速度、短周期爽点、情绪反馈。

MVP 推荐调用顺序：

```text
Market Agent
-> Plot Agent
-> Character Agent
-> Worldbuilding Agent
-> Chapter Writer Agent
-> Continuity Agent
-> Style Agent
-> Reviewer Agent
```

