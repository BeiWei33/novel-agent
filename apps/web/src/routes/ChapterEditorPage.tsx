import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import CodeMirror from "@uiw/react-codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { Download, Play, RotateCcw, Save, SearchCheck, Wand2 } from "lucide-react";
import type { Chapter, ChapterOutline, ChapterVersion } from "../types/domain";
import type { ChapterOperation, ChapterStreamEvent } from "../types/api";
import { api, apiConfig, queryKeys } from "../lib/api";
import { type ChapterProgress, type EditorDensity, type ProgressKind, useWorkspaceStore } from "../lib/store";
import { cn } from "../lib/cn";
import { agentRoleLabels, agentTaskLabels, chapterStatusLabels, clampScore, formatDateTime, formatDuration } from "../lib/format";
import { ChapterTree } from "../features/chapters/ChapterTree";
import { ReviewPanel } from "../features/chapters/ReviewPanel";
import { VersionList } from "../features/chapters/VersionList";
import { VersionDiffPanel } from "../features/chapters/VersionDiffPanel";
import { OperationTimeline } from "../features/chapters/OperationTimeline";
import { MarkdownPreview } from "../features/chapters/MarkdownPreview";
import { DraftDiffPanel } from "../features/chapters/DraftDiffPanel";
import { QualityPanel } from "../features/chapters/QualityPanel";
import { PageHeader } from "../components/PageHeader";
import { LoadingState } from "../components/LoadingState";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Section } from "../components/ui/Section";
import { StatusBanner } from "../components/StatusBanner";
import { Tabs } from "../components/ui/Tabs";

type RightPanel = "outline" | "review" | "quality" | "agent";
type EditorView = "edit" | "preview" | "diff";

const panelTabs: Array<{ value: RightPanel; label: string }> = [
  { value: "outline", label: "大纲" },
  { value: "review", label: "审稿" },
  { value: "quality", label: "质量" },
  { value: "agent", label: "Agent" },
];

