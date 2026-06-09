# novel-agent Prompt 目录

本目录由开发者 B 维护，负责小说业务侧 Agent 的提示词模板。

当前 Prompt bundle：`b-quality-2026-06-09-r3`，版本记录见 `docs/PROMPT_CHANGELOG.md`。

约定：

- 所有 Agent 默认输出 JSON，不在 JSON 外追加解释。
- 所有 Agent 输出必须使用 `docs/SCHEMAS.md` 中的 `AgentOutputEnvelope`：`role`、`structured`、`raw_notes`。
- 运行时输入由工程侧包装成 `AgentInputEnvelope`：`task`、`instructions`、`payload`、`context`。
- 各 Prompt 文件中的“输入”示例描述的是 `payload`，不是完整运行时 envelope。
- 当模型无法严格满足 schema 时，仍要输出合法 JSON，并在 `raw_notes` 内说明缺口。
- `raw_text` 是工程侧保存的完整原始响应，不由模型生成。
- `parse_error` 只由工程侧生成，不由模型生成。
- Prompt 中的字段名称应和 `docs/SCHEMAS.md` 保持一致。
- 起点向更重设定、升级、长线期待；番茄向更重开篇速度、短周期爽点、情绪反馈。
- 事实统一使用 `FactTriple`：`subject`、`predicate`、`object`、`importance`。

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
