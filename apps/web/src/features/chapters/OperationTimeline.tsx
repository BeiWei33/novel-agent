import { CheckCircle2, Circle, Loader2, XCircle } from "lucide-react";
import type { ChapterProgress } from "../../lib/store";
import { formatDateTime } from "../../lib/format";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";

export function OperationTimeline({ progress, onClear }: { progress?: ChapterProgress; onClear?: () => void }) {
  if (!progress) {
    return (
      <div className="rounded-md border border-dashed border-border bg-white p-4 text-sm leading-6 text-slate-500">
        当前没有进行中的章节任务。
      </div>
    );
  }

  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-4 flex items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-2">
            <h3 className="text-sm font-semibold text-ink">{kindLabels[progress.kind]}</h3>
            <Badge tone={progress.status === "running" ? "blue" : progress.status === "success" ? "teal" : "rose"}>
              {progress.status}
            </Badge>
          </div>
          <p className="mt-1 text-xs text-slate-500">{formatDateTime(progress.startedAt)}</p>
        </div>
        {progress.status !== "running" && onClear ? (
          <Button size="sm" variant="ghost" onClick={onClear}>
            清除
          </Button>
        ) : null}
      </div>
      <ol className="space-y-3">
        {progress.steps.map((step) => {
          const Icon = step.status === "running" ? Loader2 : step.status === "success" ? CheckCircle2 : step.status === "error" ? XCircle : Circle;
          return (
            <li key={step.id} className="grid grid-cols-[24px_minmax(0,1fr)] gap-3">
              <Icon
                className={
                  step.status === "running"
                    ? "mt-0.5 h-4 w-4 animate-spin text-sky-600"
                    : step.status === "success"
                      ? "mt-0.5 h-4 w-4 text-teal-600"
                      : step.status === "error"
                        ? "mt-0.5 h-4 w-4 text-rose-600"
                        : "mt-0.5 h-4 w-4 text-slate-400"
                }
              />
              <div>
                <div className="flex flex-wrap items-center gap-2">
                  <span className="text-sm font-medium text-ink">{step.label}</span>
                  <span className="text-xs text-slate-400">{formatDateTime(step.timestamp)}</span>
                </div>
                <p className="mt-1 text-xs leading-5 text-slate-600">{step.detail}</p>
              </div>
            </li>
          );
        })}
      </ol>
    </div>
  );
}

const kindLabels: Record<ChapterProgress["kind"], string> = {
  write: "章节生成进度",
  review: "审稿进度",
  rewrite: "重写进度",
  save: "保存进度",
};
