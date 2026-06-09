import type { AgentRun } from "../../types/domain";
import { cn } from "../../lib/cn";
import { agentRoleLabels, agentTaskLabels, formatDateTime, formatDuration } from "../../lib/format";
import { Badge } from "../../components/ui/Badge";

export function AgentRunTable({
  runs,
  selectedRunId,
  onSelectRun,
}: {
  runs: AgentRun[];
  selectedRunId?: string;
  onSelectRun?: (run: AgentRun) => void;
}) {
  return (
    <div className="overflow-x-auto bg-white">
      <table className="min-w-[980px] w-full border-collapse text-sm">
        <thead className="table-head">
          <tr>
            <th className="px-4 py-3">运行时间</th>
            <th className="px-3 py-3">Agent</th>
            <th className="px-3 py-3">provider</th>
            <th className="px-3 py-3">状态</th>
            <th className="px-3 py-3 text-right">耗时</th>
            <th className="px-3 py-3">输出摘要</th>
            <th className="px-4 py-3">错误</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-line">
          {runs.map((run) => (
            <tr
              key={run.id}
              className={cn(
                "cursor-pointer hover:bg-slate-50",
                selectedRunId === run.id && "bg-teal-50/70 outline outline-1 outline-accent",
              )}
              onClick={() => onSelectRun?.(run)}
            >
              <td className="px-4 py-3 text-slate-500">{formatDateTime(run.created_at)}</td>
              <td className="px-3 py-3">
                <div className="font-medium">{agentRoleLabels[run.role]}</div>
                <div className="text-xs text-slate-500">{agentTaskLabels[run.task]}</div>
              </td>
              <td className="px-3 py-3">{run.provider}</td>
              <td className="px-3 py-3">
                <Badge tone={run.status === "ok" ? "teal" : run.status === "running" ? "blue" : "rose"}>{run.status}</Badge>
              </td>
              <td className="px-3 py-3 text-right tabular-nums">{formatDuration(run.duration_ms)}</td>
              <td className="px-3 py-3 text-slate-600">{run.output_summary}</td>
              <td className="px-4 py-3 text-slate-500">{run.parse_error ?? "-"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
