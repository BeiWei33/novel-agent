import { useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import type { AgentRole, AgentRun, AgentRunStatus, AgentRunStatusSummary, AgentTask } from "../types/domain";
import { api, apiConfig, queryKeys } from "../lib/api";
import type { AgentRunListOptions } from "../lib/api";
import { agentRoleLabels, agentTaskLabels, formatDateTime, formatDuration } from "../lib/format";
import { AgentRunTable } from "../features/agent-runs/AgentRunTable";
import { PageHeader } from "../components/PageHeader";
import { LoadingState } from "../components/LoadingState";
import { Badge } from "../components/ui/Badge";
import { StatusBanner } from "../components/StatusBanner";
import { EmptyState } from "../components/EmptyState";
import { Button } from "../components/ui/Button";

export function AgentRunsPage() {
  const queryClient = useQueryClient();
  const [statusFilter, setStatusFilter] = useState<AgentRunStatus | "all">("all");
  const [roleFilter, setRoleFilter] = useState<AgentRole | "all">("all");
  const [taskFilter, setTaskFilter] = useState<AgentTask | "all">("all");
  const [providerFilter, setProviderFilter] = useState<AgentRun["provider"] | "all">("all");
  const [novelIdFilter, setNovelIdFilter] = useState("");
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const [streamState, setStreamState] = useState<{
    status: "idle" | "refreshing" | "ok" | "error";
    message?: string;
    checkedAt?: string;
  }>({ status: "idle" });
  const useSseSnapshots = apiConfig.sseEnabled && !apiConfig.useMock && typeof ReadableStream !== "undefined" && typeof TextDecoder !== "undefined";
  const runOptions = useMemo<AgentRunListOptions>(
    () => ({
      limit: 50,
      novelId: novelIdFilter,
      status: statusFilter,
      role: roleFilter,
      task: taskFilter,
    }),
    [novelIdFilter, roleFilter, statusFilter, taskFilter],
  );
  const runsQuery = useQuery({
    queryKey: queryKeys.agentRuns(runOptions),
    queryFn: () => api.getAgentRunReport(runOptions),
    refetchInterval: useSseSnapshots ? false : 10_000,
  });
  useEffect(() => {
    if (!useSseSnapshots) {
      setStreamState({ status: "idle" });
      return;
    }

    const controller = new AbortController();
    let timer: number | undefined;
    async function refreshSnapshot() {
      setStreamState((current) => ({ ...current, status: "refreshing" }));
      try {
        await api.streamAgentRunReport(
          runOptions,
          (report) => {
            queryClient.setQueryData(queryKeys.agentRuns(runOptions), report);
            setStreamState({ status: "ok", checkedAt: new Date().toISOString() });
          },
          controller.signal,
        );
      } catch (error) {
        if (!controller.signal.aborted) {
          setStreamState({
            status: "error",
            message: error instanceof Error ? error.message : "SSE 快照读取失败。",
            checkedAt: new Date().toISOString(),
          });
        }
      }
      if (!controller.signal.aborted) {
        timer = window.setTimeout(refreshSnapshot, 10_000);
      }
    }

    void refreshSnapshot();
    return () => {
      controller.abort();
      if (timer) {
        window.clearTimeout(timer);
      }
    };
  }, [queryClient, runOptions, useSseSnapshots]);
  const runs = runsQuery.data?.runs ?? [];
  const trimmedNovelIdFilter = novelIdFilter.trim();
  const filteredRuns = useMemo(
    () =>
      runs.filter((run) => {
        if (trimmedNovelIdFilter && run.novel_id !== trimmedNovelIdFilter) {
          return false;
        }
        if (statusFilter !== "all" && run.status !== statusFilter) {
          return false;
        }
        if (roleFilter !== "all" && run.role !== roleFilter) {
          return false;
        }
        if (taskFilter !== "all" && run.task !== taskFilter) {
          return false;
        }
        if (providerFilter !== "all" && run.provider !== providerFilter) {
          return false;
        }
        return true;
      }),
    [providerFilter, roleFilter, runs, statusFilter, taskFilter, trimmedNovelIdFilter],
  );
  const activeRun = selectedRunId ? (filteredRuns.find((run) => run.id === selectedRunId) ?? filteredRuns[0] ?? null) : filteredRuns[0] ?? null;
  const runDetailQuery = useQuery({
    queryKey: activeRun ? queryKeys.agentRun(activeRun.id) : ["agent-run", "none"],
    queryFn: () => api.getAgentRun(activeRun?.id ?? ""),
    enabled: Boolean(activeRun),
    refetchInterval: activeRun?.status === "running" ? 5_000 : false,
  });
  const detailRun = runDetailQuery.data ?? activeRun;
  const summary = useMemo(
    () => (providerFilter === "all" ? (runsQuery.data?.summary ?? summarizePageAgentRuns(filteredRuns)) : summarizePageAgentRuns(filteredRuns)),
    [filteredRuns, providerFilter, runsQuery.data?.summary],
  );

  return (
    <div>
      <PageHeader
        title="AgentRun"
        meta={
          <>
            <span>{filteredRuns.length} / {runs.length} 条</span>
            {useSseSnapshots ? (
              <Badge tone={streamState.status === "error" ? "rose" : streamState.status === "refreshing" ? "blue" : "teal"}>
                SSE {streamState.status === "refreshing" ? "刷新中" : streamState.status === "error" ? "异常" : "快照"}
              </Badge>
            ) : null}
          </>
        }
      />
      {runsQuery.isLoading ? <LoadingState label="读取运行记录" /> : null}
      {runsQuery.isError ? (
        <StatusBanner tone="danger" title="运行记录读取失败">
          {runsQuery.error instanceof Error ? runsQuery.error.message : "请检查 API 或 mock 数据。"}
        </StatusBanner>
      ) : null}
      {streamState.status === "error" ? (
        <StatusBanner tone="danger" title="SSE 快照刷新失败">
          {streamState.message ?? "请检查 API 或稍后重试。"}
        </StatusBanner>
      ) : null}
      {runsQuery.data ? (
        <>
          <AgentRunFilters
            statusFilter={statusFilter}
            roleFilter={roleFilter}
            taskFilter={taskFilter}
            providerFilter={providerFilter}
            novelIdFilter={novelIdFilter}
            onStatusChange={(value) => {
              setStatusFilter(value);
              setSelectedRunId(null);
            }}
            onRoleChange={(value) => {
              setRoleFilter(value);
              setSelectedRunId(null);
            }}
            onTaskChange={(value) => {
              setTaskFilter(value);
              setSelectedRunId(null);
            }}
            onProviderChange={(value) => {
              setProviderFilter(value);
              setSelectedRunId(null);
            }}
            onNovelIdChange={(value) => {
              setNovelIdFilter(value);
              setSelectedRunId(null);
            }}
            onReset={() => {
              setStatusFilter("all");
              setRoleFilter("all");
              setTaskFilter("all");
              setProviderFilter("all");
              setNovelIdFilter("");
              setSelectedRunId(null);
            }}
          />
          <AgentRunSummaryBar summary={summary} />
          <div className="grid min-h-[620px] grid-cols-1 xl:grid-cols-[minmax(0,1fr)_360px]">
            {filteredRuns.length > 0 ? (
              <AgentRunTable runs={filteredRuns} selectedRunId={activeRun?.id} onSelectRun={(run) => setSelectedRunId(run.id)} />
            ) : (
              <div className="p-4">
                <EmptyState
                  title="没有匹配的运行记录"
                  action={
                    <Button
                      variant="secondary"
                      onClick={() => {
                        setStatusFilter("all");
                        setRoleFilter("all");
                        setTaskFilter("all");
                        setProviderFilter("all");
                        setNovelIdFilter("");
                      }}
                    >
                      清空筛选
                    </Button>
                  }
                />
              </div>
            )}
            <AgentRunDetail run={detailRun} isLoading={runDetailQuery.isLoading} error={runDetailQuery.error} />
          </div>
        </>
      ) : null}
    </div>
  );
}

function AgentRunSummaryBar({ summary }: { summary: AgentRunStatusSummary }) {
  const badRuns = summary.fallback + summary.parse_error;
  return (
    <div className="grid grid-cols-2 gap-3 border-b border-line bg-slate-50 px-4 py-3 text-xs md:grid-cols-4 xl:grid-cols-8">
      <SummaryMetric label="总数" value={formatNumber(summary.total)} />
      <SummaryMetric label="ok" value={formatNumber(summary.ok)} tone="teal" />
      <SummaryMetric label="异常" value={formatNumber(badRuns)} tone={badRuns > 0 ? "rose" : "slate"} />
      <SummaryMetric label="fallback" value={formatNumber(summary.fallback)} />
      <SummaryMetric label="parse_error" value={formatNumber(summary.parse_error)} />
      <SummaryMetric label="总耗时" value={formatDuration(summary.duration_ms_total)} />
      <SummaryMetric label="token runs" value={formatNumber(summary.tokenized_runs)} />
      <SummaryMetric label="tokens" value={formatNumber(summary.total_tokens)} />
    </div>
  );
}

function SummaryMetric({
  label,
  value,
  tone = "slate",
}: {
  label: string;
  value: string;
  tone?: "slate" | "teal" | "rose";
}) {
  return (
    <div className="min-w-0">
      <div className="text-slate-500">{label}</div>
      <div className={`mt-1 truncate text-sm font-semibold ${tone === "teal" ? "text-teal-700" : tone === "rose" ? "text-rose-700" : "text-ink"}`}>
        {value}
      </div>
    </div>
  );
}

function AgentRunFilters({
  statusFilter,
  roleFilter,
  taskFilter,
  providerFilter,
  novelIdFilter,
  onStatusChange,
  onRoleChange,
  onTaskChange,
  onProviderChange,
  onNovelIdChange,
  onReset,
}: {
  statusFilter: AgentRunStatus | "all";
  roleFilter: AgentRole | "all";
  taskFilter: AgentTask | "all";
  providerFilter: AgentRun["provider"] | "all";
  novelIdFilter: string;
  onStatusChange: (value: AgentRunStatus | "all") => void;
  onRoleChange: (value: AgentRole | "all") => void;
  onTaskChange: (value: AgentTask | "all") => void;
  onProviderChange: (value: AgentRun["provider"] | "all") => void;
  onNovelIdChange: (value: string) => void;
  onReset: () => void;
}) {
  return (
    <div className="flex flex-wrap items-center gap-3 border-b border-line bg-white px-4 py-3">
      <FilterSelect label="状态" value={statusFilter} onChange={(value) => onStatusChange(value as AgentRunStatus | "all")}>
        <option value="all">全部</option>
        <option value="ok">ok</option>
        <option value="fallback">fallback</option>
        <option value="parse_error">parse_error</option>
        <option value="running">running</option>
      </FilterSelect>
      <FilterSelect label="角色" value={roleFilter} onChange={(value) => onRoleChange(value as AgentRole | "all")}>
        <option value="all">全部</option>
        {Object.entries(agentRoleLabels).map(([role, label]) => (
          <option key={role} value={role}>
            {label}
          </option>
        ))}
      </FilterSelect>
      <FilterSelect label="任务" value={taskFilter} onChange={(value) => onTaskChange(value as AgentTask | "all")}>
        <option value="all">全部</option>
        {Object.entries(agentTaskLabels).map(([task, label]) => (
          <option key={task} value={task}>
            {label}
          </option>
        ))}
      </FilterSelect>
      <FilterSelect
        label="provider"
        value={providerFilter}
        onChange={(value) => onProviderChange(value as AgentRun["provider"] | "all")}
      >
        <option value="all">全部</option>
        <option value="smoke">smoke</option>
        <option value="openai">openai</option>
        <option value="deepseek">deepseek</option>
      </FilterSelect>
      <FilterInput label="作品 ID" value={novelIdFilter} onChange={onNovelIdChange} placeholder="novel_id" />
      <Button size="sm" variant="ghost" onClick={onReset}>
        清空
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

function AgentRunDetail({ run, isLoading, error }: { run: AgentRun | null; isLoading?: boolean; error?: unknown }) {
  if (!run) {
    return <aside className="border-l border-line bg-slate-50 p-4 text-sm text-slate-500">暂无运行记录</aside>;
  }

  return (
    <aside className="border-l border-line bg-slate-50">
      <div className="border-b border-line bg-white p-4">
        <div className="mb-3 flex items-start justify-between gap-3">
          <div>
            <h2 className="text-sm font-semibold text-ink">{agentRoleLabels[run.role]}</h2>
            <p className="mt-1 text-xs text-slate-500">{agentTaskLabels[run.task]}</p>
          </div>
          <div className="flex flex-col items-end gap-2">
            <Badge tone={run.status === "ok" ? "teal" : run.status === "running" ? "blue" : "rose"}>{run.status}</Badge>
            {isLoading ? <span className="text-xs text-slate-500">刷新详情中</span> : null}
          </div>
        </div>
        <dl className="grid grid-cols-2 gap-3 text-xs">
          <DetailItem label="run_id" value={run.id} />
          <DetailItem label="provider" value={run.provider} />
          <DetailItem label="耗时" value={formatDuration(run.duration_ms)} />
          <DetailItem label="attempt" value={run.attempt ? String(run.attempt) : "-"} />
          <DetailItem label="tokens" value={typeof run.total_tokens === "number" ? formatNumber(run.total_tokens) : "-"} />
          <DetailItem label="时间" value={formatDateTime(run.created_at)} />
          <DetailItem label="novel_id" value={run.novel_id ?? "-"} />
        </dl>
      </div>
      <div className="space-y-4 p-4">
        {error ? (
          <div className="rounded-md border border-amber-200 bg-amber-50 p-3 text-sm leading-6 text-amber-800">
            详情接口读取失败，当前显示列表快照：{error instanceof Error ? error.message : "未知错误"}
          </div>
        ) : null}
        <DetailBlock title="输出摘要" value={run.output_summary} />
        {run.parse_error ? <DetailBlock title="错误信息" value={run.parse_error} tone="danger" /> : null}
        <DetailBlock title="raw_notes" value={run.raw_notes || "-"} />
        <DetailBlock title="raw_text" value={run.raw_text || "-"} mono />
        <div>
          <h3 className="mb-2 text-xs font-semibold text-slate-600">structured</h3>
          <pre className="max-h-80 overflow-auto rounded-md border border-border bg-white p-3 text-xs leading-5 text-slate-700">
            {JSON.stringify(run.structured, null, 2)}
          </pre>
        </div>
      </div>
    </aside>
  );
}

function summarizePageAgentRuns(runs: AgentRun[]): AgentRunStatusSummary {
  return runs.reduce(
    (summary, run) => {
      summary.total += 1;
      if (run.status === "ok") {
        summary.ok += 1;
      } else if (run.status === "fallback") {
        summary.fallback += 1;
      } else if (run.status === "parse_error") {
        summary.parse_error += 1;
      }
      summary.duration_ms_total += run.duration_ms;
      if (typeof run.total_tokens === "number") {
        summary.tokenized_runs += 1;
        summary.total_tokens += run.total_tokens;
      }
      return summary;
    },
    {
      total: 0,
      ok: 0,
      fallback: 0,
      parse_error: 0,
      duration_ms_total: 0,
      tokenized_runs: 0,
      prompt_tokens: 0,
      completion_tokens: 0,
      total_tokens: 0,
    },
  );
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat("zh-CN").format(value);
}

function DetailItem({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-slate-500">{label}</dt>
      <dd className="mt-1 truncate font-medium text-slate-800">{value}</dd>
    </div>
  );
}

function DetailBlock({ title, value, tone, mono }: { title: string; value: string; tone?: "danger"; mono?: boolean }) {
  return (
    <div>
      <h3 className="mb-2 text-xs font-semibold text-slate-600">{title}</h3>
      <p className={`max-h-72 overflow-auto whitespace-pre-wrap rounded-md border p-3 text-sm leading-6 ${mono ? "font-mono text-xs" : ""} ${tone === "danger" ? "border-rose-200 bg-rose-50 text-rose-800" : "border-border bg-white text-slate-700"}`}>
        {value}
      </p>
    </div>
  );
}
