import { create } from "zustand";
import { persist } from "zustand/middleware";

type WorkspacePanel = "outline" | "review" | "agent";
export type ProgressKind = "write" | "review" | "rewrite" | "save";
export type ProgressStatus = "running" | "success" | "error";
export type EditorDensity = "comfortable" | "compact";

export interface ProgressStep {
  id: string;
  label: string;
  detail: string;
  status: ProgressStatus;
  timestamp: string;
}

export interface ChapterProgress {
  id: string;
  chapterId: string;
  kind: ProgressKind;
  status: ProgressStatus;
  startedAt: string;
  finishedAt?: string;
  steps: ProgressStep[];
}

interface WorkspaceStore {
  rightPanel: WorkspacePanel;
  setRightPanel: (panel: WorkspacePanel) => void;
  editorDensity: EditorDensity;
  setEditorDensity: (density: EditorDensity) => void;
  editorDrafts: Record<string, string>;
  setEditorDraft: (chapterId: string, content: string) => void;
  clearEditorDraft: (chapterId: string) => void;
  progressByChapter: Record<string, ChapterProgress>;
  startProgress: (chapterId: string, kind: ProgressKind) => string;
  appendProgressStep: (chapterId: string, progressId: string, step: Omit<ProgressStep, "id" | "timestamp">) => void;
  finishProgress: (chapterId: string, progressId: string, status: Exclude<ProgressStatus, "running">, detail: string) => void;
  clearProgress: (chapterId: string) => void;
}

export const useWorkspaceStore = create<WorkspaceStore>()(
  persist(
    (set) => ({
      rightPanel: "outline",
      setRightPanel: (rightPanel) => set({ rightPanel }),
      editorDensity: "comfortable",
      setEditorDensity: (editorDensity) => set({ editorDensity }),
      editorDrafts: {},
      setEditorDraft: (chapterId, content) =>
        set((state) => ({
          editorDrafts: { ...state.editorDrafts, [chapterId]: content },
        })),
      clearEditorDraft: (chapterId) =>
        set((state) => {
          const next = { ...state.editorDrafts };
          delete next[chapterId];
          return { editorDrafts: next };
        }),
      progressByChapter: {},
      startProgress: (chapterId, kind) => {
        const progressId = crypto.randomUUID();
        const now = new Date().toISOString();
        set((state) => ({
          progressByChapter: {
            ...state.progressByChapter,
            [chapterId]: {
              id: progressId,
              chapterId,
              kind,
              status: "running",
              startedAt: now,
              steps: [
                {
                  id: crypto.randomUUID(),
                  label: progressKindLabels[kind],
                  detail: "任务已进入队列。",
                  status: "running",
                  timestamp: now,
                },
              ],
            },
          },
        }));
        return progressId;
      },
      appendProgressStep: (chapterId, progressId, step) =>
        set((state) => {
          const current = state.progressByChapter[chapterId];
          if (!current || current.id !== progressId || current.status !== "running") {
            return state;
          }
          return {
            progressByChapter: {
              ...state.progressByChapter,
              [chapterId]: {
                ...current,
                steps: [
                  ...current.steps,
                  {
                    id: crypto.randomUUID(),
                    timestamp: new Date().toISOString(),
                    ...step,
                  },
                ],
              },
            },
          };
        }),
      finishProgress: (chapterId, progressId, status, detail) =>
        set((state) => {
          const current = state.progressByChapter[chapterId];
          if (!current || current.id !== progressId) {
            return state;
          }
          const now = new Date().toISOString();
          return {
            progressByChapter: {
              ...state.progressByChapter,
              [chapterId]: {
                ...current,
                status,
                finishedAt: now,
                steps: [
                  ...current.steps,
                  {
                    id: crypto.randomUUID(),
                    label: status === "success" ? "完成" : "失败",
                    detail,
                    status,
                    timestamp: now,
                  },
                ],
              },
            },
          };
        }),
      clearProgress: (chapterId) =>
        set((state) => {
          const next = { ...state.progressByChapter };
          delete next[chapterId];
          return { progressByChapter: next };
        }),
    }),
    {
      name: "novel-agent-workspace",
      partialize: (state) => ({
        rightPanel: state.rightPanel,
        editorDensity: state.editorDensity,
        editorDrafts: state.editorDrafts,
      }),
    },
  ),
);

const progressKindLabels: Record<ProgressKind, string> = {
  write: "生成章节",
  review: "审稿",
  rewrite: "重写",
  save: "保存人工编辑稿",
};
