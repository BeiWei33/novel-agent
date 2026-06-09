# novel-agent Web 工作台

C 线 Web 工作台，使用 React + TypeScript + Vite + Tailwind。

## 启动

```powershell
npm.cmd install
npm.cmd run dev
```

默认打开：

```text
http://127.0.0.1:5173/
```

## 构建验证

```powershell
npm.cmd run typecheck
npm.cmd run build
```

## API 模式

默认使用本地 mock 数据。切真实 API 时复制 `.env.example` 为 `.env.local`，按 A 提供的服务地址调整：

```text
VITE_USE_MOCK=false
VITE_API_BASE_URL=http://127.0.0.1:3000
VITE_ENABLE_SSE=false
```

SSE 接口就绪后再打开：

```text
VITE_ENABLE_SSE=true
```

当前前端已准备 `text/event-stream` 读取器和章节操作事件映射。默认不在真实模式下额外发起 SSE 请求，避免后端尚未实现时重复调用写作接口。

## 已实现路由

```text
/novels
/novels/new
/novels/:novelId
/novels/:novelId/chapters/:chapterIndex
/agent-runs
```

## Mock 覆盖

- 3 本小说
- 每本 30 章目录
- 小说圣经、人物卡、世界观、事实表、伏笔表
- 章节正文、审稿报告、章节版本、版本对比
- AgentRun 列表、筛选、详情
- 生成、审稿、重写、保存、导出主链路
