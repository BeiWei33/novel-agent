import type {
  AgentRole,
  AgentRun,
  AgentRunReport,
  AgentRunStatus,
  AgentRunStatusSummary,
  AgentTask,
  ApiJob,
  ApiJobKind,
  ApiJobStatus,
  Chapter,
  ChapterDraft,
  ChapterOutline,
  ChapterVersion,
  ContinuityReport,
  CreateNovelInput,
  Novel,
  NovelBible,
  NovelDetail,
  NovelListItem,
  ReviewReport,
  WorldSetting,
} from "../types/domain";
import type { ApiClientStatus, ApiHealthStatus, ChapterOperation, ChapterStreamEvent } from "../types/api";
import { countWords } from "./format";
import { createMockDatabase, makeGeneratedChapter, makeNewNovel, makeRetriedJob, makeRuntimeAgentRun } from "./mockData";
import { readServerSentEvents, type SseMessage } from "./sse";

const apiBaseUrl = import.meta.env.VITE_API_BASE_URL?.replace(/\/$/, "");
const useMock = import.meta.env.VITE_USE_MOCK !== "false" || !apiBaseUrl;
const sseEnabled = import.meta.env.VITE_ENABLE_SSE === "true";
const db = createMockDatabase();

export const apiConfig = {
  baseUrl: apiBaseUrl || null,
  mode: useMock ? "mock" : "real",
  sseEnabled,
  useMock,
  manualSaveEnabled: true,
} as const;

