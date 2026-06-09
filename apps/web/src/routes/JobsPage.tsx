import { useMemo, useState } from "react";
import type { ReactNode } from "react";
import { RefreshCw } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { EmptyState } from "../components/EmptyState";
import { LoadingState } from "../components/LoadingState";
import { PageHeader } from "../components/PageHeader";
import { StatusBanner } from "../components/StatusBanner";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { JobTable, jobStatusTone } from "../features/jobs/JobTable";
import { api, queryKeys } from "../lib/api";
import type { JobListOptions } from "../lib/api";
import { formatDateTime, jobKindLabels, jobStatusLabels } from "../lib/format";
import type { ApiJob, ApiJobKind, ApiJobStatus } from "../types/domain";

const jobLimit = 50;

export function JobsPage() {
  const queryClient = useQueryClient();
  const [statusFilter, setStatusFilter] = useState<ApiJobStatus | "all">("all");
  const [kindFilter, setKindFilter] = useState<ApiJobKind | "all">("all");
  const [novelIdFilter, setNovelIdFilter] = useState("");
  const [sourceJobIdFilter, setSourceJobIdFilter] = useState("");
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null);
  const [retryMessage, setRetryMessage] = useState<string | null>(null);
  const [cancelMessage, setCancelMessage] = useState<string | null>(null);
  const jobOptions = useMemo<JobListOptions>(
    () => ({
      limit: jobLimit,
      status: statusFilter,
      kind: kindFilter,
      novelId: novelIdFilter,
      sourceJobId: sourceJobIdFilter,
    }),
    [kindFilter, novelIdFilter, sourceJobIdFilter, statusFilter],
  );
  const jobsQueryKey = queryKeys.jobs(jobOptions);
  const jobsQuery = useQuery({
    queryKey: jobsQueryKey,
    queryFn: () => api.getJobs(jobOptions),
    refetchInterval: 5_000,
  });
  const retryMutation = useMutation({
    mutationFn: (jobId: string) => api.retryJob(jobId),
    onSuccess: (job) => {
      setStatusFilter("all");
      setSelectedJobId(job.id);
      setRetryMessage(`已创建重试任务 ${job.id}`);
      setCancelMessage(null);
      queryClient.setQueryData<ApiJob[]>(queryKeys.jobs({ ...jobOptions, status: "all" }), (current) => [
        job,
        ...(current ?? []).filter((item) => item.id !== job.id),
      ]);
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobsRoot });
    },
  });
  const cancelMutation = useMutation({
    mutationFn: (jobId: string) => api.cancelJob(jobId),
    onSuccess: (job) => {
      setStatusFilter("all");
      setSelectedJobId(job.id);
      setCancelMessage(`已取消任务 ${job.id}`);
      setRetryMessage(null);
      queryClient.setQueryData<ApiJob[]>(queryKeys.jobs({ ...jobOptions, status: "all" }), (current) => {
        const jobs = current ?? [];
        const updated = jobs.map((item) => (item.id === job.id ? job : item));
        return jobs.some((item) => item.id === job.id) ? updated : [job, ...updated];
      });
      void queryClient.invalidateQueries({ queryKey: queryKeys.jobsRoot });
    },
  });

  const jobs = jobsQuery.data ?? [];
  const trimmedNovelIdFilter = novelIdFilter.trim();
  const trimmedSourceJobIdFilter = sourceJobIdFilter.trim();
  const filteredJobs = useMemo(
    () =>
      jobs.filter((job) => {
        if (statusFilter !== "all" && job.status !== statusFilter) {
          return false;
        }
        if (kindFilter !== "all" && job.kind !== kindFilter) {
          return false;
        }
        if (trimmedNovelIdFilter && job.novel_id !== trimmedNovelIdFilter) {
          return false;
        }
        if (trimmedSourceJobIdFilter && job.source_job_id !== trimmedSourceJobIdFilter) {
          return false;
        }
        return true;
      }),
    [jobs, kindFilter, statusFilter, trimmedNovelIdFilter, trimmedSourceJobIdFilter],
  );
  const activeJob = selectedJobId ? (filteredJobs.find((job) => job.id === selectedJobId) ?? filteredJobs[0] ?? null) : filteredJobs[0] ?? null;
  const retryingJobId = retryMutation.isPending ? retryMutation.variables : undefined;
  const cancelingJobId = cancelMutation.isPending ? cancelMutation.variables : undefined;

  return (
    <div>
      <PageHeader title="任务队列" meta={<span>{filteredJobs.length} / {jobs.length} 条</span>} />
      {jobsQuery.isLoading ? <LoadingState label="读取后台任务" /> : null}
      {jobsQuery.isError ? (
        <StatusBanner tone="danger" title="任务读取失败">
          {jobsQuery.error instanceof Error ? jobsQuery.error.message : "请检查 API 或 mock 数据。"}
        </StatusBanner>
      ) : null}
      {retryMessage ? (
        <StatusBanner tone="success" title="重试已提交">
          {retryMessage}
        </StatusBanner>
      ) : null}
      {retryMutation.isError ? (
        <StatusBanner tone="danger" title="重试失败">
          {retryMutation.error instanceof Error ? retryMutation.error.message : "只能重试 failed 状态的任务。"}
        </StatusBanner>
      ) : null}
      {cancelMessage ? (
        <StatusBanner tone="success" title="任务已取消">
          {cancelMessage}
        </StatusBanner>
      ) : null}
      {cancelMutation.isError ? (
        <StatusBanner tone="danger" title="取消失败">
          {cancelMutation.error instanceof Error ? cancelMutation.error.message : "只能取消 queued 或 running 状态的任务。"}
        </StatusBanner>
      ) : null}
      {jobsQuery.data ? (
        <>
          <JobFilters
            statusFilter={statusFilter}
            kindFilter={kindFilter}
            novelIdFilter={novelIdFilter}
            sourceJobIdFilter={sourceJobIdFilter}
            isRefreshing={jobsQuery.isFetching}
            onStatusChange={(value) => {
              setStatusFilter(value);
              setSelectedJobId(null);
            }}
            onKindChange={(value) => {
              setKindFilter(value);
              setSelectedJobId(null);
            }}
            onNovelIdChange={(value) => {
              setNovelIdFilter(value);
              setSelectedJobId(null);
            }}
            onSourceJobIdChange={(value) => {
              setSourceJobIdFilter(value);
              setSelectedJobId(null);
            }}
            onRefresh={() => void jobsQuery.refetch()}
            onReset={() => {
              setStatusFilter("all");
              setKindFilter("all");
              setNovelIdFilter("");
              setSourceJobIdFilter("");
              setSelectedJobId(null);
            }}
          />
          <div className="grid min-h-[620px] grid-cols-1 xl:grid-cols-[minmax(0,1fr)_380px]">
            {filteredJobs.length > 0 ? (
              <JobTable
                jobs={filteredJobs}
                selectedJobId={activeJob?.id}
                retryingJobId={retryingJobId}
                cancelingJobId={cancelingJobId}
                onSelectJob={(job) => setSelectedJobId(job.id)}
                onRetryJob={(job) => {
                  setRetryMessage(null);
                  setCancelMessage(null);
                  retryMutation.mutate(job.id);
                }}
                onCancelJob={(job) => {
                  setCancelMessage(null);
                  setRetryMessage(null);
                  cancelMutation.mutate(job.id);
                }}
              />
            ) : (
              <div className="p-4">
                <EmptyState
                  title="没有匹配的任务"
                  action={
                    <Button
                      variant="secondary"
                      onClick={() => {
                        setStatusFilter("all");
                        setKindFilter("all");
                        setNovelIdFilter("");
                        setSourceJobIdFilter("");
                      }}
                    >
                      清空筛选
                    </Button>
                  }
                />
              </div>
            )}
            <JobDetail job={activeJob} />
          </div>
        </>
      ) : null}
    </div>
  );
}

