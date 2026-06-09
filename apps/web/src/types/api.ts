import type { AgentRole, AgentRunStatus } from "./domain";

export type ApiMode = "mock" | "real";
export type ChapterOperation = "write" | "review" | "rewrite";

export interface ApiClientStatus {
  mode: ApiMode;
  baseUrl: string | null;
  mockEnabled: boolean;
  sseEnabled: boolean;
  sseReady: boolean;
  manualSaveEnabled: boolean;
}

export interface ApiHealthStatus {
  status: string;
  service?: string;
  version?: string;
  checked_at: string;
  sse?: boolean;
}

export type ApiModelProvider = "smoke" | "openai" | "deepseek";

export interface ApiModelSettings {
  provider: ApiModelProvider | (string & {});
  model: string;
  reasoning_effort?: string | null;
}

export interface ChapterStreamEvent {
  event:
    | "queued"
    | "started"
    | "chapter_chunk"
    | "agent_started"
    | "agent_delta"
    | "agent_completed"
    | "artifact_saved"
    | "completed"
    | "error";
  run_id?: string;
  role?: AgentRole;
  status?: AgentRunStatus;
  message: string;
  data?: Record<string, unknown>;
  created_at: string;
}