export function ChapterEditorPage() {
  const { novelId = "", chapterIndex = "1" } = useParams();
  const chapterNumber = Number(chapterIndex);
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [selectedVersion, setSelectedVersion] = useState<ChapterVersion | null>(null);
  const [editorView, setEditorView] = useState<EditorView>("edit");
  const {
    rightPanel,
    setRightPanel,
    editorDensity,
    setEditorDensity,
    editorDrafts,
    setEditorDraft,
    clearEditorDraft,
    progressByChapter,
    startProgress,
    appendProgressStep,
    finishProgress,
    clearProgress,
  } = useWorkspaceStore();

  const detailQuery = useQuery({
    queryKey: queryKeys.novel(novelId),
    queryFn: () => api.getNovel(novelId),
    enabled: Boolean(novelId),
  });
  const chaptersQuery = useQuery({
    queryKey: queryKeys.chapters(novelId),
    queryFn: () => api.getChapters(novelId),
    enabled: Boolean(novelId),
  });
  const chapterQuery = useQuery({
    queryKey: queryKeys.chapter(novelId, chapterNumber),
    queryFn: () => api.getChapter(novelId, chapterNumber),
    enabled: Boolean(novelId && chapterNumber),
  });
  const versionsQuery = useQuery({
    queryKey: ["chapter-versions", novelId, chapterNumber],
    queryFn: () => api.getChapterVersions(novelId, chapterNumber),
    enabled: Boolean(novelId && chapterNumber),
  });
  const reviewQuery = useQuery({
    queryKey: ["chapter-review", novelId, chapterNumber],
    queryFn: () => api.getReviewReport(novelId, chapterNumber),
    enabled: Boolean(novelId && chapterNumber),
  });
  const continuityQuery = useQuery({
    queryKey: queryKeys.continuity(novelId, chapterNumber),
    queryFn: () => api.getContinuityReport(novelId, chapterNumber),
    enabled: Boolean(novelId && chapterNumber),
  });
  const runsQuery = useQuery({
    queryKey: queryKeys.agentRunList(novelId),
    queryFn: () => api.getAgentRuns(novelId),
    enabled: Boolean(novelId),
    refetchInterval: 10_000,
  });

  const chapter = chapterQuery.data;
  const outline = useMemo(
    () => detailQuery.data?.chapter_outlines.find((item) => item.chapter_index === chapterNumber),
    [chapterNumber, detailQuery.data?.chapter_outlines],
  );
  const editorValue = chapter ? (editorDrafts[chapter.id] ?? chapter.content ?? "") : "";
  const savedContent = chapter?.content ?? "";
  const isDirty = chapter ? editorValue !== savedContent : false;
  const currentProgress = chapter ? progressByChapter[chapter.id] : undefined;
  const manualSaveEnabled = apiConfig.manualSaveEnabled;

  function beginProgress(kind: ProgressKind): MutationProgressContext | undefined {
    if (!chapter) {
      return undefined;
    }
    const progressId = startProgress(chapter.id, kind);
    scheduleOperationProgress(chapter.id, progressId, kind, novelId, chapterNumber, appendProgressStep);
    return { chapterId: chapter.id, progressId };
  }

  async function invalidateChapter(updated?: Chapter) {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.novels }),
      queryClient.invalidateQueries({ queryKey: queryKeys.novel(novelId) }),
      queryClient.invalidateQueries({ queryKey: queryKeys.chapters(novelId) }),
      queryClient.invalidateQueries({ queryKey: queryKeys.chapter(novelId, chapterNumber) }),
      queryClient.invalidateQueries({ queryKey: ["chapter-versions", novelId, chapterNumber] }),
      queryClient.invalidateQueries({ queryKey: ["chapter-review", novelId, chapterNumber] }),
      queryClient.invalidateQueries({ queryKey: queryKeys.continuity(novelId, chapterNumber) }),
      queryClient.invalidateQueries({ queryKey: queryKeys.agentRunsRoot }),
    ]);
    if (updated) {
      clearEditorDraft(updated.id);
      setSelectedVersion(null);
    }
  }

  const writeMutation = useMutation({
    mutationFn: () => api.writeChapter(novelId, chapterNumber),
    onMutate: () => {
      setRightPanel("agent");
      return beginProgress("write");
    },
    onSuccess: (updated, _variables, context) => {
      finishMutationProgress(context, finishProgress, "success", `已保存为 v${updated.version}，正文 ${updated.word_count} 字。`);
      return invalidateChapter(updated);
    },
    onError: (error, _variables, context) => {
      finishMutationProgress(context, finishProgress, "error", mutationErrorMessage(error) ?? "章节生成失败。");
    },
  });
  const reviewMutation = useMutation({
    mutationFn: () => api.reviewChapter(novelId, chapterNumber),
    onMutate: () => {
      setRightPanel("review");
      return beginProgress("review");
    },
    onSuccess: (report, _variables, context) => {
      finishMutationProgress(context, finishProgress, "success", `审稿完成，评分 ${report.total_score}，${report.passed ? "通过" : "需要返工"}。`);
      return invalidateChapter();
    },
    onError: (error, _variables, context) => {
      finishMutationProgress(context, finishProgress, "error", mutationErrorMessage(error) ?? "审稿失败。");
    },
  });
  const rewriteMutation = useMutation({
    mutationFn: () => api.rewriteChapter(novelId, chapterNumber),
    onMutate: () => {
      setRightPanel("agent");
      return beginProgress("rewrite");
    },
    onSuccess: (updated, _variables, context) => {
      finishMutationProgress(context, finishProgress, "success", `重写完成，已保存为 v${updated.version}。`);
      return invalidateChapter(updated);
    },
    onError: (error, _variables, context) => {
      finishMutationProgress(context, finishProgress, "error", mutationErrorMessage(error) ?? "重写失败。");
    },
  });
  const saveMutation = useMutation({
    mutationFn: () => api.saveChapterContent(novelId, chapterNumber, editorValue),
    onMutate: () => beginProgress("save"),
    onSuccess: (updated, _variables, context) => {
      finishMutationProgress(context, finishProgress, "success", `人工编辑稿已保存为 v${updated.version}。`);
      return invalidateChapter(updated);
    },
    onError: (error, _variables, context) => {
      finishMutationProgress(context, finishProgress, "error", mutationErrorMessage(error) ?? "保存失败。");
    },
  });
  const activeOperation = writeMutationLabel(writeMutation.isPending, reviewMutation.isPending, rewriteMutation.isPending, saveMutation.isPending);
  const operationError = mutationErrorMessage(writeMutation.error ?? reviewMutation.error ?? rewriteMutation.error ?? saveMutation.error);
  const isMutating = writeMutation.isPending || reviewMutation.isPending || rewriteMutation.isPending || saveMutation.isPending;

  const writeCurrentChapter = useCallback(() => {
    if (!writeMutation.isPending) {
      writeMutation.mutate();
    }
  }, [writeMutation]);

  const reviewCurrentChapter = useCallback(() => {
    if (!reviewMutation.isPending && editorValue) {
      reviewMutation.mutate();
    }
  }, [editorValue, reviewMutation]);

  const rewriteCurrentChapter = useCallback(() => {
    if (!rewriteMutation.isPending && editorValue) {
      rewriteMutation.mutate();
    }
  }, [editorValue, rewriteMutation]);

  const saveCurrentDraft = useCallback(() => {
    if (!saveMutation.isPending && editorValue.trim() && isDirty && manualSaveEnabled) {
      saveMutation.mutate();
    }
  }, [editorValue, isDirty, manualSaveEnabled, saveMutation]);

  useEffect(() => {
    if (!isDirty) {
      return;
    }

    function handleBeforeUnload(event: BeforeUnloadEvent) {
      event.preventDefault();
      event.returnValue = "";
    }

    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => window.removeEventListener("beforeunload", handleBeforeUnload);
  }, [isDirty]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      const hasModifier = event.ctrlKey || event.metaKey;
      if (!hasModifier || isMutating || event.defaultPrevented) {
        return;
      }

      const key = event.key.toLowerCase();
      if (key === "s") {
        event.preventDefault();
        saveCurrentDraft();
        return;
      }

      if (event.key === "Enter") {
        event.preventDefault();
        if (event.altKey) {
          rewriteCurrentChapter();
        } else if (event.shiftKey) {
          reviewCurrentChapter();
        } else {
          writeCurrentChapter();
        }
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isMutating, reviewCurrentChapter, rewriteCurrentChapter, saveCurrentDraft, writeCurrentChapter]);

  function exportChapter() {
    if (!chapter) {
      return;
    }
    const blob = new Blob([editorValue], { type: "text/markdown;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${chapter.chapter_index}-${chapter.title}.md`;
    anchor.click();
    URL.revokeObjectURL(url);
  }

  if (chapterQuery.isLoading || chaptersQuery.isLoading) {
    return <LoadingState label="打开章节编辑器" />;
  }

  if (!chapter) {
    return <div className="p-4 text-sm text-danger">章节不存在</div>;
  }

  return (
    <div className="min-h-screen">
      <PageHeader
        title={`第 ${chapter.chapter_index} 章 ${chapter.title}`}
        meta={
          <>
            <Link to={`/novels/${chapter.novel_id}`} className="text-accent hover:underline">
              {detailQuery.data?.novel.title ?? "小说工作台"}
            </Link>
            <Badge tone={chapter.status === "rewrite_needed" ? "rose" : chapter.status === "reviewed" ? "teal" : "slate"}>
              {chapterStatusLabels[chapter.status]}
            </Badge>
            <span>{chapter.word_count} 字</span>
            <span>v{chapter.version}</span>
            <span>评分 {clampScore(chapter.score)}</span>
          </>
        }
        actions={
          <>
            <Button variant="secondary" onClick={writeCurrentChapter} disabled={writeMutation.isPending}>
              <Play className="h-4 w-4" />
              {writeMutation.isPending ? "生成中" : "生成"}
            </Button>
            <Button variant="secondary" onClick={reviewCurrentChapter} disabled={reviewMutation.isPending || !editorValue}>
              <SearchCheck className="h-4 w-4" />
              {reviewMutation.isPending ? "审稿中" : "审稿"}
            </Button>
            <Button variant="secondary" onClick={rewriteCurrentChapter} disabled={rewriteMutation.isPending || !editorValue}>
              <Wand2 className="h-4 w-4" />
              {rewriteMutation.isPending ? "重写中" : "重写"}
            </Button>
            <Button
              variant="primary"
              onClick={saveCurrentDraft}
              disabled={saveMutation.isPending || !editorValue.trim() || !isDirty || !manualSaveEnabled}
              title={manualSaveEnabled ? "保存人工编辑稿" : "当前模式暂不可保存"}
            >
              <Save className="h-4 w-4" />
              {saveMutation.isPending ? "保存中" : "保存"}
            </Button>
            <Button variant="secondary" onClick={exportChapter} disabled={!editorValue}>
              <Download className="h-4 w-4" />
              导出
            </Button>
          </>
        }
      />
      {activeOperation ? (
        <StatusBanner title={activeOperation.title}>{activeOperation.description}</StatusBanner>
      ) : null}
      {operationError ? (
        <StatusBanner tone="danger" title="操作失败">
          {operationError}
        </StatusBanner>
      ) : null}
      {isDirty ? (
        <StatusBanner title="有未保存编辑">
          {manualSaveEnabled
            ? "当前草稿只保存在浏览器会话中，保存后会生成新的章节版本。"
            : "当前模式暂不可保存；可先导出当前草稿。"}
        </StatusBanner>
      ) : null}
      <MobileChapterPicker
        chapters={chaptersQuery.data ?? []}
        activeIndex={chapterNumber}
        onChange={(nextChapter) => navigate(`/novels/${novelId}/chapters/${nextChapter}`)}
      />

      <div className="grid min-h-[620px] grid-cols-1 lg:grid-cols-[260px_minmax(0,1fr)_360px]">
        <div className="hidden lg:block">
          <ChapterTree novelId={novelId} chapters={chaptersQuery.data ?? []} activeIndex={chapterNumber} />
        </div>

        <section className="min-w-0 border-r border-line bg-slate-50">
          <div className="flex flex-wrap items-center justify-between gap-3 border-b border-line bg-white px-3 py-2">
            <Tabs
              value={editorView}
              items={[
                { value: "edit", label: "编辑" },
                { value: "preview", label: "预览" },
                { value: "diff", label: "差异" },
              ]}
              onChange={setEditorView}
            />
            <div className="flex items-center gap-2 text-xs text-slate-500">
              <DensityToggle value={editorDensity} onChange={setEditorDensity} />
              {isDirty ? <Badge tone="amber">未保存</Badge> : <Badge tone="teal">已同步</Badge>}
              <span>{editorValue.trim().length > 0 ? `${editorValue.replace(/\s/g, "").length} 字` : "0 字"}</span>
              <Button size="sm" variant="ghost" onClick={() => clearEditorDraft(chapter.id)} disabled={!isDirty}>
                <RotateCcw className="h-4 w-4" />
                重置
              </Button>
            </div>
          </div>
          <div
            className={cn(
              "h-[calc(100vh-318px)] min-h-[520px] p-3",
              editorDensity === "compact" ? "chapter-editor-compact" : "chapter-editor-comfortable",
            )}
          >
            {editorView === "edit" ? (
              <CodeMirror
                value={editorValue}
                extensions={[markdown()]}
                basicSetup={{
                  lineNumbers: true,
                  foldGutter: true,
                  highlightActiveLine: true,
                }}
                onChange={(value) => setEditorDraft(chapter.id, value)}
              />
            ) : null}
            {editorView === "preview" ? <MarkdownPreview content={editorValue} /> : null}
            {editorView === "diff" ? <DraftDiffPanel savedContent={savedContent} draftContent={editorValue} /> : null}
          </div>
        </section>

        <aside className="min-w-0 bg-slate-50">
          <div className="sticky top-0 z-10 border-b border-line bg-white px-3 py-2">
            <Tabs value={rightPanel as RightPanel} items={panelTabs} onChange={(value) => setRightPanel(value)} />
          </div>
          <div className="max-h-[calc(100vh-120px)] overflow-y-auto">
            {rightPanel === "outline" ? <OutlinePanel chapter={chapter} outline={outline} /> : null}
            {rightPanel === "review" ? (
              <ReviewPanel
                report={reviewQuery.data}
                isLoading={reviewQuery.isLoading}
                continuityReport={continuityQuery.data}
                continuityLoading={continuityQuery.isLoading}
                onReview={() => reviewMutation.mutate()}
                onRewrite={() => rewriteMutation.mutate()}
                reviewPending={reviewMutation.isPending}
                rewritePending={rewriteMutation.isPending}
              />
            ) : null}
            {rightPanel === "quality" ? (
              <QualityPanel
                chapter={chapter}
                report={reviewQuery.data}
                reviewLoading={reviewQuery.isLoading}
                continuityReport={continuityQuery.data}
                continuityLoading={continuityQuery.isLoading}
                runs={runsQuery.data ?? []}
              />
            ) : null}
            {rightPanel === "agent" ? (
              <ChapterAgentPanel
                runs={runsQuery.data ?? []}
                progress={currentProgress}
                onClearProgress={chapter ? () => clearProgress(chapter.id) : undefined}
              />
            ) : null}
          </div>
        </aside>
      </div>

      <Section title="版本记录">
        <VersionList versions={versionsQuery.data ?? []} selectedVersion={selectedVersion?.version} onSelect={setSelectedVersion} />
        {selectedVersion ? (
          <div className="border-t border-line bg-white p-4">
            <div className="mb-2 flex items-center justify-between">
              <h3 className="text-sm font-semibold">v{selectedVersion.version} 正文</h3>
              <div className="flex items-center gap-2">
                <span className="text-xs text-slate-500">{formatDateTime(selectedVersion.created_at)}</span>
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() => {
                    setEditorDraft(chapter.id, selectedVersion.content);
                    setEditorView("diff");
                  }}
                >
                  套用到编辑器
                </Button>
              </div>
            </div>
            <pre className="max-h-72 overflow-auto whitespace-pre-wrap rounded-md border border-border bg-slate-50 p-3 text-sm leading-6 text-slate-700">
              {selectedVersion.content}
            </pre>
          </div>
        ) : null}
        <VersionDiffPanel versions={versionsQuery.data ?? []} />
      </Section>
    </div>
  );
}

