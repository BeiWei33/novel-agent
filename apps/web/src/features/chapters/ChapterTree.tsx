import { Link } from "react-router-dom";
import type { Chapter } from "../../types/domain";
import { cn } from "../../lib/cn";
import { chapterStatusLabels, clampScore } from "../../lib/format";
import { Badge } from "../../components/ui/Badge";

export function ChapterTree({ novelId, chapters, activeIndex }: { novelId: string; chapters: Chapter[]; activeIndex: number }) {
  return (
    <nav className="h-full overflow-y-auto border-r border-line bg-white">
      <div className="sticky top-0 z-10 flex h-11 items-center border-b border-line bg-white px-3 text-sm font-semibold">
        章节树
      </div>
      <div className="divide-y divide-line">
        {chapters.map((chapter) => (
          <Link
            key={chapter.id}
            to={`/novels/${novelId}/chapters/${chapter.chapter_index}`}
            className={cn(
              "block px-3 py-2.5 transition hover:bg-slate-50",
              chapter.chapter_index === activeIndex && "bg-teal-50/80",
            )}
          >
            <div className="flex items-start justify-between gap-2">
              <div className="min-w-0">
                <div className="truncate text-sm font-medium text-ink">
                  {chapter.chapter_index}. {chapter.title}
                </div>
                <div className="mt-1 flex items-center gap-2 text-xs text-slate-500">
                  <span>{chapterStatusLabels[chapter.status]}</span>
                  <span>{chapter.word_count} 字</span>
                </div>
              </div>
              <Badge tone={chapter.status === "rewrite_needed" ? "rose" : chapter.score ? "teal" : "slate"} className="shrink-0">
                {clampScore(chapter.score)}
              </Badge>
            </div>
          </Link>
        ))}
      </div>
    </nav>
  );
}
