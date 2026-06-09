import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { BookOpen, Download } from "lucide-react";
import type { ApiJobKind } from "../types/domain";
import { api, queryKeys } from "../lib/api";
import { formatDateTime, novelStatusLabels, platformLabels } from "../lib/format";
import { BibleTabs } from "../features/novels/BibleTabs";
import { ChapterProductionTable } from "../features/chapters/ChapterProductionTable";
import { PageHeader } from "../components/PageHeader";
import { LoadingState } from "../components/LoadingState";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { StatusBanner } from "../components/StatusBanner";

export function NovelWorkspacePage() {
  const { novelId = "" } = useParams();
  const queryClient = useQueryClient();
  const [selectedChapters, setSelectedChapters] = useState<Set<number>>(() => new Set());
  const [batchProgress, setBatchProgress] = useState<{ completed: number; total: number; chapterIndex?: number } | null>(null);
  const [reviewProgress, setReviewProgress] = useState<{ completed: number; total: number; chapterIndex?: number } | null>(null);
  const [rewriteProgress, setRewriteProgress] = useState<{ completed: number; total: number; chapterIndex?: number } | null>(null);
  const [jobProgress, setJobProgress] = useState<{
    kind: ChapterJobKind;
    completed: number;
    total: number;
    chapterIndex?: number;
  } | null>(null);
  const [queuedJobCount, setQueuedJobCount] = useState(0);
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
  const exportMutation = useMutation({
    mutationFn: () => api.exportMarkdown(novelId),
    onSuccess: (content) => {
      const title = detailQuery.data?.novel.title ?? "novel";
      const blob = new Blob([content], { type: "text/markdown;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `${title}.md`;
      anchor.click();
      URL.revokeObjectURL(url);
    },
  });
  const batchGenerateMutation = useMutation({
    mutationFn: (chapterIndexes: number[]) =>
      api.writeChapters(novelId, chapterIndexes, (completed, total, chapterIndex) => {
        setBatchProgress({ completed, total, chapterIndex });
      }),
    onMutate: (chapterIndexes) => {
      setBatchProgress({ completed: 0, total: chapterIndexes.length });
    },
    onSuccess: async () => {
      setSelectedChapters(new Set());
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: queryKeys.novels }),
        queryClient.invalidateQueries({ queryKey: queryKeys.novel(novelId) }),
        queryClient.invalidateQueries({ queryKey: queryKeys.chapters(novelId) }),
        queryClient.invalidateQueries({ queryKey: queryKeys.agentRunsRoot }),
      ]);
    },
    onSettled: () => {
      window.setTimeout(() => setBatchProgress(null), 1400);
    },
  });
  const batchReviewMutation = useMutation({
    mutationFn: (chapterIndexes: number[]) =>
      api.reviewChapters(novelId, chapterIndexes, (completed, total, chapterIndex) => {
        setReviewProgress({ completed, total, chapterIndex });
      }),
    onMutate: (chapterIndexes) => {
      setReviewProgress({ completed: 0, total: chapterIndexes.length });
    },
    onSuccess: async () => {
      setSelectedChapters(new Set());
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: queryKeys.novels }),
        queryClient.invalidateQueries({ queryKey: queryKeys.novel(novelId) }),
        queryClient.invalidateQueries({ queryKey: queryKeys.chapters(novelId) }),
        queryClient.invalidateQueries({ queryKey: queryKeys.agentRunsRoot }),
      ]);
    },
    onSettled: () => {
      window.setTimeout(() => setReviewProgress(null), 1400);
    },
  });
  const batchRewriteMutation = useMutation({
    mutationFn: (chapterIndexes: number[]) =>
      api.rewriteChapters(novelId, chapterIndexes, (completed, total, chapterIndex) => {
        setRewriteProgress({ completed, total, chapterIndex });
      }),
    onMutate: (chapterIndexes) => {
      setRewriteProgress({ completed: 0, total: chapterIndexes.length });
    },
    onSuccess: async () => {
      setSelectedChapters(new Set());
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: queryKeys.novels }),
        queryClient.invalidateQueries({ queryKey: queryKeys.novel(novelId) }),
        queryClient.invalidateQueries({ queryKey: queryKeys.chapters(novelId) }),
        queryClient.invalidateQueries({ queryKey: queryKeys.agentRunsRoot }),
      ]);
    },
    onSettled: () => {
      window.setTimeout(() => setRewriteProgress(null), 1400);
    },
  });
  const batchJobMutation = useMutation({
    mutationFn: ({ kind, chapterIndexes }: { kind: ChapterJobKind; chapterIndexes: number[] }) =>
      api.createChapterJobs(novelId, chapterIndexes, kind, (completed, total, chapterIndex) => {
        setJobProgress({ kind, completed, total, chapterIndex });
      }),
    onMutate: ({ kind, chapterIndexes }) => {
      setQueuedJobCount(0);
      setJobProgress({ kind, completed: 0, total: chapterIndexes.length });
    },
    onSuccess: async (jobs) => {
      setQueuedJobCount(jobs.length);
      setSelectedChapters(new Set());
      await queryClient.invalidateQueries({ queryKey: queryKeys.jobsRoot });
    },
    onSettled: () => {
      window.setTimeout(() => setJobProgress(null), 1800);
    },
  });

  if (detailQuery.isLoading) {
    return <LoadingState label="读取小说工作台" />;
  }

  if (!detailQuery.data) {
    return <div className="p-4 text-sm text-danger">小说不存在</div>;
  }

  const { novel } = detailQuery.data;
  const chapters = chaptersQuery.data ?? [];
  const firstChapter = chapters[0];
  const selectedIndexes = selectedChapters;

  function toggleChapter(chapterIndex: number) {
    setSelectedChapters((current) => {
      const next = new Set(current);
      if (next.has(chapterIndex)) {
        next.delete(chapterIndex);
      } else {
        next.add(chapterIndex);
      }
      return next;
    });
  }

  function selectNextDraftable() {
    const next = chapters
      .filter((chapter) => !chapter.content || chapter.status === "outlined")
      .slice(0, 3)
      .map((chapter) => chapter.chapter_index);
    setSelectedChapters(new Set(next));
  }

  function selectAllDraftable() {
    setSelectedChapters(new Set(chapters.filter((chapter) => !chapter.content).map((chapter) => chapter.chapter_index)));
  }

  function generateSelected() {
    const indexes = [...selectedChapters].sort((a, b) => a - b);
    if (indexes.length > 0) {
      batchGenerateMutation.mutate(indexes);
    }
  }

  function reviewSelected() {
    const indexes = [...selectedChapters].sort((a, b) => a - b);
    const reviewableIndexes = indexes.filter((chapterIndex) => {
      const chapter = chapters.find((item) => item.chapter_index === chapterIndex);
      return chapter?.content;
    });
    if (reviewableIndexes.length > 0) {
      batchReviewMutation.mutate(reviewableIndexes);
    }
  }

  function rewriteSelected() {
    const indexes = [...selectedChapters].sort((a, b) => a - b);
    const rewriteableIndexes = indexes.filter((chapterIndex) => {
      const chapter = chapters.find((item) => item.chapter_index === chapterIndex);
      return Boolean(chapter?.content) && (chapter?.status === "rewrite_needed" || (typeof chapter?.score === "number" && chapter.score < 75));
    });
    if (rewriteableIndexes.length > 0) {
      batchRewriteMutation.mutate(rewriteableIndexes);
    }
  }

  function queueSelected(kind: ChapterJobKind) {
    const indexes = [...selectedChapters].sort((a, b) => a - b);
    const eligibleIndexes = indexes.filter((chapterIndex) => {
      const chapter = chapters.find((item) => item.chapter_index === chapterIndex);
      if (!chapter) {
        return false;
      }
      if (kind === "write_chapter") {
        return true;
      }
      if (kind === "review_chapter") {
        return Boolean(chapter.content);
      }
      return Boolean(chapter.content) && (chapter.status === "rewrite_needed" || (typeof chapter.score === "number" && chapter.score < 75));
    });
    if (eligibleIndexes.length > 0) {
      batchJobMutation.mutate({ kind, chapterIndexes: eligibleIndexes });
    }
  }

  return (
    <div>
      <PageHeader
        title={novel.title}
        meta={
          <>
            <Badge tone="blue">{platformLabels[novel.target_platform]}</Badge>
            <Badge tone={novel.status === "active" ? "teal" : "slate"}>{novelStatusLabels[novel.status]}</Badge>
            <span>{novel.genre}</span>
            <span>更新 {formatDateTime(novel.updated_at)}</span>
          </>
        }
        actions={
          <>
            {firstChapter ? (
              <Link to={`/novels/${novel.id}/chapters/${firstChapter.chapter_index}`}>
                <Button variant="primary">
                  <BookOpen className="h-4 w-4" />
                  打开第 1 章
                </Button>
              </Link>
            ) : null}
            <Button variant="secondary" onClick={() => exportMutation.mutate()} disabled={exportMutation.isPending}>
              <Download className="h-4 w-4" />
              导出
            </Button>
          </>
        }
      />
      {exportMutation.isPending ? <StatusBanner title="正在导出 Markdown">正在合并已生成章节正文。</StatusBanner> : null}
      {exportMutation.isError ? (
        <StatusBanner tone="danger" title="导出失败">
          {exportMutation.error instanceof Error ? exportMutation.error.message : "请稍后重试。"}
        </StatusBanner>
      ) : null}
      {batchGenerateMutation.isPending && batchProgress ? (
        <StatusBanner title="正在批量生成章节">
          已完成 {batchProgress.completed} / {batchProgress.total}
          {batchProgress.chapterIndex ? `，刚完成第 ${batchProgress.chapterIndex} 章` : ""}
        </StatusBanner>
      ) : null}
      {batchGenerateMutation.isSuccess && batchProgress ? (
        <StatusBanner tone="success" title="批量生成完成">
          已完成 {batchProgress.completed} / {batchProgress.total} 章。
        </StatusBanner>
      ) : null}
      {batchGenerateMutation.isError ? (
        <StatusBanner tone="danger" title="批量生成失败">
          {batchGenerateMutation.error instanceof Error ? batchGenerateMutation.error.message : "请稍后重试。"}
        </StatusBanner>
      ) : null}
      {batchReviewMutation.isPending && reviewProgress ? (
        <StatusBanner title="正在批量审稿">
          已完成 {reviewProgress.completed} / {reviewProgress.total}
          {reviewProgress.chapterIndex ? `，刚完成第 ${reviewProgress.chapterIndex} 章` : ""}
        </StatusBanner>
      ) : null}
      {batchReviewMutation.isSuccess && reviewProgress ? (
        <StatusBanner tone="success" title="批量审稿完成">
          已完成 {reviewProgress.completed} / {reviewProgress.total} 章。
        </StatusBanner>
      ) : null}
      {batchReviewMutation.isError ? (
        <StatusBanner tone="danger" title="批量审稿失败">
          {batchReviewMutation.error instanceof Error ? batchReviewMutation.error.message : "请稍后重试。"}
        </StatusBanner>
      ) : null}
      {batchRewriteMutation.isPending && rewriteProgress ? (
        <StatusBanner title="正在批量返工">
          已完成 {rewriteProgress.completed} / {rewriteProgress.total}
          {rewriteProgress.chapterIndex ? `，刚完成第 ${rewriteProgress.chapterIndex} 章` : ""}
        </StatusBanner>
      ) : null}
      {batchRewriteMutation.isSuccess && rewriteProgress ? (
        <StatusBanner tone="success" title="批量返工完成">
          已完成 {rewriteProgress.completed} / {rewriteProgress.total} 章。
        </StatusBanner>
      ) : null}
      {batchRewriteMutation.isError ? (
        <StatusBanner tone="danger" title="批量返工失败">
          {batchRewriteMutation.error instanceof Error ? batchRewriteMutation.error.message : "请稍后重试。"}
        </StatusBanner>
      ) : null}
      {batchJobMutation.isPending && jobProgress ? (
        <StatusBanner title={`正在提交${jobKindLabel(jobProgress.kind)}任务`}>
          已提交 {jobProgress.completed} / {jobProgress.total}
          {jobProgress.chapterIndex ? `，刚提交第 ${jobProgress.chapterIndex} 章` : ""}
        </StatusBanner>
      ) : null}
      {batchJobMutation.isSuccess && jobProgress ? (
        <StatusBanner tone="success" title="后台任务已提交">
          已创建 {queuedJobCount} 个后台任务，可在任务队列查看。
        </StatusBanner>
      ) : null}
      {batchJobMutation.isError ? (
        <StatusBanner tone="danger" title="后台任务提交失败">
          {batchJobMutation.error instanceof Error ? batchJobMutation.error.message : "请稍后重试。"}
        </StatusBanner>
      ) : null}
      <BibleTabs detail={detailQuery.data} />
      <ChapterProductionTable
        novelId={novel.id}
        chapters={chapters}
        selectedIndexes={selectedIndexes}
        activeChapterIndex={batchProgress?.chapterIndex ?? reviewProgress?.chapterIndex ?? rewriteProgress?.chapterIndex}
        isGenerating={batchGenerateMutation.isPending}
        isReviewing={batchReviewMutation.isPending}
        isRewriting={batchRewriteMutation.isPending}
        isQueueing={batchJobMutation.isPending}
        onToggleChapter={toggleChapter}
        onSetSelection={(chapterIndexes) => setSelectedChapters(new Set(chapterIndexes))}
        onSelectNext={selectNextDraftable}
        onSelectAllDraftable={selectAllDraftable}
        onClearSelection={() => setSelectedChapters(new Set())}
        onGenerateOne={(chapterIndex) => batchGenerateMutation.mutate([chapterIndex])}
        onGenerateSelected={generateSelected}
        onReviewOne={(chapterIndex) => batchReviewMutation.mutate([chapterIndex])}
        onReviewSelected={reviewSelected}
        onRewriteOne={(chapterIndex) => batchRewriteMutation.mutate([chapterIndex])}
        onRewriteSelected={rewriteSelected}
        onQueueGenerateSelected={() => queueSelected("write_chapter")}
        onQueueReviewSelected={() => queueSelected("review_chapter")}
        onQueueRewriteSelected={() => queueSelected("rewrite_chapter")}
      />
    </div>
  );
}

type ChapterJobKind = Extract<ApiJobKind, "write_chapter" | "review_chapter" | "rewrite_chapter">;

function jobKindLabel(kind: ChapterJobKind): string {
  if (kind === "review_chapter") {
    return "审稿";
  }
  if (kind === "rewrite_chapter") {
    return "返工";
  }
  return "生成";
}