export class ApiError extends Error {
  constructor(
    message: string,
    public readonly status?: number,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

interface CreateNovelResponse {
  novel: Novel;
  bible?: NovelBible | null;
  characters?: NovelDetail["characters"];
  outlines?: ChapterOutline[];
}

interface NovelDetailResponse {
  novel: Novel;
  bible?: NovelBible | null;
  characters?: NovelDetail["characters"];
  chapters?: Chapter[];
  world_setting?: WorldSetting | null;
  facts?: NovelDetail["facts"];
}

interface BibleResponse {
  bible?: NovelBible | null;
}

interface CharactersResponse {
  characters?: NovelDetail["characters"];
}

interface WorldSettingResponse {
  world_setting?: WorldSetting | null;
}

interface FactsResponse {
  facts?: NovelDetail["facts"];
}

interface OutlineResponse {
  outlines?: ChapterOutline[];
}

interface ChapterVersionsResponse {
  novel_id: string;
  chapter_id: string;
  chapter_index: number;
  versions: number[];
}

interface ChapterVersionResponse {
  novel_id: string;
  chapter_id: string;
  chapter_index: number;
  version: number;
  content: string;
}

interface ContinuityResponse {
  chapter: Chapter;
  report: ContinuityReport | null;
}

interface ExportMarkdownResponse {
  filename: string;
  markdown: string;
}

interface AgentRunsResponse {
  runs: Array<Omit<AgentRun, "provider" | "duration_ms" | "output_summary" | "model" | "reasoning_effort"> & {
    attempt?: number | null;
    duration_ms?: number | null;
    output_summary?: string;
    provider?: string | null;
    model?: string | null;
    reasoning_effort?: string | null;
    total_tokens?: number | null;
  }>;
  summary?: AgentRunStatusSummary;
}

type ChapterJobKind = Extract<ApiJobKind, "write_chapter" | "review_chapter" | "rewrite_chapter">;

export interface AgentRunListOptions {
  novelId?: string;
  limit?: number;
  role?: AgentRole | "all";
  task?: AgentTask | "all";
  status?: AgentRunStatus | "all";
}

interface NormalizedAgentRunListOptions {
  novelId?: string;
  limit: number;
  role?: AgentRole;
  task?: AgentTask;
  status?: AgentRunStatus;
}

function agentRunListOptions(options: string | AgentRunListOptions = {}): AgentRunListOptions {
  return typeof options === "string" ? { novelId: options } : options;
}

function normalizeAgentRunListOptions(options: string | AgentRunListOptions = {}): NormalizedAgentRunListOptions {
  const input = agentRunListOptions(options);
  const normalized: NormalizedAgentRunListOptions = {
    limit: Math.max(1, Math.round(input.limit ?? 50)),
  };
  const novelId = input.novelId?.trim();
  if (novelId) {
    normalized.novelId = novelId;
  }
  if (input.role && input.role !== "all") {
    normalized.role = input.role;
  }
  if (input.task && input.task !== "all") {
    normalized.task = input.task;
  }
  if (input.status && input.status !== "all") {
    normalized.status = input.status;
  }
  return normalized;
}

export interface JobListOptions {
  limit?: number;
  status?: ApiJobStatus | "all";
  kind?: ApiJobKind | "all";
  novelId?: string;
  sourceJobId?: string;
}

interface NormalizedJobListOptions {
  limit: number;
  status?: ApiJobStatus;
  kind?: ApiJobKind;
  novelId?: string;
  sourceJobId?: string;
}

function normalizeJobListOptions(options: JobListOptions = {}): NormalizedJobListOptions {
  const normalized: NormalizedJobListOptions = {
    limit: Math.max(1, Math.round(options.limit ?? 50)),
  };
  if (options.status && options.status !== "all") {
    normalized.status = options.status;
  }
  if (options.kind && options.kind !== "all") {
    normalized.kind = options.kind;
  }
  const novelId = options.novelId?.trim();
  if (novelId) {
    normalized.novelId = novelId;
  }
  const sourceJobId = options.sourceJobId?.trim();
  if (sourceJobId) {
    normalized.sourceJobId = sourceJobId;
  }
  return normalized;
}

export const queryKeys = {
  health: ["health"] as const,
  novels: ["novels"] as const,
  novel: (novelId: string) => ["novel", novelId] as const,
  chapters: (novelId: string) => ["chapters", novelId] as const,
  chapter: (novelId: string, chapterIndex: number) => ["chapter", novelId, chapterIndex] as const,
  versions: (chapterId: string) => ["versions", chapterId] as const,
  review: (chapterId: string) => ["review", chapterId] as const,
  continuity: (novelId: string, chapterIndex: number) => ["chapter-continuity", novelId, chapterIndex] as const,
  agentRunsRoot: ["agent-runs"] as const,
  agentRuns: (options: string | AgentRunListOptions = {}) => ["agent-runs", normalizeAgentRunListOptions(options)] as const,
  agentRun: (runId: string) => ["agent-run", runId] as const,
  jobsRoot: ["jobs"] as const,
  jobs: (options: JobListOptions = {}) => ["jobs", normalizeJobListOptions(options)] as const,
  job: (jobId: string) => ["job", jobId] as const,
};

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${apiBaseUrl}${path}`, {
    headers: {
      "Content-Type": "application/json",
      ...init?.headers,
    },
    ...init,
  });
  if (!response.ok) {
    let message = response.statusText;
    try {
      const payload = (await response.json()) as { error?: string | { message?: string }; message?: string };
      message = typeof payload.error === "string" ? payload.error : (payload.error?.message ?? payload.message ?? message);
    } catch {
      // Keep status text when the backend does not return JSON.
    }
    throw new ApiError(message, response.status);
  }
  return response.json() as Promise<T>;
}

async function requestIfAvailable<T>(path: string): Promise<T | null> {
  try {
    return await request<T>(path);
  } catch (error) {
    if (error instanceof ApiError && (error.status === 404 || error.status === 405)) {
      return null;
    }
    throw error;
  }
}

function clone<T>(value: T): T {
  return structuredClone(value);
}

function sleep(ms = 220): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

function findChapter(novelId: string, chapterIndex: number): Chapter {
  const chapter = db.chapters[novelId]?.find((item) => item.chapter_index === chapterIndex);
  if (!chapter) {
    throw new ApiError(`Chapter ${chapterIndex} not found`, 404);
  }
  return chapter;
}

function pushVersion(chapter: Chapter, source: ChapterVersion["data"]["source"], notes: string): ChapterVersion {
  const list = db.versions[chapter.id] ?? [];
  const version: ChapterVersion = {
    id: `version-${chapter.id}-${chapter.version}`,
    chapter_id: chapter.id,
    version: chapter.version,
    title: chapter.title,
    content: chapter.content ?? "",
    summary: chapter.summary ?? "",
    word_count: chapter.word_count,
    data: {
      source,
      score: chapter.score,
      notes,
    },
    created_at: new Date().toISOString(),
  };
  db.versions[chapter.id] = [...list.filter((item) => item.version !== chapter.version), version].sort(
    (a, b) => a.version - b.version,
  );
  return version;
}

function appendRun(run: AgentRun): void {
  db.agentRuns = [run, ...db.agentRuns].slice(0, 80);
}

function updateNovelTimestamp(novelId: string): void {
  const novel = db.novels.find((item) => item.id === novelId);
  if (novel) {
    novel.updated_at = new Date().toISOString();
    if (novel.status === "draft") {
      novel.status = "active";
    }
  }
}

function emptyBible(novel: Novel): NovelBible {
  return {
    novel_id: novel.id,
    title_candidates: [{ title: novel.title, reason: "真实 API 未返回完整 Bible，前端保留占位展示。" }],
    premise: "",
    genre: novel.genre,
    target_platform: novel.target_platform,
    target_readers: "",
    core_selling_points: [],
    reader_expectations: [],
    main_conflict: "",
    protagonist_goal: "",
    emotional_value: "",
    tone: "",
    platform_tags: [],
    world_rules: [],
    constraints: [],
    opening_strategy: {
      first_scene: "",
      first_conflict: "",
      first_three_chapters_goal: "",
    },
  };
}

function emptyWorldSetting(): WorldSetting {
  return {
    genre_type: "",
    overview: "",
    power_system: { name: "", levels: [], rules: [], costs: [], limits: [] },
    organizations: [],
    locations: [],
    taboos: [],
    hard_rules: [],
  };
}

function outlineFromChapter(chapter: Chapter): ChapterOutline {
  return {
    novel_id: chapter.novel_id,
    volume_index: chapter.volume_index,
    chapter_index: chapter.chapter_index,
    title: chapter.title,
    pov: "第三人称限知",
    goal: chapter.outline,
    conflict: "",
    key_events: [],
    character_changes: [],
    new_facts: [],
    payoff: "",
    foreshadowing: [],
    cliffhanger: "",
    estimated_word_count: chapter.word_count || 2600,
  };
}

function normalizeAgentRun(run: AgentRunsResponse["runs"][number]): AgentRun {
  return {
    ...run,
    provider: run.provider ?? "unknown",
    model: run.model ?? null,
    reasoning_effort: run.reasoning_effort ?? null,
    attempt: run.attempt ?? null,
    duration_ms: run.duration_ms ?? 0,
    total_tokens: run.total_tokens ?? null,
    output_summary:
      run.output_summary ??
      (run.parse_error ? `运行失败：${run.parse_error}` : `${run.role} / ${run.task} 已记录。`),
  };
}

function emptyAgentRunSummary(): AgentRunStatusSummary {
  return {
    total: 0,
    ok: 0,
    fallback: 0,
    parse_error: 0,
    duration_ms_total: 0,
    tokenized_runs: 0,
    prompt_tokens: 0,
    completion_tokens: 0,
    total_tokens: 0,
  };
}

function summarizeAgentRuns(runs: AgentRun[]): AgentRunStatusSummary {
  return runs.reduce((summary, run) => {
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
  }, emptyAgentRunSummary());
}

function normalizeAgentRunReport(payload: AgentRunsResponse, filters: NormalizedAgentRunListOptions): AgentRunReport {
  const runs = payload.runs
    .map(normalizeAgentRun)
    .filter((run) => matchesAgentRunFilters(run, filters))
    .slice(0, filters.limit);
  const shouldRecomputeSummary = filters.status === "running" || runs.length !== payload.runs.length;
  return {
    runs,
    summary: shouldRecomputeSummary ? summarizeAgentRuns(runs) : (payload.summary ?? summarizeAgentRuns(runs)),
  };
}

function agentRunParams(filters: NormalizedAgentRunListOptions, limit = filters.limit, includeNovelId = false): string {
  const params = new URLSearchParams({ limit: String(limit) });
  if (includeNovelId && filters.novelId) {
    params.set("novel_id", filters.novelId);
  }
  if (filters.role) {
    params.set("role", filters.role);
  }
  if (filters.task) {
    params.set("task", filters.task);
  }
  if (filters.status && filters.status !== "running") {
    params.set("status", filters.status);
  }
  return params.toString();
}

function matchesAgentRunFilters(run: AgentRun, filters: NormalizedAgentRunListOptions): boolean {
  if (filters.novelId && run.novel_id !== filters.novelId) {
    return false;
  }
  if (filters.role && run.role !== filters.role) {
    return false;
  }
  if (filters.task && run.task !== filters.task) {
    return false;
  }
  if (filters.status && run.status !== filters.status) {
    return false;
  }
  return true;
}

function createNovelRequest(input: CreateNovelInput): Record<string, unknown> {
  return {
    idea: input.idea,
    platform: input.target_platform,
    chapters: Math.max(1, Math.round(input.target_words / Math.max(1, input.chapter_words))),
    outline_batch_size: 5,
  };
}

function chapterJobOperation(kind: ChapterJobKind): "write" | "review" | "rewrite" {
  if (kind === "review_chapter") {
    return "review";
  }
  if (kind === "rewrite_chapter") {
    return "rewrite";
  }
  return "write";
}

function contiguousRanges(chapterIndexes: number[]): number[][] {
  const sorted = [...new Set(chapterIndexes)].sort((a, b) => a - b);
  const ranges: number[][] = [];
  for (const chapterIndex of sorted) {
    const current = ranges[ranges.length - 1];
    if (!current || current[current.length - 1] + 1 !== chapterIndex) {
      ranges.push([chapterIndex]);
    } else {
      current.push(chapterIndex);
    }
  }
  return ranges;
}

async function requestChapter(novelId: string, chapterIndex: number): Promise<Chapter> {
  const payload = await request<{ chapter: Chapter }>(`/api/novels/${novelId}/chapters/${chapterIndex}`);
  return payload.chapter;
}

function normalizeContinuityReport(report: ContinuityReport | null | undefined): ContinuityReport | null {
  if (!report) {
    return null;
  }
  return {
    ...report,
    passed: report.passed !== false,
    issues: Array.isArray(report.issues) ? report.issues : [],
    new_facts: Array.isArray(report.new_facts) ? report.new_facts : [],
    character_state_updates: Array.isArray(report.character_state_updates) ? report.character_state_updates : [],
    foreshadowing_updates: Array.isArray(report.foreshadowing_updates) ? report.foreshadowing_updates : [],
  };
}

function makeMockContinuityReport(novelId: string, chapter: Chapter): ContinuityReport | null {
  if (!chapter.content) {
    return null;
  }
  const newFacts = (db.facts[novelId] ?? [])
    .filter((fact) => fact.chapter_id === chapter.id)
    .map(({ subject, predicate, object, importance }) => ({ subject, predicate, object, importance }));
  return normalizeContinuityReport({
    passed: true,
    issues: [],
    new_facts: newFacts,
    character_state_updates: [
      {
        character: "主角",
        state: chapter.summary ?? "本章行动已写入最新草稿。",
        reason: `第 ${chapter.chapter_index} 章连续性 mock 检查通过。`,
      },
    ],
    foreshadowing_updates: [
      {
        seed: chapter.title,
        status: "advanced",
        note: chapter.summary ?? "本章伏笔状态随正文推进。",
      },
    ],
    raw_notes: "Mock 连续性报告，真实模式会读取后端最新 Continuity Agent 结果。",
  });
}

function makeQueuedMockChapterJob(kind: ChapterJobKind, novelId: string, chapterIndex: number): ApiJob {
  const now = new Date().toISOString();
  return {
    id: `job-${crypto.randomUUID()}`,
    kind,
    status: "queued",
    novel_id: novelId,
    chapter_index: chapterIndex,
    source_job_id: null,
    progress_current: 0,
    progress_total: 1,
    payload: {
      novel_id: novelId,
      chapter_index: chapterIndex,
    },
    result: null,
    error: null,
    created_at: now,
    updated_at: now,
  };
}

function makeQueuedMockBatchWriteJob(novelId: string, chapterIndexes: number[]): ApiJob {
  const now = new Date().toISOString();
  const sorted = [...new Set(chapterIndexes)].sort((a, b) => a - b);
  return {
    id: `job-${crypto.randomUUID()}`,
    kind: "write_chapters",
    status: "queued",
    novel_id: novelId,
    chapter_index: null,
    source_job_id: null,
    progress_current: 0,
    progress_total: sorted.length,
    payload: {
      novel_id: novelId,
      chapter_start: sorted[0] ?? 0,
      chapter_end: sorted[sorted.length - 1] ?? 0,
      chapter_indexes: sorted,
    },
    result: null,
    error: null,
    created_at: now,
    updated_at: now,
  };
}

function makeQueuedMockCreateNovelJob(input: CreateNovelInput): ApiJob {
  const now = new Date().toISOString();
  return {
    id: `job-${crypto.randomUUID()}`,
    kind: "create_novel",
    status: "queued",
    novel_id: null,
    chapter_index: null,
    source_job_id: null,
    progress_current: 0,
    progress_total: 1,
    payload: createNovelRequest(input),
    result: null,
    error: null,
    created_at: now,
    updated_at: now,
  };
}

function updateMockJob(jobId: string, patch: Partial<ApiJob>): void {
  db.jobs = db.jobs.map((job) => (job.id === jobId ? { ...job, ...patch, updated_at: new Date().toISOString() } : job));
}

function getMockJob(jobId: string): ApiJob | undefined {
  return db.jobs.find((job) => job.id === jobId);
}

function settleRetriedMockJob(job: ApiJob): void {
  window.setTimeout(() => {
    if (getMockJob(job.id)?.status === "queued") {
      updateMockJob(job.id, { status: "running" });
    }
  }, 800);
  window.setTimeout(() => {
    if (getMockJob(job.id)?.status === "running") {
      updateMockJob(job.id, {
        status: "succeeded",
        progress_current: job.progress_total ?? 1,
        result: {
          retried_from: job.payload,
        },
      });
    }
  }, 2200);
}

function settleQueuedMockJob(job: ApiJob): void {
  window.setTimeout(() => {
    if (getMockJob(job.id)?.status === "queued") {
      updateMockJob(job.id, { status: "running" });
    }
  }, 700);
  window.setTimeout(() => {
    if (getMockJob(job.id)?.status === "running") {
      updateMockJob(job.id, {
        status: "succeeded",
        progress_current: 1,
        result: {
          job_kind: job.kind,
          chapter_index: job.chapter_index,
        },
      });
    }
  }, 2100);
}

function settleBatchWriteMockJob(job: ApiJob, chapterIndexes: number[]): void {
  window.setTimeout(() => {
    if (getMockJob(job.id)?.status === "queued") {
      updateMockJob(job.id, { status: "running" });
    }
  }, 600);

  chapterIndexes.forEach((chapterIndex, index) => {
    window.setTimeout(() => {
      if (getMockJob(job.id)?.status !== "running") {
        return;
      }
      updateMockJob(job.id, {
        progress_current: index + 1,
      });
      if (index + 1 === chapterIndexes.length) {
        updateMockJob(job.id, {
          status: "succeeded",
          result: {
            chapter_start: chapterIndexes[0],
            chapter_end: chapterIndexes[chapterIndexes.length - 1],
            drafts: chapterIndexes.map((item) => ({ chapter_index: item })),
          },
        });
      }
    }, 1200 + index * 520);
  });
}

function settleCreateNovelMockJob(job: ApiJob, input: CreateNovelInput): void {
  window.setTimeout(() => {
    if (getMockJob(job.id)?.status === "queued") {
      updateMockJob(job.id, { status: "running" });
    }
  }, 700);
  window.setTimeout(() => {
    if (getMockJob(job.id)?.status !== "running") {
      return;
    }
    const generated = makeNewNovel(input);
    db.novels = [generated.novel, ...db.novels];
    db.bibles[generated.novel.id] = generated.bible;
    db.characters[generated.novel.id] = generated.characters;
    db.worldSettings[generated.novel.id] = generated.worldSetting;
    db.outlines[generated.novel.id] = generated.outlines;
    db.chapters[generated.novel.id] = generated.chapters;
    db.facts[generated.novel.id] = generated.facts;
    generated.chapters.forEach((chapter) => {
      db.versions[chapter.id] = [];
    });
    updateMockJob(job.id, {
      status: "succeeded",
      progress_current: 1,
      result: {
        novel: generated.novel,
        used_fallback: false,
      },
    });
    appendRun(
      makeRuntimeAgentRun({
        novel_id: generated.novel.id,
        role: "market",
        task: "create_novel",
        output_summary: `后台创建《${generated.novel.title}》并生成基础素材。`,
      }),
    );
  }, 2200);
}

export const api = {
  getClientStatus(): ApiClientStatus {
    return {
      mode: apiConfig.mode,
      baseUrl: apiConfig.baseUrl,
      mockEnabled: apiConfig.useMock,
      sseEnabled: apiConfig.sseEnabled,
      sseReady: typeof ReadableStream !== "undefined" && typeof TextDecoder !== "undefined",
      manualSaveEnabled: apiConfig.manualSaveEnabled,
    };
  },

  async getHealth(): Promise<ApiHealthStatus> {
    if (!useMock) {
      const payload = await request<Partial<ApiHealthStatus> & { status: string }>("/health");
      return {
        status: payload.status,
        service: payload.service,
        version: payload.version,
        checked_at: payload.checked_at ?? new Date().toISOString(),
        sse: payload.sse,
      };
    }
    await sleep(80);
    return {
      status: "mock",
      service: "novel-agent",
      version: "mock",
      checked_at: new Date().toISOString(),
      sse: sseEnabled,
    };
  },

  async getNovels(): Promise<NovelListItem[]> {
    if (!useMock) {
      const payload = await request<{ novels: Novel[] }>("/api/novels?limit=50");
      return payload.novels.map((novel) => ({
        ...novel,
        chapter_count: 0,
        recent_score: null,
      }));
    }
    await sleep();
    return clone(
      db.novels.map((novel) => {
        const chapters = db.chapters[novel.id] ?? [];
        const scored = chapters.filter((chapter) => typeof chapter.score === "number");
        return {
          ...novel,
          chapter_count: chapters.length,
          recent_score: scored[0]?.score ?? null,
        };
      }),
    );
  },

  async createNovel(input: CreateNovelInput): Promise<Novel> {
    if (!useMock) {
      const payload = await request<CreateNovelResponse>("/api/novels", {
        method: "POST",
        body: JSON.stringify(createNovelRequest(input)),
      });
      return payload.novel;
    }
    await sleep(650);
    const generated = makeNewNovel(input);
    db.novels = [generated.novel, ...db.novels];
    db.bibles[generated.novel.id] = generated.bible;
    db.characters[generated.novel.id] = generated.characters;
    db.worldSettings[generated.novel.id] = generated.worldSetting;
    db.outlines[generated.novel.id] = generated.outlines;
    db.chapters[generated.novel.id] = generated.chapters;
    db.facts[generated.novel.id] = generated.facts;
    generated.chapters.forEach((chapter) => {
      db.versions[chapter.id] = [];
    });
    appendRun(
      makeRuntimeAgentRun({
        novel_id: generated.novel.id,
        role: "market",
        task: "create_novel",
        output_summary: `创建《${generated.novel.title}》并生成基础圣经与 30 章大纲。`,
      }),
    );
    return clone(generated.novel);
  },

  async createNovelJob(input: CreateNovelInput): Promise<ApiJob> {
    if (!useMock) {
      const payload = await request<{ job: ApiJob }>("/api/novels/jobs", {
        method: "POST",
        body: JSON.stringify(createNovelRequest(input)),
      });
      return payload.job;
    }
    await sleep(300);
    const job = makeQueuedMockCreateNovelJob(input);
    db.jobs = [job, ...db.jobs].slice(0, 80);
    settleCreateNovelMockJob(job, input);
    return clone(job);
  },

  async getNovel(novelId: string): Promise<NovelDetail> {
    if (!useMock) {
      const payload = await request<NovelDetailResponse>(`/api/novels/${novelId}`);
      const [biblePayload, charactersPayload, worldPayload, factsPayload, outlinePayload] = await Promise.all([
        requestIfAvailable<BibleResponse>(`/api/novels/${novelId}/bible`),
        requestIfAvailable<CharactersResponse>(`/api/novels/${novelId}/characters`),
        requestIfAvailable<WorldSettingResponse>(`/api/novels/${novelId}/world-settings`),
        requestIfAvailable<FactsResponse>(`/api/novels/${novelId}/facts?limit=100`),
        requestIfAvailable<OutlineResponse>(`/api/novels/${novelId}/outline`),
      ]);
      const chapters = payload.chapters ?? [];
      return {
        novel: payload.novel,
        bible: biblePayload?.bible ?? payload.bible ?? emptyBible(payload.novel),
        characters: charactersPayload?.characters ?? payload.characters ?? [],
        world_setting: worldPayload?.world_setting ?? payload.world_setting ?? emptyWorldSetting(),
        chapter_outlines: outlinePayload?.outlines ?? chapters.map(outlineFromChapter),
        facts: factsPayload?.facts ?? payload.facts ?? [],
      };
    }
    await sleep();
    const novel = db.novels.find((item) => item.id === novelId);
    if (!novel) {
      throw new ApiError("Novel not found", 404);
    }
    return clone({
      novel,
      bible: db.bibles[novelId],
      characters: db.characters[novelId] ?? [],
      world_setting: db.worldSettings[novelId],
      chapter_outlines: db.outlines[novelId] ?? [],
      facts: db.facts[novelId] ?? [],
    });
  },

  async getChapters(novelId: string): Promise<Chapter[]> {
    if (!useMock) {
      const payload = await request<{ chapters: Chapter[] }>(`/api/novels/${novelId}/chapters`);
      return payload.chapters;
    }
    await sleep();
    return clone(db.chapters[novelId] ?? []);
  },

  async getChapter(novelId: string, chapterIndex: number): Promise<Chapter> {
    if (!useMock) {
      return requestChapter(novelId, chapterIndex);
    }
    await sleep();
    return clone(findChapter(novelId, chapterIndex));
  },

  async writeChapter(novelId: string, chapterIndex: number): Promise<Chapter> {
    if (!useMock) {
      await request<{ draft: ChapterDraft }>(`/api/novels/${novelId}/chapters/${chapterIndex}/write`, { method: "POST" });
      return requestChapter(novelId, chapterIndex);
    }
    await sleep(900);
    const chapter = findChapter(novelId, chapterIndex);
    const content = makeGeneratedChapter(chapter);
    chapter.content = content;
    chapter.summary = `${chapter.title}已生成初稿，主线继续推进。`;
    chapter.status = "drafted";
    chapter.score = null;
    chapter.word_count = countWords(content);
    chapter.version += 1;
    chapter.updated_at = new Date().toISOString();
    pushVersion(chapter, "writer", "Writer mock 生成初稿。");
    appendRun(
      makeRuntimeAgentRun({
        novel_id: novelId,
        role: "writer",
        task: "generate_chapter",
        output_summary: `生成第 ${chapterIndex} 章初稿，${chapter.word_count} 字。`,
      }),
    );
    updateNovelTimestamp(novelId);
    return clone(chapter);
  },

  async writeChapters(
    novelId: string,
    chapterIndexes: number[],
    onProgress?: (completed: number, total: number, chapterIndex: number) => void,
  ): Promise<Chapter[]> {
    const results: Chapter[] = [];
    const uniqueIndexes = [...new Set(chapterIndexes)].sort((a, b) => a - b);
    for (const [index, chapterIndex] of uniqueIndexes.entries()) {
      const chapter = await this.writeChapter(novelId, chapterIndex);
      results.push(chapter);
      onProgress?.(index + 1, uniqueIndexes.length, chapterIndex);
    }
    return results;
  },

  async reviewChapter(novelId: string, chapterIndex: number): Promise<ReviewReport> {
    if (!useMock) {
      const payload = await request<{ report: ReviewReport }>(`/api/novels/${novelId}/chapters/${chapterIndex}/review`, { method: "POST" });
      return payload.report;
    }
    await sleep(760);
    const chapter = findChapter(novelId, chapterIndex);
    const hasStrongEnding = chapter.content?.includes("账本") || chapter.content?.includes("下一场冲突");
    const totalScore = hasStrongEnding ? 82 : 70;
    const report: ReviewReport = {
      id: `review-${chapter.id}-${Date.now()}`,
      chapter_id: chapter.id,
      total_score: totalScore,
      passed: totalScore >= 75,
      scores: {
        opening_hook_score: 8,
        pacing_score: totalScore >= 75 ? 8 : 6,
        payoff_score: totalScore >= 75 ? 8 : 6,
        character_score: 8,
        dialogue_score: 7,
        continuity_score: 9,
        cliffhanger_score: totalScore >= 75 ? 8 : 6,
        platform_fit_score: 8,
      },
      strengths: ["主角目标明确", "冲突能持续推动下一步", "事实没有明显矛盾"],
      issues:
        totalScore >= 75
          ? [
              {
                severity: "low",
                dimension: "dialogue",
                location: "中后段",
                description: "可增加一处短对白，让反派压力更直接。",
              },
            ]
          : [
              {
                severity: "high",
                dimension: "cliffhanger",
                location: "章尾",
                description: "章尾问题不够具体，下一章期待偏弱。",
              },
              {
                severity: "medium",
                dimension: "payoff",
                location: "中段",
                description: "行动后的即时反馈不足。",
              },
            ],
      suggestions:
        totalScore >= 75
          ? ["保留当前行动链，后续补强对手反应。"]
          : ["补强章尾明确问题。", "增加一个被迫让步或信息反转。"],
      rewrite_instruction: {
        needed: totalScore < 75,
        rewrite_type: totalScore < 75 ? "partial" : "none",
        priority: totalScore < 75 ? "high" : "low",
        goals: totalScore < 75 ? ["重写章尾", "补强即时回报"] : [],
        preserve: ["本章核心事实", "主角行动目标"],
        change: totalScore < 75 ? ["章尾钩子", "中段反馈"] : [],
        avoid: ["不要增加无行动的说明段"],
      },
      created_at: new Date().toISOString(),
    };
    chapter.status = report.passed ? "reviewed" : "rewrite_needed";
    chapter.score = report.total_score;
    chapter.updated_at = new Date().toISOString();
    db.reviews[chapter.id] = report;
    appendRun(
      makeRuntimeAgentRun({
        novel_id: novelId,
        role: "reviewer",
        task: "review_chapter",
        output_summary: `第 ${chapterIndex} 章评分 ${report.total_score}，${report.passed ? "通过" : "需要返工"}。`,
      }),
    );
    updateNovelTimestamp(novelId);
    return clone(report);
  },

  async reviewChapters(
    novelId: string,
    chapterIndexes: number[],
    onProgress?: (completed: number, total: number, chapterIndex: number) => void,
  ): Promise<ReviewReport[]> {
    const results: ReviewReport[] = [];
    const uniqueIndexes = [...new Set(chapterIndexes)].sort((a, b) => a - b);
    for (const [index, chapterIndex] of uniqueIndexes.entries()) {
      const report = await this.reviewChapter(novelId, chapterIndex);
      results.push(report);
      onProgress?.(index + 1, uniqueIndexes.length, chapterIndex);
    }
    return results;
  },

  async rewriteChapter(novelId: string, chapterIndex: number): Promise<Chapter> {
    if (!useMock) {
      await request<{ draft: ChapterDraft }>(`/api/novels/${novelId}/chapters/${chapterIndex}/rewrite`, { method: "POST" });
      return requestChapter(novelId, chapterIndex);
    }
    await sleep(980);
    const chapter = findChapter(novelId, chapterIndex);
    const addition = `\n\n---\n\n他没有急着庆祝，而是把下一步写在纸角：找到能证明对手说谎的人。门外的脚步声越来越近，新的选择已经没有退路。`;
    const content = chapter.content ? `${chapter.content}${addition}` : `${makeGeneratedChapter(chapter)}${addition}`;
    chapter.content = content;
    chapter.summary = "按审稿意见补强章尾和下一步行动。";
    chapter.status = "drafted";
    chapter.score = null;
    chapter.word_count = countWords(content);
    chapter.version += 1;
    chapter.updated_at = new Date().toISOString();
    pushVersion(chapter, "rewrite", "Reviewer 建议触发局部重写。");
    appendRun(
      makeRuntimeAgentRun({
        novel_id: novelId,
        role: "writer",
        task: "rewrite_chapter",
        output_summary: `重写第 ${chapterIndex} 章，新增章尾行动钩子。`,
      }),
    );
    updateNovelTimestamp(novelId);
    return clone(chapter);
  },

  async rewriteChapters(
    novelId: string,
    chapterIndexes: number[],
    onProgress?: (completed: number, total: number, chapterIndex: number) => void,
  ): Promise<Chapter[]> {
    const results: Chapter[] = [];
    const uniqueIndexes = [...new Set(chapterIndexes)].sort((a, b) => a - b);
    for (const [index, chapterIndex] of uniqueIndexes.entries()) {
      const chapter = await this.rewriteChapter(novelId, chapterIndex);
      results.push(chapter);
      onProgress?.(index + 1, uniqueIndexes.length, chapterIndex);
    }
    return results;
  },

  async saveChapterContent(novelId: string, chapterIndex: number, content: string): Promise<Chapter> {
    if (!useMock) {
      const saveRequest = {
        method: "PUT",
        body: JSON.stringify({ content }),
      };
      try {
        await request<{ draft: ChapterDraft }>(`/api/novels/${novelId}/chapters/${chapterIndex}/content`, saveRequest);
      } catch (error) {
        if (!(error instanceof ApiError) || (error.status !== 404 && error.status !== 405)) {
          throw error;
        }
        await request<{ draft: ChapterDraft }>(`/api/novels/${novelId}/chapters/${chapterIndex}/edit`, saveRequest);
      }
      return requestChapter(novelId, chapterIndex);
    }
    await sleep(420);
    const chapter = findChapter(novelId, chapterIndex);
    chapter.content = content;
    chapter.summary = "人工编辑稿已保存。";
    chapter.status = "drafted";
    chapter.score = null;
    chapter.word_count = countWords(content);
    chapter.version += 1;
    chapter.updated_at = new Date().toISOString();
    pushVersion(chapter, "manual_edit", "人工编辑保存。");
    updateNovelTimestamp(novelId);
    return clone(chapter);
  },

  async getChapterVersions(novelId: string, chapterIndex: number): Promise<ChapterVersion[]> {
    if (!useMock) {
      const [chapter, payload] = await Promise.all([
        requestChapter(novelId, chapterIndex),
        request<ChapterVersionsResponse>(`/api/novels/${novelId}/chapters/${chapterIndex}/versions`),
      ]);
      const versions = await Promise.all(
        payload.versions.map((version) =>
          request<ChapterVersionResponse>(`/api/novels/${novelId}/chapters/${chapterIndex}/versions/${version}`),
        ),
      );
      return versions.map((version) => ({
        id: `version-${payload.chapter_id}-${version.version}`,
        chapter_id: payload.chapter_id,
        version: version.version,
        title: chapter.title,
        content: version.content,
        summary: "",
        word_count: countWords(version.content),
        data: {
          source: "writer",
          notes: "API 版本正文。",
        },
        created_at: chapter.updated_at,
      }));
    }
    await sleep();
    const chapter = findChapter(novelId, chapterIndex);
    return clone(db.versions[chapter.id] ?? []);
  },

  async getReviewReport(novelId: string, chapterIndex: number): Promise<ReviewReport | null> {
    if (!useMock) {
      const payload = await request<{ report: ReviewReport | null }>(`/api/novels/${novelId}/chapters/${chapterIndex}/review`);
      return payload.report;
    }
    await sleep(120);
    const chapter = findChapter(novelId, chapterIndex);
    return clone(db.reviews[chapter.id] ?? null);
  },

  async getContinuityReport(novelId: string, chapterIndex: number): Promise<ContinuityReport | null> {
    if (!useMock) {
      const payload = await request<ContinuityResponse>(`/api/novels/${novelId}/chapters/${chapterIndex}/continuity`);
      return normalizeContinuityReport(payload.report);
    }
    await sleep(120);
    return clone(makeMockContinuityReport(novelId, findChapter(novelId, chapterIndex)));
  },

  async getAgentRunReport(options: string | AgentRunListOptions = {}): Promise<AgentRunReport> {
    const filters = normalizeAgentRunListOptions(options);
    if (!useMock) {
      const path = filters.novelId
        ? `/api/novels/${filters.novelId}/runs?${agentRunParams(filters)}`
        : `/api/runs?${agentRunParams(filters)}`;
      const payload = await request<AgentRunsResponse>(path);
      return normalizeAgentRunReport(payload, filters);
    }
    await sleep();
    const runs = db.agentRuns.filter((run) => matchesAgentRunFilters(run, filters)).slice(0, filters.limit);
    return clone({
      runs,
      summary: summarizeAgentRuns(runs),
    });
  },

  async getAgentRuns(options: string | AgentRunListOptions = {}): Promise<AgentRun[]> {
    const report = await this.getAgentRunReport(options);
    return report.runs;
  },

  async getAgentRun(runId: string): Promise<AgentRun> {
    if (!useMock) {
      const payload = await request<{ run: AgentRunsResponse["runs"][number] }>(`/api/runs/${runId}`);
      return normalizeAgentRun(payload.run);
    }
    await sleep(120);
    const run = db.agentRuns.find((item) => item.id === runId);
    if (!run) {
      throw new ApiError("Agent run not found", 404);
    }
    return clone(run);
  },

  async streamAgentRunReport(
    options: string | AgentRunListOptions = {},
    onSnapshot: (report: AgentRunReport) => void,
    signal?: AbortSignal,
  ): Promise<void> {
    const filters = normalizeAgentRunListOptions(options);
    if (useMock) {
      onSnapshot(await this.getAgentRunReport(options));
      return;
    }

    const response = await fetch(`${apiBaseUrl}/api/runs/stream?${agentRunParams(filters, filters.limit, true)}`, {
      headers: {
        Accept: "text/event-stream",
      },
      signal,
    });
    await readServerSentEvents<AgentRunsResponse | { total: number }>(response, (data, raw) => {
      if (raw.event === "snapshot") {
        onSnapshot(normalizeAgentRunReport(data as AgentRunsResponse, filters));
      }
    });
  },

  async createChapterJob(novelId: string, chapterIndex: number, kind: ChapterJobKind): Promise<ApiJob> {
    if (!useMock) {
      const operation = chapterJobOperation(kind);
      const payload = await request<{ job: ApiJob }>(`/api/novels/${novelId}/chapters/${chapterIndex}/${operation}/jobs`, {
        method: "POST",
      });
      return payload.job;
    }
    await sleep(260);
    const job = makeQueuedMockChapterJob(kind, novelId, chapterIndex);
    db.jobs = [job, ...db.jobs].slice(0, 80);
    settleQueuedMockJob(job);
    return clone(job);
  },

  async createBatchWriteJob(novelId: string, chapterIndexes: number[]): Promise<ApiJob> {
    const uniqueIndexes = [...new Set(chapterIndexes)].sort((a, b) => a - b);
    if (uniqueIndexes.length === 0) {
      throw new ApiError("No chapters selected", 400);
    }
    if (!useMock) {
      const payload = await request<{ job: ApiJob }>(`/api/novels/${novelId}/chapters/write/jobs`, {
        method: "POST",
        body: JSON.stringify({
          chapter_start: uniqueIndexes[0],
          chapter_end: uniqueIndexes[uniqueIndexes.length - 1],
        }),
      });
      return payload.job;
    }
    await sleep(260);
    const job = makeQueuedMockBatchWriteJob(novelId, uniqueIndexes);
    db.jobs = [job, ...db.jobs].slice(0, 80);
    settleBatchWriteMockJob(job, uniqueIndexes);
    return clone(job);
  },

  async createChapterJobs(
    novelId: string,
    chapterIndexes: number[],
    kind: ChapterJobKind,
    onProgress?: (completed: number, total: number, chapterIndex: number) => void,
  ): Promise<ApiJob[]> {
    if (kind === "write_chapter") {
      const ranges = contiguousRanges(chapterIndexes);
      const total = ranges.reduce((sum, range) => sum + range.length, 0);
      let completed = 0;
      const jobs: ApiJob[] = [];
      for (const range of ranges) {
        const job = await this.createBatchWriteJob(novelId, range);
        jobs.push(job);
        completed += range.length;
        onProgress?.(completed, total, range[range.length - 1]);
      }
      return jobs;
    }
    const results: ApiJob[] = [];
    const uniqueIndexes = [...new Set(chapterIndexes)].sort((a, b) => a - b);
    for (const [index, chapterIndex] of uniqueIndexes.entries()) {
      const job = await this.createChapterJob(novelId, chapterIndex, kind);
      results.push(job);
      onProgress?.(index + 1, uniqueIndexes.length, chapterIndex);
    }
    return results;
  },

  async getJobs(options: JobListOptions = {}): Promise<ApiJob[]> {
    const filters = normalizeJobListOptions(options);
    if (!useMock) {
      const params = new URLSearchParams({ limit: String(filters.limit) });
      if (filters.status) {
        params.set("status", filters.status);
      }
      if (filters.kind) {
        params.set("kind", filters.kind);
      }
      if (filters.novelId) {
        params.set("novel_id", filters.novelId);
      }
      if (filters.sourceJobId) {
        params.set("source_job_id", filters.sourceJobId);
      }
      const payload = await request<{ jobs: ApiJob[] }>(`/api/jobs?${params.toString()}`);
      return payload.jobs;
    }
    await sleep();
    return clone(
      db.jobs
        .filter((job) => {
          if (filters.status && job.status !== filters.status) {
            return false;
          }
          if (filters.kind && job.kind !== filters.kind) {
            return false;
          }
          if (filters.novelId && job.novel_id !== filters.novelId) {
            return false;
          }
          if (filters.sourceJobId && job.source_job_id !== filters.sourceJobId) {
            return false;
          }
          return true;
        })
        .slice(0, filters.limit),
    );
  },

  async getJob(jobId: string): Promise<ApiJob> {
    if (!useMock) {
      const payload = await request<{ job: ApiJob }>(`/api/jobs/${jobId}`);
      return payload.job;
    }
    await sleep(120);
    const job = db.jobs.find((item) => item.id === jobId);
    if (!job) {
      throw new ApiError("Job not found", 404);
    }
    return clone(job);
  },

  async retryJob(jobId: string): Promise<ApiJob> {
    if (!useMock) {
      const payload = await request<{ job: ApiJob }>(`/api/jobs/${jobId}/retry`, { method: "POST" });
      return payload.job;
    }
    await sleep(420);
    const source = db.jobs.find((item) => item.id === jobId);
    if (!source) {
      throw new ApiError("Job not found", 404);
    }
    if (source.status !== "failed") {
      throw new ApiError("Only failed jobs can be retried", 400);
    }
    const job = makeRetriedJob(source);
    db.jobs = [job, ...db.jobs];
    settleRetriedMockJob(job);
    return clone(job);
  },

  async cancelJob(jobId: string): Promise<ApiJob> {
    if (!useMock) {
      const payload = await request<{ job: ApiJob }>(`/api/jobs/${jobId}/cancel`, { method: "POST" });
      return payload.job;
    }
    await sleep(260);
    const source = db.jobs.find((item) => item.id === jobId);
    if (!source) {
      throw new ApiError("Job not found", 404);
    }
    if (source.status !== "queued" && source.status !== "running") {
      throw new ApiError("Only queued or running jobs can be cancelled", 400);
    }
    updateMockJob(jobId, {
      status: "cancelled",
      result: null,
      error: "用户从 Web 工作台取消任务。",
    });
    return clone(getMockJob(jobId) ?? source);
  },

  async exportMarkdown(novelId: string): Promise<string> {
    if (!useMock) {
      let payload: ExportMarkdownResponse;
      try {
        payload = await request<ExportMarkdownResponse>(`/api/novels/${novelId}/export`, { method: "POST" });
      } catch (error) {
        if (!(error instanceof ApiError) || (error.status !== 404 && error.status !== 405)) {
          throw error;
        }
        payload = await request<ExportMarkdownResponse>(`/api/novels/${novelId}/export/markdown`);
      }
      return payload.markdown;
    }
    await sleep(260);
    const novel = db.novels.find((item) => item.id === novelId);
    if (!novel) {
      throw new ApiError("Novel not found", 404);
    }
    const chapters = db.chapters[novelId] ?? [];
    return [
      `# ${novel.title}`,
      "",
      ...chapters
        .filter((chapter) => chapter.content)
        .flatMap((chapter) => [`## 第${chapter.chapter_index}章 ${chapter.title}`, "", chapter.content ?? "", ""]),
    ].join("\n");
  },

  async streamChapterOperation(
    novelId: string,
    chapterIndex: number,
    operation: ChapterOperation,
    onEvent: (event: ChapterStreamEvent) => void,
    signal?: AbortSignal,
  ): Promise<void> {
    if (useMock) {
      await mockChapterStream(operation, onEvent, signal);
      return;
    }

    const path = `/api/novels/${novelId}/chapters/${chapterIndex}/${operation}/stream`;
    const response = await fetch(`${apiBaseUrl}${path}`, {
      method: "POST",
      headers: {
        Accept: "text/event-stream",
        "Content-Type": "application/json",
      },
      signal,
    });
    await readServerSentEvents<Record<string, unknown>>(response, (data, raw) => {
      onEvent(streamMessageToChapterEvent(data, raw));
    });
  },
};

