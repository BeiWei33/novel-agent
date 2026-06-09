import { useMemo, useState } from "react";
import type { ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import type { AgentRole, AgentRun, AgentRunStatus, AgentTask } from "../types/domain";
import { api, queryKeys } from "../lib/api";
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
  const [statusFilter, setStatusFilter] = useState<AgentRunStatus | "all">("all");
  const [roleFilter, setRoleFilter] = useState<AgentRole | "all">("all");
  const [taskFilter, setTaskFilter] = useState<AgentTask | "all">("all");
  const [providerFilter, setProviderFilter] = useState<AgentRun["provider"] | "all">("all");
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const runOptions = useMemo<AgentRunListOptions>(
    () => ({
      limit: 50,
      status: statusFilter,
      role: roleFilter,
      task: taskFilter,
    }),
    [roleFilter, statusFilter, taskFilter],
  );
  const runsQuery = useQuery({
    queryKey: queryKeys.agentRuns(runOptions),
    queryFn: () => api.getAgentRuns(runOptions),
    refetchInterval: 10_000,
  });
  const runs = runsQuery.data ?? [];
  const filteredRuns = useMemo(
    () =>
      runs.filter((run) => {
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
    [providerFilter, roleFilter, runs, statusFilter, taskFilter],
  );
  const activeRun = selectedRunId ? (filteredRuns.find((run) => run.id === selectedRunId) ?? filteredRuns[0] ?? null) : filteredRuns[0] ?? null;

  return (
    <div>
      <PageHeader title="AgentRun" meta={<span>{filteredRuns.length} / {runs.length} 条</span>} />
      {runsQuery.isLoading ? <LoadingState label="读取运行记录" /> : null}
      {runsQuery.isError ? (
        <StatusBanner tone="danger" title="运行记录读取失败">
          {runsQuery.error instanceof Error ? runsQuery.error.message : "请检查 API 或 mock 数据。"}
        </StatusBanner>
      ) : null}
      {runsQuery.data ? (
        <>
          <AgentRunFilters
            statusFilter={statusFilter}
            roleFilter={roleFilter}
            taskFilter={taskFilter}
            providerFilter={providerFilter}
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
            onReset={() => {
              setStatusFilter("all");
              setRoleFilter("all");
              setTaskFilter("all");
              setProviderFilter("all");
              setSelectedRunId(null);
            }}
          />
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
                      }}
                    >
                      清空筛选
                    </Button>
                  }
                />
              </div>
            )}
            <AgentRunDetail run={activeRun} />
          </div>
        </>
      ) : null}
    </div>
  );
}

function AgentRunFilters({
  statusFilter,
  roleFilter,
  taskFilter,
  providerFilter,
  onStatusChange,
  onRoleChange,
  onTaskChange,
  onProviderChange,
  onReset,
}: {
  statusFilter: AgentRunStatus | "all";
  roleFilter: AgentRole | "all";
  taskFilter: AgentTask | "all";
  providerFilter: AgentRun["provider"] | "all";
  onStatusChange: (value: AgentRunStatus | "all") => void;
  onRoleChange: (value: AgentRole | "all") => void;
  onTaskChange: (value: AgentTask | "all") => void;
  onProviderChange: (value: AgentRun["provider"] | "all") => void;
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

function AgentRunDetail({ run }: { run: AgentRun | null }) {
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
          <Badge tone={run.status === "ok" ? "teal" : run.status === "running" ? "blue" : "rose"}>{run.status}</Badge>
        </div>
        <dl className="grid grid-cols-2 gap-3 text-xs">
          <DetailItem label="provider" value={run.provider} />
          <DetailItem label="耗时" value={formatDuration(run.duration_ms)} />
          <DetailItem label="时间" value={formatDateTime(run.created_at)} />
          <DetailItem label="novel_id" value={run.novel_id ?? "-"} />
        </dl>
      </div>
      <div className="space-y-4 p-4">
        <DetailBlock title="输出摘要" value={run.output_summary} />
        {run.parse_error ? <DetailBlock title="错误信息" value={run.parse_error} tone="danger" /> : null}
        <DetailBlock title="raw_notes" value={run.raw_notes || "-"} />
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
