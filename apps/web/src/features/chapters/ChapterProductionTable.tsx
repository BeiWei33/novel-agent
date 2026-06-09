import { useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { Clock3, ExternalLink, Play, PlusSquare, Search, Square, Wand2, X, SearchCheck } from "lucide-react";
import type { Chapter, ChapterStatus } from "../../types/domain";
import { chapterStatusLabels, clampScore, formatDateTime } from "../../lib/format";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";

type ProductionFilter = "all" | ChapterStatus | "ungenerated" | "low_score" | "needs_review";

export function ChapterProductionTable({
  novelId,
  chapters,
  selectedIndexes,
  activeChapterIndex,
  isGenerating,
  isReviewing,
  isRewriting,
  isQueueing,
  onToggleChapter,
  onSetSelection,
  onSelectNext,
  onSelectAllDraftable,
  onClearSelection,
  onGenerateOne,
  onGenerateSelected,
  onReviewOne,
  onReviewSelected,
  onRewriteOne,
  onRewriteSelected,
  onQueueGenerateSelected,
  onQueueReviewSelected,
  onQueueRewriteSelected,
}: {
  novelId: string;
  chapters: Chapter[];
  selectedIndexes: Set<number>;
  activeChapterIndex?: number;
  isGenerating: boolean;
  isReviewing: boolean;
  isRewriting: boolean;
  isQueueing: boolean;
  onToggleChapter: (chapterIndex: number) => void;
  onSetSelection: (chapterIndexes: number[]) => void;
  onSelectNext: () => void;
  onSelectAllDraftable: () => void;
  onClearSelection: () => void;
  onGenerateOne: (chapterIndex: number) => void;
  onGenerateSelected: () => void;
  onReviewOne: (chapterIndex: number) => void;
  onReviewSelected: () => void;
  onRewriteOne: (chapterIndex: number) => void;
  onRewriteSelected: () => void;
  onQueueGenerateSelected: () => void;
  onQueueReviewSelected: () => void;
  onQueueRewriteSelected: () => void;
}) {
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState<ProductionFilter>("all");
  const selectedCount = selectedIndexes.size;
  const draftedCount = chapters.filter((chapter) => chapter.content).length;
  const reviewedCount = chapters.filter((chapter) => chapter.status === "reviewed" || chapter.status === "final").length;
  const rewriteCount = chapters.filter((chapter) => chapter.status === "rewrite_needed").length;
  const lowScoreCount = chapters.filter((chapter) => typeof chapter.score === "number" && chapter.score < 75).length;
  const ungeneratedCount = chapters.filter((chapter) => !chapter.content).length;
  const needsReviewCount = chapters.filter(needsReview).length;
  const busy = isGenerating || isReviewing || isRewriting || isQueueing;
  const filteredChapters = useMemo(
    () =>
      chapters.filter((chapter) => {
        const keyword = search.trim().toLowerCase();
        const matchesSearch =
          keyword.length === 0 ||
          chapter.title.toLowerCase().includes(keyword) ||
          chapter.outline.toLowerCase().includes(keyword) ||
          String(chapter.chapter_index).includes(keyword);
        if (!matchesSearch) {
          return false;
        }
        if (filter === "all") {
          return true;
        }
        if (filter === "ungenerated") {
          return !chapter.content;
        }
        if (filter === "low_score") {
          return typeof chapter.score === "number" && chapter.score < 75;
        }
        if (filter === "needs_review") {
          return needsReview(chapter);
        }
        return chapter.status === filter;
      }),
    [chapters, filter, search],
  );

  return (
    <section className="border-t border-line bg-white">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-line px-4 py-3">
        <div>
          <h2 className="text-sm font-semibold text-ink">章节生产</h2>
          <div className="mt-1 flex flex-wrap gap-2 text-xs text-slate-500">
            <span>{chapters.length} 章</span>
            <span>{draftedCount} 章有正文</span>
            <span>{reviewedCount} 章已审稿</span>
            <span>{needsReviewCount} 章待审</span>
            <span>{rewriteCount} 章需返工</span>
            <span>{lowScoreCount} 章低分</span>
            <span>选中 {selectedCount} 章</span>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="secondary" onClick={onSelectNext} disabled={busy}>
            <PlusSquare className="h-4 w-4" />
            选择后 3 章
          </Button>
          <Button size="sm" variant="secondary" onClick={onSelectAllDraftable} disabled={busy}>
            <Square className="h-4 w-4" />
            全选未生成
          </Button>
          <Button
            size="sm"
            variant="secondary"
            onClick={() => onSetSelection(filteredChapters.map((chapter) => chapter.chapter_index))}
            disabled={filteredChapters.length === 0 || busy}
          >
            <Square className="h-4 w-4" />
            选择当前
          </Button>
          <Button size="sm" variant="ghost" onClick={onClearSelection} disabled={selectedCount === 0 || busy}>
            <X className="h-4 w-4" />
            清空
          </Button>
          <Button size="sm" variant="secondary" onClick={onReviewSelected} disabled={selectedCount === 0 || busy}>
            <SearchCheck className="h-4 w-4" />
            {isReviewing ? "审稿中" : "审稿选中"}
          </Button>
          <Button size="sm" variant="secondary" onClick={onRewriteSelected} disabled={selectedCount === 0 || busy}>
            <Wand2 className="h-4 w-4" />
            {isRewriting ? "返工中" : "返工选中"}
          </Button>
          <Button size="sm" variant="primary" onClick={onGenerateSelected} disabled={selectedCount === 0 || busy}>
            <Play className="h-4 w-4" />
            {isGenerating ? "生成中" : "生成选中"}
          </Button>
          <Button size="sm" variant="secondary" onClick={onQueueReviewSelected} disabled={selectedCount === 0 || busy}>
            <Clock3 className="h-4 w-4" />
            后台审稿
          </Button>
          <Button size="sm" variant="secondary" onClick={onQueueRewriteSelected} disabled={selectedCount === 0 || busy}>
            <Clock3 className="h-4 w-4" />
            后台返工
          </Button>
          <Button size="sm" variant="secondary" onClick={onQueueGenerateSelected} disabled={selectedCount === 0 || busy}>
            <Clock3 className="h-4 w-4" />
            {isQueueing ? "提交中" : "后台生成"}
          </Button>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-3 border-b border-line px-4 py-3">
        <label className="relative min-w-56 flex-1 md:max-w-md">
          <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            className="input h-9 pl-9"
            placeholder="搜索章节、标题或大纲"
          />
        </label>
        <select value={filter} onChange={(event) => setFilter(event.target.value as ProductionFilter)} className="input h-9 w-40">
          <option value="all">全部章节</option>
          <option value="ungenerated">未生成</option>
          <option value="needs_review">待审</option>
          <option value="outlined">有大纲</option>
          <option value="drafted">已起草</option>
          <option value="reviewed">已审稿</option>
          <option value="rewrite_needed">需返工</option>
          <option value="final">定稿</option>
          <option value="low_score">低分</option>
        </select>
        <div className="flex flex-wrap gap-2">
          <QuickFilter active={filter === "ungenerated"} label={`未生成 ${ungeneratedCount}`} onClick={() => setFilter("ungenerated")} />
          <QuickFilter active={filter === "needs_review"} label={`待审 ${needsReviewCount}`} onClick={() => setFilter("needs_review")} />
          <QuickFilter active={filter === "rewrite_needed"} label={`返工 ${rewriteCount}`} onClick={() => setFilter("rewrite_needed")} />
          <QuickFilter active={filter === "low_score"} label={`低分 ${lowScoreCount}`} onClick={() => setFilter("low_score")} />
          {(filter !== "all" || search) && (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => {
                setSearch("");
                setFilter("all");
              }}
            >
              清除筛选
            </Button>
          )}
        </div>
      </div>

      <div className="overflow-x-auto">
        <table className="min-w-[980px] w-full border-collapse text-sm">
          <thead className="table-head">
            <tr>
              <th className="w-12 px-4 py-3">选</th>
              <th className="px-3 py-3">章节</th>
              <th className="px-3 py-3">状态</th>
              <th className="px-3 py-3 text-right">字数</th>
              <th className="px-3 py-3 text-right">评分</th>
              <th className="px-3 py-3 text-right">版本</th>
              <th className="px-3 py-3">更新时间</th>
              <th className="px-4 py-3 text-right">操作</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-line">
            {filteredChapters.map((chapter) => {
              const selected = selectedIndexes.has(chapter.chapter_index);
              const active = activeChapterIndex === chapter.chapter_index;
              return (
                <tr key={chapter.id} className={active ? "bg-sky-50" : "hover:bg-slate-50"}>
                  <td className="px-4 py-3">
                    <input
                      type="checkbox"
                      checked={selected}
                      disabled={busy}
                      onChange={() => onToggleChapter(chapter.chapter_index)}
                      className="h-4 w-4 rounded border-border text-accent"
                      aria-label={`选择第 ${chapter.chapter_index} 章`}
                    />
                  </td>
                  <td className="px-3 py-3">
                    <div className="font-medium text-ink">
                      {chapter.chapter_index}. {chapter.title}
                    </div>
                    <div className="mt-1 line-clamp-1 text-xs text-slate-500">{chapter.outline}</div>
                  </td>
                  <td className="px-3 py-3">
                    <Badge tone={statusTone(chapter)}>{chapterStatusLabels[chapter.status]}</Badge>
                  </td>
                  <td className="px-3 py-3 text-right tabular-nums text-slate-700">{chapter.word_count}</td>
                  <td className="px-3 py-3 text-right tabular-nums text-slate-700">{clampScore(chapter.score)}</td>
                  <td className="px-3 py-3 text-right tabular-nums text-slate-700">v{chapter.version}</td>
                  <td className="px-3 py-3 text-slate-500">{formatDateTime(chapter.updated_at)}</td>
                  <td className="px-4 py-3">
                    <div className="flex justify-end gap-2">
                      <Button size="sm" variant="ghost" onClick={() => onReviewOne(chapter.chapter_index)} disabled={busy || !chapter.content}>
                        <SearchCheck className="h-4 w-4" />
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => onRewriteOne(chapter.chapter_index)}
                        disabled={busy || !canRewrite(chapter)}
                      >
                        <Wand2 className="h-4 w-4" />
                      </Button>
                      <Button size="sm" variant="ghost" onClick={() => onGenerateOne(chapter.chapter_index)} disabled={busy}>
                        <Play className="h-4 w-4" />
                      </Button>
                      <Link to={`/novels/${novelId}/chapters/${chapter.chapter_index}`}>
                        <Button size="sm" variant="secondary">
                          <ExternalLink className="h-4 w-4" />
                          打开
                        </Button>
                      </Link>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
        {filteredChapters.length === 0 ? (
          <div className="border-t border-line px-4 py-10 text-center text-sm text-slate-500">没有匹配的章节</div>
        ) : null}
      </div>
    </section>
  );
}

function QuickFilter({ active, label, onClick }: { active: boolean; label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`h-8 rounded-md border px-2.5 text-xs font-medium transition ${
        active ? "border-accent bg-teal-50 text-teal-700" : "border-border bg-white text-slate-600 hover:bg-slate-50"
      }`}
    >
      {label}
    </button>
  );
}

function needsReview(chapter: Chapter): boolean {
  return Boolean(chapter.content) && chapter.status !== "reviewed" && chapter.status !== "final";
}

function canRewrite(chapter: Chapter): boolean {
  return Boolean(chapter.content) && (chapter.status === "rewrite_needed" || (typeof chapter.score === "number" && chapter.score < 75));
}

function statusTone(chapter: Chapter): "slate" | "teal" | "amber" | "rose" | "blue" {
  if (chapter.status === "rewrite_needed") {
    return "rose";
  }
  if (chapter.status === "reviewed" || chapter.status === "final") {
    return "teal";
  }
  if (chapter.status === "drafted") {
    return "blue";
  }
  return "slate";
}