function DensityToggle({ value, onChange }: { value: EditorDensity; onChange: (density: EditorDensity) => void }) {
  return (
    <div className="inline-flex h-8 items-center rounded-md border border-border bg-slate-50 p-1">
      {densityOptions.map((option) => (
        <button
          key={option.value}
          type="button"
          onClick={() => onChange(option.value)}
          className={cn(
            "h-6 rounded px-2 text-xs font-medium text-slate-600 transition",
            value === option.value && "bg-white text-ink shadow-soft",
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

function MobileChapterPicker({
  chapters,
  activeIndex,
  onChange,
}: {
  chapters: Chapter[];
  activeIndex: number;
  onChange: (chapterIndex: number) => void;
}) {
  if (chapters.length === 0) {
    return null;
  }

  return (
    <div className="border-b border-line bg-white px-4 py-3 lg:hidden">
      <label className="flex items-center gap-2 text-xs font-medium text-slate-600">
        章节
        <select value={activeIndex} onChange={(event) => onChange(Number(event.target.value))} className="input h-9 flex-1">
          {chapters.map((chapter) => (
            <option key={chapter.id} value={chapter.chapter_index}>
              {chapter.chapter_index}. {chapter.title}
            </option>
          ))}
        </select>
      </label>
    </div>
  );
}

function OutlinePanel({ chapter, outline }: { chapter: Chapter; outline?: ChapterOutline }) {
  return (
    <div className="space-y-4 p-3">
      <div className="rounded-md border border-border bg-white p-4 shadow-soft">
        <div className="mb-2 text-sm font-semibold">本章目标</div>
        <p className="text-sm leading-6 text-slate-700">{outline?.goal ?? chapter.outline}</p>
      </div>
      <div className="rounded-md border border-border bg-white p-4 shadow-soft">
        <div className="mb-2 text-sm font-semibold">冲突</div>
        <p className="text-sm leading-6 text-slate-700">{outline?.conflict ?? chapter.outline}</p>
      </div>
      <div className="rounded-md border border-border bg-white p-4 shadow-soft">
        <div className="mb-2 text-sm font-semibold">关键事件</div>
        <ul className="space-y-2 text-sm leading-6 text-slate-700">
          {(outline?.key_events ?? []).map((event) => (
            <li key={event}>{event}</li>
          ))}
        </ul>
      </div>
      <div className="rounded-md border border-border bg-white p-4 shadow-soft">
        <div className="mb-2 text-sm font-semibold">章尾钩子</div>
        <p className="text-sm leading-6 text-slate-700">{outline?.cliffhanger ?? "-"}</p>
      </div>
    </div>
  );
}

function ChapterAgentPanel({
  runs,
  progress,
  onClearProgress,
}: {
  runs: Awaited<ReturnType<typeof api.getAgentRuns>>;
  progress?: ChapterProgress;
  onClearProgress?: () => void;
}) {
  return (
    <div className="space-y-3 p-3">
      <OperationTimeline progress={progress} onClear={onClearProgress} />
      {runs.slice(0, 8).map((run) => (
        <div key={run.id} className="rounded-md border border-border bg-white p-3 shadow-soft">
          <div className="mb-2 flex items-start justify-between gap-2">
            <div>
              <div className="text-sm font-semibold">{agentRoleLabels[run.role]}</div>
              <div className="text-xs text-slate-500">{agentTaskLabels[run.task]}</div>
            </div>
            <Badge tone={run.status === "ok" ? "teal" : "rose"}>{run.status}</Badge>
          </div>
          <p className="text-xs leading-5 text-slate-600">{run.output_summary}</p>
          <div className="mt-3 flex justify-between text-xs text-slate-500">
            <span>{formatDuration(run.duration_ms)}</span>
            <span>{formatDateTime(run.created_at)}</span>
          </div>
        </div>
      ))}
    </div>
  );
}

function writeMutationLabel(
  writing: boolean,
  reviewing: boolean,
  rewriting: boolean,
  saving: boolean,
): { title: string; description: string } | null {
  if (writing) {
    return { title: "正在生成章节", description: "Writer、Continuity 和 Style 输出完成后会写入新版本。" };
  }
  if (reviewing) {
    return { title: "正在审稿", description: "Reviewer 会给出评分、问题列表和返工建议。" };
  }
  if (rewriting) {
    return { title: "正在重写", description: "系统会按当前审稿意见生成新的章节版本。" };
  }
  if (saving) {
    return { title: "正在保存人工编辑稿", description: "保存后会进入版本记录，可继续审稿或导出。" };
  }
  return null;
}

function mutationErrorMessage(error: unknown): string | null {
  if (!error) {
    return null;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "未知错误";
}

interface MutationProgressContext {
  chapterId: string;
  progressId: string;
}

function finishMutationProgress(
  context: MutationProgressContext | undefined,
  finishProgress: ReturnType<typeof useWorkspaceStore.getState>["finishProgress"],
  status: "success" | "error",
  detail: string,
) {
  if (!context) {
    return;
  }
  finishProgress(context.chapterId, context.progressId, status, detail);
}

function scheduleOperationProgress(
  chapterId: string,
  progressId: string,
  kind: ProgressKind,
  novelId: string,
  chapterNumber: number,
  appendProgressStep: ReturnType<typeof useWorkspaceStore.getState>["appendProgressStep"],
) {
  const operation = progressKindToOperation[kind];
  const realStreamSupported = operation === "write" || operation === "rewrite";
  if (!operation || (!apiConfig.useMock && (!apiConfig.sseEnabled || !realStreamSupported))) {
    localProgressPlans[kind].forEach((step, index) => {
      window.setTimeout(() => {
        appendProgressStep(chapterId, progressId, {
          label: step.label,
          detail: step.detail,
          status: "running",
        });
      }, step.delayMs + index * 60);
    });
    return;
  }

  void api.streamChapterOperation(
    novelId,
    chapterNumber,
    operation,
    (event) => {
      appendProgressStep(chapterId, progressId, streamEventToProgressStep(event));
    },
  ).catch((error: unknown) => {
    appendProgressStep(chapterId, progressId, {
      label: "SSE",
      detail: mutationErrorMessage(error) ?? "流式事件读取失败，等待普通响应返回。",
      status: "error",
    });
  });
}

function streamEventToProgressStep(event: ChapterStreamEvent): { label: string; detail: string; status: "running" | "success" | "error" } {
  return {
    label: event.role ? agentRoleLabels[event.role] : streamEventLabels[event.event],
    detail: event.message,
    status: event.event === "error" ? "error" : event.event === "completed" ? "success" : "running",
  };
}

const progressKindToOperation: Partial<Record<ProgressKind, ChapterOperation>> = {
  write: "write",
  review: "review",
  rewrite: "rewrite",
};

const streamEventLabels: Record<ChapterStreamEvent["event"], string> = {
  queued: "排队",
  started: "开始",
  chapter_chunk: "正文片段",
  agent_started: "Agent 启动",
  agent_delta: "增量输出",
  agent_completed: "Agent 完成",
  artifact_saved: "保存产物",
  completed: "完成",
  error: "失败",
};

const localProgressPlans: Record<ProgressKind, Array<{ delayMs: number; label: string; detail: string }>> = {
  write: [
    { delayMs: 220, label: "Writer", detail: "根据章节大纲生成正文草稿。" },
    { delayMs: 520, label: "Continuity", detail: "检查人物、事实和伏笔是否连续。" },
    { delayMs: 760, label: "Style", detail: "压缩说明段，保留章节节奏。" },
  ],
  review: [
    { delayMs: 180, label: "Reviewer", detail: "按开篇、节奏、回报和章尾钩子评分。" },
    { delayMs: 480, label: "RewriteDecision", detail: "整理问题列表、建议和返工目标。" },
  ],
  rewrite: [
    { delayMs: 180, label: "RewriteInstruction", detail: "读取最近审稿意见并锁定返工范围。" },
    { delayMs: 460, label: "Writer", detail: "保留核心事实，重写薄弱段落。" },
    { delayMs: 720, label: "Style", detail: "统一重写稿和原章节语气。" },
  ],
  save: [
    { delayMs: 160, label: "ChapterVersion", detail: "保存人工编辑稿版本快照。" },
    { delayMs: 320, label: "Chapter", detail: "更新章节最新正文、字数和更新时间。" },
  ],
};

const densityOptions: Array<{ value: EditorDensity; label: string }> = [
  { value: "comfortable", label: "舒适" },
  { value: "compact", label: "紧凑" },
];