function streamMessageToChapterEvent(data: Record<string, unknown>, raw: SseMessage): ChapterStreamEvent {
  const now = new Date().toISOString();
  if (raw.event === "started") {
    return {
      event: "started",
      role: "writer",
      status: "running",
      message: `开始${data.operation === "rewrite" ? "重写" : "生成"}第 ${data.chapter_index ?? "-"} 章。`,
      data,
      created_at: now,
    };
  }
  if (raw.event === "chapter_chunk") {
    const chunkIndex = typeof data.chunk_index === "number" ? data.chunk_index + 1 : "-";
    return {
      event: "chapter_chunk",
      role: "writer",
      status: "running",
      message: `收到正文片段 ${chunkIndex}。`,
      data,
      created_at: now,
    };
  }
  if (raw.event === "completed") {
    return {
      event: "completed",
      role: "writer",
      status: "ok",
      message: data.operation === "rewrite" ? "重写完成。" : "章节生成完成。",
      data,
      created_at: now,
    };
  }
  return {
    event: "agent_delta",
    role: "writer",
    status: "running",
    message: raw.event,
    data,
    created_at: now,
  };
}

async function mockChapterStream(
  operation: ChapterOperation,
  onEvent: (event: ChapterStreamEvent) => void,
  signal?: AbortSignal,
): Promise<void> {
  const events = mockStreamEvents[operation];
  for (const event of events) {
    if (signal?.aborted) {
      return;
    }
    await sleep(event.delayMs);
    onEvent({
      event: event.event,
      role: event.role,
      message: event.message,
      status: event.event === "error" ? "parse_error" : "running",
      created_at: new Date().toISOString(),
      data: event.data,
    });
  }
}