function JobFilters({
  statusFilter,
  kindFilter,
  novelIdFilter,
  sourceJobIdFilter,
  isRefreshing,
  onStatusChange,
  onKindChange,
  onNovelIdChange,
  onSourceJobIdChange,
  onRefresh,
  onReset,
}: {
  statusFilter: ApiJobStatus | "all";
  kindFilter: ApiJobKind | "all";
  novelIdFilter: string;
  sourceJobIdFilter: string;
  isRefreshing: boolean;
  onStatusChange: (value: ApiJobStatus | "all") => void;
  onKindChange: (value: ApiJobKind | "all") => void;
  onNovelIdChange: (value: string) => void;
  onSourceJobIdChange: (value: string) => void;
  onRefresh: () => void;
  onReset: () => void;
}) {
  return (
    <div className="flex flex-wrap items-center gap-3 border-b border-line bg-white px-4 py-3">
      <FilterSelect label="状态" value={statusFilter} onChange={(value) => onStatusChange(value as ApiJobStatus | "all")}>
        <option value="all">全部</option>
        {Object.entries(jobStatusLabels).map(([status, label]) => (
          <option key={status} value={status}>
            {label}
          </option>
        ))}
      </FilterSelect>
      <FilterSelect label="类型" value={kindFilter} onChange={(value) => onKindChange(value as ApiJobKind | "all")}>
        <option value="all">全部</option>
        {Object.entries(jobKindLabels).map(([kind, label]) => (
          <option key={kind} value={kind}>
            {label}
          </option>
        ))}
      </FilterSelect>
      <FilterInput label="作品 ID" value={novelIdFilter} onChange={onNovelIdChange} placeholder="novel_id" />
      <FilterInput label="源任务" value={sourceJobIdFilter} onChange={onSourceJobIdChange} placeholder="source_job_id" />
      <Button size="sm" variant="ghost" onClick={onReset}>
        清空
      </Button>
      <Button size="sm" variant="secondary" onClick={onRefresh}>
        <RefreshCw className={`h-3.5 w-3.5 ${isRefreshing ? "animate-spin" : ""}`} />
        刷新
      </Button>
    </div>
  );
}

