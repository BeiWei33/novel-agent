import { Ban, RotateCcw } from "lucide-react";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { cn } from "../../lib/cn";
import { formatDateTime, jobKindLabels, jobStatusLabels } from "../../lib/format";
import type { ApiJob, ApiJobStatus } from "../../types/domain";

export function JobTable({
  jobs,
  selectedJobId,
  retryingJobId,
  cancelingJobId,
  onSelectJob,
  onRetryJob,
  onCancelJob,
}: {
  jobs: ApiJob[];
  selectedJobId?: string;
  retryingJobId?: string;
  cancelingJobId?: string;
  onSelectJob?: (job: ApiJob) => void;
  onRetryJob?: (job: ApiJob) => void;
  onCancelJob?: (job: ApiJob) => void;
}) {
  return (
    <div className="overflow-x-auto bg-white">
      <table className="min-w-[980px] w-full border-collapse text-sm">
        <thead className="table-head">
          <tr>
            <th className="px-4 py-3">更新时间</th>
            <th className="px-3 py-3">任务</th>
            <th className="px-3 py-3">状态</th>
            <th className="px-3 py-3">进度</th>
            <th className="px-3 py-3">作品</th>
            <th className="px-3 py-3 text-right">章节</th>
            <th className="px-3 py-3">错误</th>
            <th className="px-4 py-3 text-right">操作</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-line">
          {jobs.map((job) => (
            <tr
              key={job.id}
              className={cn(
                "cursor-pointer hover:bg-slate-50",
                selectedJobId === job.id && "bg-teal-50/70 outline outline-1 outline-accent",
              )}
              onClick={() => onSelectJob?.(job)}
            >
              <td className="px-4 py-3 text-slate-500">{formatDateTime(job.updated_at)}</td>
              <td className="px-3 py-3">
                <div className="font-medium">{jobKindLabels[job.kind]}</div>
                <div className="text-xs text-slate-500">{job.id}</div>
              </td>
              <td className="px-3 py-3">
                <Badge tone={jobStatusTone(job.status)}>{jobStatusLabels[job.status]}</Badge>
              </td>
              <td className="px-3 py-3">
                <JobProgress job={job} />
              </td>
              <td className="px-3 py-3 text-slate-600">{job.novel_id ?? "-"}</td>
              <td className="px-3 py-3 text-right tabular-nums">{job.chapter_index ?? "-"}</td>
              <td className="max-w-[280px] truncate px-3 py-3 text-slate-500">{job.error ?? "-"}</td>
              <td className="px-4 py-3">
                <div className="flex justify-end gap-2">
                  <Button
                    aria-label={`重试任务 ${job.id}`}
                    disabled={job.status !== "failed" || retryingJobId === job.id}
                    size="sm"
                    variant={job.status === "failed" ? "secondary" : "ghost"}
                    onClick={(event) => {
                      event.stopPropagation();
                      onRetryJob?.(job);
                    }}
                  >
                    <RotateCcw className="h-3.5 w-3.5" />
                    {retryingJobId === job.id ? "重试中" : "重试"}
                  </Button>
                  <Button
                    aria-label={`取消任务 ${job.id}`}
                    disabled={!canCancelJob(job) || cancelingJobId === job.id}
                    size="sm"
                    variant={canCancelJob(job) ? "secondary" : "ghost"}
                    onClick={(event) => {
                      event.stopPropagation();
                      onCancelJob?.(job);
                    }}
                  >
                    <Ban className="h-3.5 w-3.5" />
                    {cancelingJobId === job.id ? "取消中" : "取消"}
                  </Button>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function JobProgress({ job }: { job: ApiJob }) {
  const current = job.progress_current ?? (job.status === "succeeded" ? 1 : 0);
  const total = Math.max(1, job.progress_total ?? 1);
  const percent = Math.min(100, Math.round((current / total) * 100));
  return (
    <div className="min-w-28">
      <div className="mb-1 flex justify-between text-xs text-slate-500">
        <span>{current} / {total}</span>
        <span>{percent}%</span>
      </div>
      <div className="h-1.5 overflow-hidden rounded-full bg-slate-200">
        <div className="h-full rounded-full bg-teal-500" style={{ width: `${percent}%` }} />
      </div>
    </div>
  );
}

export function jobStatusTone(status: ApiJobStatus): "slate" | "teal" | "amber" | "rose" | "blue" {
  if (status === "succeeded") {
    return "teal";
  }
  if (status === "failed") {
    return "rose";
  }
  if (status === "running") {
    return "blue";
  }
  if (status === "cancelled") {
    return "slate";
  }
  return "amber";
}

function canCancelJob(job: ApiJob): boolean {
  return job.status === "queued" || job.status === "running";
}