const mockStreamEvents: Record<
  ChapterOperation,
  Array<{
    delayMs: number;
    event: ChapterStreamEvent["event"];
    role?: ChapterStreamEvent["role"];
    message: string;
    data?: Record<string, unknown>;
  }>
> = {
  write: [
    { delayMs: 80, event: "queued", message: "章节生成任务已排队。" },
    { delayMs: 180, event: "agent_started", role: "writer", message: "Writer 开始生成正文草稿。" },
    { delayMs: 240, event: "agent_completed", role: "writer", message: "Writer 已完成正文草稿。" },
    { delayMs: 180, event: "agent_completed", role: "continuity", message: "Continuity 已完成连续性检查。" },
    { delayMs: 180, event: "artifact_saved", role: "style", message: "Style 已完成润色并保存章节版本。" },
    { delayMs: 80, event: "completed", message: "章节生成完成。" },
  ],
  review: [
    { delayMs: 80, event: "queued", message: "审稿任务已排队。" },
    { delayMs: 180, event: "agent_started", role: "reviewer", message: "Reviewer 开始评分。" },
    { delayMs: 260, event: "agent_completed", role: "reviewer", message: "Reviewer 已输出评分和修改建议。" },
    { delayMs: 80, event: "completed", message: "审稿完成。" },
  ],
  rewrite: [
    { delayMs: 80, event: "queued", message: "重写任务已排队。" },
    { delayMs: 160, event: "agent_started", role: "reviewer", message: "读取最近审稿意见。" },
    { delayMs: 240, event: "agent_started", role: "writer", message: "Writer 按返工目标重写薄弱段落。" },
    { delayMs: 220, event: "artifact_saved", role: "style", message: "Style 已统一语气并保存新版本。" },
    { delayMs: 80, event: "completed", message: "重写完成。" },
  ],
};