function FilterSelect({
  label,
  value,
  onChange,
  children,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  children: ReactNode;
}) {
  return (
    <label className="flex items-center gap-2 text-xs font-medium text-slate-600">
      {label}
      <select value={value} onChange={(event) => onChange(event.target.value)} className="input h-8 w-36">
        {children}
      </select>
    </label>
  );
}

function FilterInput({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
}) {
  return (
    <label className="flex items-center gap-2 text-xs font-medium text-slate-600">
      {label}
      <input
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        className="input h-8 w-44"
      />
    </label>
  );
}

function JobDetail({ job }: { job: ApiJob | null }) {
  if (!job) {
    return <aside className="border-l border-line bg-slate-50 p-4 text-sm text-slate-500">暂无后台任务</aside>;
  }

  return (
    <aside className="border-l border-line bg-slate-50">
      <div className="border-b border-line bg-white p-4">
        <div className="mb-3 flex items-start justify-between gap-3">
          <div>
            <h2 className="text-sm font-semibold text-ink">{jobKindLabels[job.kind]}</h2>
            <p className="mt-1 max-w-[280px] truncate text-xs text-slate-500">{job.id}</p>
          </div>
          <Badge tone={jobStatusTone(job.status)}>{jobStatusLabels[job.status]}</Badge>
        </div>
        <dl className="grid grid-cols-2 gap-3 text-xs">
          <DetailItem label="novel_id" value={job.novel_id ?? "-"} />
          <DetailItem label="章节" value={job.chapter_index ? String(job.chapter_index) : "-"} />
          <DetailItem label="源任务" value={job.source_job_id ?? "-"} />
          <DetailItem label="进度" value={`${job.progress_current ?? 0} / ${job.progress_total ?? 1}`} />
          <DetailItem label="创建" value={formatDateTime(job.created_at)} />
          <DetailItem label="更新" value={formatDateTime(job.updated_at)} />
        </dl>
      </div>
      <div className="space-y-4 p-4">
        {job.error ? <DetailBlock title="错误信息" value={job.error} tone="danger" /> : null}
        <JsonBlock title="payload" value={job.payload} />
        <JobResultLink job={job} />
        <JsonBlock title="result" value={job.result ?? null} />
      </div>
    </aside>
  );
}

function DetailItem({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-slate-500">{label}</dt>
      <dd className="mt-1 truncate font-medium text-slate-800">{value}</dd>
    </div>
  );
}

function DetailBlock({ title, value, tone }: { title: string; value: string; tone?: "danger" }) {
  return (
    <div>
      <h3 className="mb-2 text-xs font-semibold text-slate-600">{title}</h3>
      <p className={`rounded-md border p-3 text-sm leading-6 ${tone === "danger" ? "border-rose-200 bg-rose-50 text-rose-800" : "border-border bg-white text-slate-700"}`}>
        {value}
      </p>
    </div>
  );
}

function JobResultLink({ job }: { job: ApiJob }) {
  const resultNovel = job.result?.novel;
  const resultNovelId =
    resultNovel && typeof resultNovel === "object" && "id" in resultNovel && typeof resultNovel.id === "string"
      ? resultNovel.id
      : null;
  const targetNovelId = job.novel_id ?? resultNovelId;
  if (!targetNovelId) {
    return null;
  }

  const chapterIndex = job.chapter_index;
  const to = chapterIndex ? `/novels/${targetNovelId}/chapters/${chapterIndex}` : `/novels/${targetNovelId}`;
  return (
    <Link to={to}>
      <Button variant="secondary" className="w-full">
        {chapterIndex ? "打开章节" : "打开作品"}
      </Button>
    </Link>
  );
}

function JsonBlock({ title, value }: { title: string; value: unknown }) {
  return (
    <div>
      <h3 className="mb-2 text-xs font-semibold text-slate-600">{title}</h3>
      <pre className="max-h-80 overflow-auto rounded-md border border-border bg-white p-3 text-xs leading-5 text-slate-700">
        {JSON.stringify(value, null, 2)}
      </pre>
    </div>
  );
}
