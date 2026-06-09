import { Navigate, Route, Routes } from "react-router-dom";
import { AppShell } from "./components/AppShell";
import { AgentRunsPage } from "./routes/AgentRunsPage";
import { ChapterEditorPage } from "./routes/ChapterEditorPage";
import { JobsPage } from "./routes/JobsPage";
import { NewNovelPage } from "./routes/NewNovelPage";
import { NovelListPage } from "./routes/NovelListPage";
import { NovelWorkspacePage } from "./routes/NovelWorkspacePage";

export function App() {
  return (
    <Routes>
      <Route element={<AppShell />}>
        <Route index element={<Navigate to="/novels" replace />} />
        <Route path="/novels" element={<NovelListPage />} />
        <Route path="/novels/new" element={<NewNovelPage />} />
        <Route path="/novels/:novelId" element={<NovelWorkspacePage />} />
        <Route path="/novels/:novelId/chapters/:chapterIndex" element={<ChapterEditorPage />} />
        <Route path="/jobs" element={<JobsPage />} />
        <Route path="/agent-runs" element={<AgentRunsPage />} />
      </Route>
    </Routes>
  );
}
