import { useQuery } from "@tanstack/react-query";
import { Activity, AlertTriangle, CheckCircle2, Clock3 } from "lucide-react";
import { api, queryKeys } from "../lib/api";
import { agentRoleLabels, agentTaskLabels, formatDateTime, formatDuration } from "../lib/format";
import { Badge } from "./ui/Badge";

export function AgentRail() {
  const { data: runs = [] } = useQuery({
    queryKey: queryKeys.agentRunList({ limit: 8 }),
    queryFn: () => api.getAgentRuns({ limit: 8 }),
    refetchInterval: 10_000,
  });

  return (
    <aside className="hidden min-h-screen border-l border-line bg-white xl:block">
      <div className="flex h-14 items-center gap-2 border-b border-line px-4">
        <Activity className="h-4 w-4 text-accent" />
        <h2 className="text-sm font-semibold text-ink">AgentRun</h2>
      </div>
      <div className="space-y-3 p-3">
        {runs.slice(0, 8).map((run) => (
          <div key={run.id} className="rounded-md border border-border bg-white p-3 shadow-soft">
            <div className="mb-2 flex items-start justify-between gap-2">
              <div>
                <div className="text-sm font-semibold text-ink">{agentRoleLabels[run.role]}</div>
                <div className="text-xs text-slate-500">{agentTaskLabels[run.task]}</div>
              </div>
              <RunStatus status={run.status} />
            </div>
            <p className="line-clamp-2 text-xs leading-5 text-slate-600">{run.output_summary}</p>
            <div className="mt-3 flex items-center justify-between text-xs text-slate-500">
              <span className="inline-flex items-center gap-1">
                <Clock3 className="h-3.5 w-3.5" />
                {formatDuration(run.duration_ms)}
              </span>
              <span>{formatDateTime(run.created_at)}</span>
            </div>
          </div>
        ))}
      </div>
    </aside>
  );
}

function RunStatus({ status }: { status: "ok" | "fallback" | "parse_error" | "running" }) {
  if (status === "ok") {
    return (
      <Badge tone="teal" className="gap-1">
        <CheckCircle2 className="h-3 w-3" />
        ok
      </Badge>
    );
  }
  if (status === "running") {
    return <Badge tone="blue">running</Badge>;
  }
  return (
    <Badge tone={status === "fallback" ? "amber" : "rose"} className="gap-1">
      <AlertTriangle className="h-3 w-3" />
      {status}
    </Badge>
  );
}
