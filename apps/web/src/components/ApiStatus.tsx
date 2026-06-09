import { RadioTower, Server } from "lucide-react";
import { api } from "../lib/api";
import { Badge } from "./ui/Badge";

export function ApiStatus() {
  const status = api.getClientStatus();
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
        <div className="flex items-center justify-between gap-2 text-xs text-slate-500">
          <span className="inline-flex items-center gap-1">
            <RadioTower className="h-3.5 w-3.5" />
            SSE
          </span>
          <Badge tone={status.sseEnabled ? "teal" : "slate"}>{status.sseEnabled ? "on" : "off"}</Badge>
        </div>
      </div>
    </div>
  );
}
