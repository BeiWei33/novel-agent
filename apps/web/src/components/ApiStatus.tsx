import { useQuery } from "@tanstack/react-query";
import { RadioTower, Server, Wifi, WifiOff } from "lucide-react";
import { api, queryKeys } from "../lib/api";
import { formatDateTime } from "../lib/format";
import { Badge } from "./ui/Badge";
import { ModelSettingsPanel } from "./ModelSettingsPanel";

export function ApiStatus() {
  const status = api.getClientStatus();
  const healthQuery = useQuery({
    queryKey: queryKeys.health,
    queryFn: () => api.getHealth(),
    refetchInterval: status.mode === "real" ? 30_000 : false,
    retry: status.mode === "real" ? 1 : false,
  });
  const healthTone = healthQuery.isError ? "rose" : healthQuery.isFetching ? "amber" : "teal";
  const healthLabel = healthQuery.isError
    ? "offline"
    : healthQuery.isFetching && !healthQuery.data
      ? "checking"
      : status.mode === "mock"
        ? "local"
        : "online";
  const healthDetails = healthQuery.data
    ? [healthQuery.data.service, healthQuery.data.version].filter(Boolean).join(" ")
    : "";
  const sseLabel = !status.sseEnabled
    ? "off"
    : !status.sseReady
      ? "unsupported"
      : healthQuery.data?.sse === false
        ? "server off"
        : "on";
  const sseTone = sseLabel === "on" ? "teal" : "slate";
  const HealthIcon = healthQuery.isError ? WifiOff : Wifi;
  return (
    <div className="border-t border-line p-3">
      <div className="hidden space-y-2 rounded-md border border-border bg-slate-50 p-3 md:block">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 text-xs font-semibold text-slate-700">
            <Server className="h-3.5 w-3.5" />
            API
          </div>
          <Badge tone={status.mode === "mock" ? "amber" : "teal"}>{status.mode}</Badge>
        </div>
        <div className="truncate text-xs text-slate-500">{status.baseUrl ?? "mock://local-session"}</div>
        {healthDetails ? <div className="truncate text-xs text-slate-500">{healthDetails}</div> : null}
        <div className="flex items-center justify-between gap-2 text-xs text-slate-500">
          <span className="inline-flex items-center gap-1">
            <HealthIcon className="h-3.5 w-3.5" />
            Health
          </span>
          <Badge tone={healthTone}>{healthLabel}</Badge>
        </div>
        <div className="truncate text-xs text-slate-500">
          {healthQuery.data ? `checked ${formatDateTime(healthQuery.data.checked_at)}` : healthQuery.isError ? "health check failed" : "checking health"}
        </div>
        <div className="flex items-center justify-between gap-2 text-xs text-slate-500">
          <span className="inline-flex items-center gap-1">
            <RadioTower className="h-3.5 w-3.5" />
            SSE
          </span>
          <Badge tone={sseTone}>{sseLabel}</Badge>
        </div>
        <ModelSettingsPanel />
      </div>
    </div>
  );
}
