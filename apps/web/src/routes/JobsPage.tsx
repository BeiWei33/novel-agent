import { useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { RefreshCw } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useSearchParams } from "react-router-dom";
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
const jobStatusValues: ApiJobStatus[] = ["queued", "running", "succeeded", "failed", "cancelled"];
const jobKindValues: ApiJobKind[] = ["create_novel", "write_chapter", "write_chapters", "review_chapter", "rewrite_chapter"];

export function JobsPage() {
  const queryClient = useQueryClient();
  const [searchParams, setSearchParams] = useSearchParams();
  const [statusFilter, setStatusFilter] = useState<ApiJobStatus | "all">(() => jobStatusParam(searchParams.get("status")));
  const [kindFilter, setKindFilter] = useState<ApiJobKind | "all">(() => jobKindParam(searchParams.get("kind")));
  const [novelIdFilter, setNovelIdFilter] = useState(() => trimmedParam(searchParams.get("novel_id")));
  const [sourceJobIdFilter, setSourceJobIdFilter] = useState(() => trimmedParam(searchParams.get("source_job_id")));
  const [selectedJobId, setSelectedJobId] = useState<string | null>(() => jobIdParam(searchParams.get("job_id")));
  const [retryMessage, setRetryMessage] = useState<string | null>(null);
  const [cancelMessage, setCancelMessage] = useState<string | null>(null);

  useEffect(() => {
    setStatusFilter(jobStatusParam(searchParams.get("status")));
    setKindFilter(jobKindParam(searchParams.get("kind")));
    setNovelIdFilter(trimmedParam(searchParams.get("novel_id")));
    setSourceJobIdFilter(trimmedParam(searchParams.get("source_job_id")));
    setSelectedJobId(jobIdParam(searchParams.get("job_id")));
  }, [searchParams]);

  function updateSearch(patch: Partial<JobSearchPatch>) {
    const next = new URLSearchParams(searchParams);
    if (patch.status !== undefined) {
      setOptionalParam(next, "status", patch.status !== "all" ? patch.status : null);
    }
    if (patch.kind !== undefined) {
      setOptionalParam(next, "kind", patch.kind !== "all" ? patch.kind : null);
    }
    if (patch.novelId !== undefined) {
      setOptionalParam(next, "novel_id", patch.novelId);
    }
    if (patch.sourceJobId !== undefined) {
      setOptionalParam(next, "source_job_id", patch.sourceJobId);
    }
    if (patch.jobId !== undefined) {
      setOptionalParam(next, "job_id", patch.jobId);
    }
    setSearchParams(next, { replace: true });
  }

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
      updateSearch({ status: "all", jobId: job.id });
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
      updateSearch({ status: "all", jobId: job.id });
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
              const nextStatus = value as ApiJobStatus | "all";
              setStatusFilter(nextStatus);
              setSelectedJobId(null);
              updateSearch({ status: nextStatus, jobId: null });
            }}
            onKindChange={(value) => {
              const nextKind = value as ApiJobKind | "all";
              setKindFilter(nextKind);
              setSelectedJobId(null);
              updateSearch({ kind: nextKind, jobId: null });
            }}
            onNovelIdChange={(value) => {
              setNovelIdFilter(value);
              setSelectedJobId(null);
              updateSearch({ novelId: value, jobId: null });
            }}
            onSourceJobIdChange={(value) => {
              setSourceJobIdFilter(value);
              setSelectedJobId(null);
              updateSearch({ sourceJobId: value, jobId: null });
            }}
            onRefresh={() => void jobsQuery.refetch()}
            onReset={() => {
              setStatusFilter("all");
              setKindFilter("all");
              setNovelIdFilter("");
              setSourceJobIdFilter("");
              setSelectedJobId(null);
              setSearchParams({}, { replace: true });
            }}
          />
          <div className="grid min-h-[620px] grid-cols-1 xl:grid-cols-[minmax(0,1fr)_380px]">
            {filteredJobs.length > 0 ? (
              <JobTable
                jobs={filteredJobs}
                selectedJobId={activeJob?.id}
                retryingJobId={retryingJobId}
                cancelingJobId={cancelingJobId}
                onSelectJob={(job) => {
                  setSelectedJobId(job.id);
                  updateSearch({ jobId: job.id });
                }}
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

interface JobSearchPatch {
  status: ApiJobStatus | "all";
  kind: ApiJobKind | "all";
  novelId: string | null;
  sourceJobId: string | null;
  jobId: string | null;
}

function jobStatusParam(value: string | null): ApiJobStatus | "all" {
  if (!value || value === "all") {
    return "all";
  }
  return jobStatusValues.includes(value as ApiJobStatus) ? (value as ApiJobStatus) : "all";
}

function jobKindParam(value: string | null): ApiJobKind | "all" {
  if (!value || value === "all") {
    return "all";
  }
  return jobKindValues.includes(value as ApiJobKind) ? (value as ApiJobKind) : "all";
}

function jobIdParam(value: string | null): string | null {
  return trimmedParam(value) || null;
}

function trimmedParam(value: string | null): string {
  return value?.trim() ?? "";
}

function setOptionalParam(params: URLSearchParams, key: string, value: string | null | undefined): void {
  const trimmed = value?.trim();
  if (trimmed) {
    params.set(key, trimmed);
  } else {
    params.delete(key);
  }
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

  const chapterLinks = getJobChapterLinks(job);
  return (
    <div className="grid gap-2">
      <Link to={`/novels/${targetNovelId}`}>
        <Button variant={chapterLinks.length > 0 ? "ghost" : "secondary"} className="w-full">
          打开作品
        </Button>
      </Link>
      {chapterLinks.map((link) => (
        <Link key={`${link.label}-${link.chapterIndex}`} to={`/novels/${targetNovelId}/chapters/${link.chapterIndex}`}>
          <Button variant="secondary" className="w-full">
            {link.label}
          </Button>
        </Link>
      ))}
    </div>
  );
}

function getJobChapterLinks(job: ApiJob): Array<{ label: string; chapterIndex: number }> {
  const indexes = getJobChapterIndexes(job);
  if (indexes.length === 0) {
    return [];
  }
  if (job.kind !== "write_chapters") {
    return [{ label: "打开章节", chapterIndex: indexes[0] }];
  }
  const first = indexes[0];
  const last = indexes[indexes.length - 1];
  if (first === last) {
    return [{ label: "打开章节", chapterIndex: first }];
  }
  return [
    { label: "打开首章", chapterIndex: first },
    { label: "打开末章", chapterIndex: last },
  ];
}

function getJobChapterIndexes(job: ApiJob): number[] {
  const indexes: number[] = [];
  const addIndex = (value: unknown) => {
    const index = toChapterIndex(value);
    if (index) {
      indexes.push(index);
    }
  };

  addIndex(job.chapter_index);
  addIndex(job.payload.chapter_index);
  addIndex(job.result?.chapter_index);
  addIndex(job.payload.chapter_start);
  addIndex(job.payload.chapter_end);
  addIndex(job.result?.chapter_start);
  addIndex(job.result?.chapter_end);
  addDraftIndexes(job.payload.drafts, addIndex);
  addDraftIndexes(job.result?.drafts, addIndex);

  return [...new Set(indexes)].sort((a, b) => a - b);
}

function addDraftIndexes(value: unknown, addIndex: (value: unknown) => void): void {
  if (!Array.isArray(value)) {
    return;
  }
  value.forEach((item) => {
    if (item && typeof item === "object" && "chapter_index" in item) {
      addIndex(item.chapter_index);
    }
  });
}

function toChapterIndex(value: unknown): number | null {
  const numberValue = typeof value === "number" ? value : typeof value === "string" ? Number.parseInt(value, 10) : Number.NaN;
  if (!Number.isInteger(numberValue) || numberValue < 1) {
    return null;
  }
  return numberValue;
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
