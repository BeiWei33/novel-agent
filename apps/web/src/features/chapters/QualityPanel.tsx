import { AlertTriangle, CheckCircle2, Gauge, ListChecks, ServerCog } from "lucide-react";
import type { AgentRun, Chapter, ContinuityReport, ReviewReport } from "../../types/domain";
import { Badge } from "../../components/ui/Badge";
import { LoadingState } from "../../components/LoadingState";
import { agentRoleLabels, agentTaskLabels, formatDateTime } from "../../lib/format";

const scoreLabels: Record<keyof ReviewReport["scores"], string> = {
  opening_hook_score: "开篇钩子",
  pacing_score: "节奏",
  payoff_score: "回报",
  character_score: "人物",
  dialogue_score: "对白",
  continuity_score: "连续性",
  cliffhanger_score: "章尾",
  platform_fit_score: "平台",
};

const rewriteTypeLabels: Record<string, string> = {
  none: "无需返工",
  partial: "局部返工",
  full: "整章重写",
  opening: "开头重写",
  ending: "结尾重写",
  style: "语言润色",
};

const priorityLabels: Record<string, string> = {
  high: "高优先级",
  medium: "中优先级",
  low: "低优先级",
};

const qualityTasks = new Set<AgentRun["task"]>(["generate_chapter", "review_chapter", "rewrite_chapter", "check_continuity", "polish_style"]);

export function QualityPanel({
  chapter,
  report,
  reviewLoading,
  continuityReport,
  continuityLoading,
  runs,
}: {
  chapter: Chapter;
  report?: ReviewReport | null;
  reviewLoading?: boolean;
  continuityReport?: ContinuityReport | null;
  continuityLoading?: boolean;
  runs: AgentRun[];
}) {
  const sortedRuns = [...runs].sort((left, right) => Date.parse(right.created_at) - Date.parse(left.created_at));
  const qualityRuns = sortedRuns.filter((run) => qualityTasks.has(run.task));
  const latestRun = qualityRuns[0] ?? sortedRuns[0] ?? null;
  const badRuns = sortedRuns.filter((run) => run.status === "fallback" || run.status === "parse_error" || run.parse_error).slice(0, 3);
  const hints = qualityHints(report, continuityReport, badRuns);

  if (reviewLoading) {
    return <LoadingState label="读取质量视图" />;
  }

  return (
    <div className="space-y-4 p-3">
      <QualitySummary chapter={chapter} report={report} />
      <QualitySignalCard hints={hints} badRuns={badRuns} />
      {report ? (
        <>
          <ScoreEvidence scores={report.scores} />
          <RewriteEvidence report={report} />
        </>
      ) : (
        <EmptyPanel text="本章还没有 ReviewReport。运行审稿后会显示评分、通过线和返工目标。" />
      )}
      <ContinuityEvidence report={continuityReport} isLoading={continuityLoading} />
      <RunEvidence run={latestRun} badRuns={badRuns} />
    </div>
  );
}

function QualitySummary({ chapter, report }: { chapter: Chapter; report?: ReviewReport | null }) {
  const score = report?.total_score ?? chapter.score ?? null;
  const passed = report?.passed ?? (typeof score === "number" ? score >= 75 : null);
  const scoreClass = typeof score === "number" && score < 75 ? "text-rose-700" : "text-teal-700";
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 flex items-start justify-between gap-3">
        <div>
          <div className="flex items-center gap-2 text-sm font-semibold">
            <Gauge className="h-4 w-4 text-accent" />
            章节质量
          </div>
          <div className={`mt-2 text-3xl font-semibold tabular-nums ${scoreClass}`}>
            {score ?? "-"}
          </div>
        </div>
        <Badge tone={passed === false ? "rose" : passed === true ? "teal" : "slate"}>{passed === null ? "待审稿" : passed ? "通过" : "需返工"}</Badge>
      </div>
      <div className="grid gap-2 text-xs leading-5 text-slate-600">
        <div>{"通过线：总分 >= 75，节奏 >= 7，连续性 >= 8，章尾钩子 >= 7"}</div>
        <div>章节状态：v{chapter.version} / {chapter.word_count} 字</div>
        {report ? <div>审稿时间：{formatDateTime(report.created_at)}</div> : null}
      </div>
    </div>
  );
}

function QualitySignalCard({ hints, badRuns }: { hints: string[]; badRuns: AgentRun[] }) {
  const hasRisk = hints.length > 0 || badRuns.length > 0;
  return (
    <div className={`rounded-md border p-4 shadow-soft ${hasRisk ? "border-amber-200 bg-amber-50" : "border-teal-200 bg-teal-50"}`}>
      <div className="mb-3 flex items-center gap-2 text-sm font-semibold">
        {hasRisk ? <AlertTriangle className="h-4 w-4 text-amber-700" /> : <CheckCircle2 className="h-4 w-4 text-teal-700" />}
        质量信号
      </div>
      {hasRisk ? (
        <ul className="space-y-2 text-sm leading-6 text-slate-700">
          {hints.map((hint) => (
            <li key={hint}>{hint}</li>
          ))}
        </ul>
      ) : (
        <p className="text-sm leading-6 text-teal-800">暂未发现 fallback、parse_error 或明显质量退化信号。</p>
      )}
    </div>
  );
}

function ScoreEvidence({ scores }: { scores: ReviewReport["scores"] }) {
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 flex items-center gap-2 text-sm font-semibold">
        <ListChecks className="h-4 w-4 text-accent" />
        分项分数
      </div>
      <div className="grid grid-cols-2 gap-2">
        {Object.entries(scores).map(([key, value]) => (
          <div key={key} className="rounded-md bg-slate-50 p-2">
            <div className="mb-1 flex items-center justify-between gap-2">
              <span className="text-xs text-slate-500">{scoreLabels[key as keyof ReviewReport["scores"]]}</span>
              <Badge tone={scoreTone(value)} className="min-h-5 px-1.5">
                {value}
              </Badge>
            </div>
            <div className="h-1.5 overflow-hidden rounded-full bg-slate-200">
              <div className={`h-full rounded-full ${scoreBarClass(value)}`} style={{ width: `${Math.min(100, value * 10)}%` }} />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function RewriteEvidence({ report }: { report: ReviewReport }) {
  const instruction = report.rewrite_instruction;
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <div className="text-sm font-semibold">返工目标</div>
        <div className="flex flex-wrap gap-2">
          <Badge tone={instruction.needed ? "amber" : "teal"}>{rewriteTypeLabels[instruction.rewrite_type] ?? instruction.rewrite_type}</Badge>
          <Badge tone={priorityTone(instruction.priority)}>{priorityLabels[instruction.priority] ?? instruction.priority}</Badge>
        </div>
      </div>
      <CompactList values={instruction.goals} emptyText={instruction.needed ? "暂无明确目标。" : "当前章节达到通过线。"} />
      {instruction.change.length > 0 ? <CompactList title="必须修改" values={instruction.change} /> : null}
      {instruction.preserve.length > 0 ? <CompactList title="必须保留" values={instruction.preserve} /> : null}
    </div>
  );
}

function ContinuityEvidence({ report, isLoading }: { report?: ContinuityReport | null; isLoading?: boolean }) {
  if (isLoading) {
    return (
      <div className="rounded-md border border-border bg-white shadow-soft">
        <LoadingState label="读取连续性证据" />
      </div>
    );
  }
  if (!report) {
    return <EmptyPanel text="本章还没有连续性检查结果。生成或重写章节后会显示事实、人物状态和伏笔更新。" />;
  }

  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 flex items-center justify-between gap-2">
        <div className="text-sm font-semibold">连续性 / 事实 / 伏笔</div>
        <Badge tone={report.passed ? "teal" : "rose"}>{report.passed ? "通过" : "需处理"}</Badge>
      </div>
      <div className="grid grid-cols-2 gap-2">
        <Metric label="问题" value={report.issues.length} tone={report.issues.length > 0 ? "rose" : "teal"} />
        <Metric label="新事实" value={report.new_facts.length} tone="blue" />
        <Metric label="人物状态" value={report.character_state_updates.length} tone="amber" />
        <Metric label="伏笔" value={report.foreshadowing_updates.length} tone="slate" />
      </div>
      <div className="mt-3 space-y-2 text-sm leading-6 text-slate-700">
        {report.new_facts.slice(0, 2).map((fact, index) => (
          <p key={`${fact.subject}-${fact.predicate}-${index}`}>
            {fact.subject} {fact.predicate} {fact.object}
          </p>
        ))}
        {report.foreshadowing_updates.slice(0, 2).map((item, index) => (
          <p key={`${item.seed ?? "foreshadowing"}-${index}`}>{item.seed ?? "伏笔"}：{item.note ?? item.status ?? "已更新"}</p>
        ))}
      </div>
    </div>
  );
}

function RunEvidence({ run, badRuns }: { run: AgentRun | null; badRuns: AgentRun[] }) {
  if (!run) {
    return <EmptyPanel text="暂无 AgentRun。真实 API 或 mock 运行后会显示 provider、model、reasoning 和 prompt bundle。" />;
  }

  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 flex items-center gap-2 text-sm font-semibold">
        <ServerCog className="h-4 w-4 text-accent" />
        AgentRun 证据
      </div>
      <dl className="grid grid-cols-2 gap-3 text-xs">
        <DetailItem label="provider" value={run.provider} />
        <DetailItem label="model" value={run.model ?? "-"} />
        <DetailItem label="reasoning" value={run.reasoning_effort ?? "-"} />
        <DetailItem label="prompt_bundle" value={promptBundleLabel(run)} />
        <DetailItem label="role/task" value={`${agentRoleLabels[run.role]} / ${agentTaskLabels[run.task]}`} />
        <DetailItem label="status" value={run.status} />
      </dl>
      <p className="mt-3 text-xs leading-5 text-slate-600">{run.output_summary}</p>
      {badRuns.length > 0 ? (
        <div className="mt-3 rounded-md border border-rose-200 bg-rose-50 p-3">
          <div className="mb-2 text-xs font-semibold text-rose-800">fallback / parse_error</div>
          <div className="space-y-2">
            {badRuns.map((item) => (
              <div key={item.id} className="text-xs leading-5 text-rose-800">
                {formatDateTime(item.created_at)} {item.status}：{item.parse_error ?? item.output_summary}
              </div>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
}

function Metric({
  label,
  value,
  tone,
}: {
  label: string;
  value: number;
  tone: "teal" | "blue" | "amber" | "rose" | "slate";
}) {
  return (
    <div className="rounded-md bg-slate-50 p-2">
      <div className="text-xs text-slate-500">{label}</div>
      <Badge tone={tone} className="mt-1 tabular-nums">
        {value}
      </Badge>
    </div>
  );
}

function DetailItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0">
      <dt className="text-slate-500">{label}</dt>
      <dd className="mt-1 truncate font-medium text-slate-800" title={value}>{value}</dd>
    </div>
  );
}

function CompactList({ title, values, emptyText }: { title?: string; values: string[]; emptyText?: string }) {
  if (values.length === 0 && !emptyText) {
    return null;
  }
  return (
    <div className="mb-3 last:mb-0">
      {title ? <div className="mb-1 text-xs font-semibold text-slate-600">{title}</div> : null}
      {values.length === 0 ? (
        <p className="text-sm leading-6 text-slate-500">{emptyText}</p>
      ) : (
        <ul className="space-y-1 text-sm leading-6 text-slate-700">
          {values.map((value) => (
            <li key={value}>{value}</li>
          ))}
        </ul>
      )}
    </div>
  );
}

function EmptyPanel({ text }: { text: string }) {
  return <div className="rounded-md border border-dashed border-border p-4 text-sm leading-6 text-slate-500">{text}</div>;
}

function qualityHints(report: ReviewReport | null | undefined, continuityReport: ContinuityReport | null | undefined, badRuns: AgentRun[]): string[] {
  const hints: string[] = [];
  if (badRuns.length > 0) {
    hints.push(`发现 ${badRuns.length} 条 fallback / parse_error 运行记录，需要复核生成质量。`);
  }
  if (!report) {
    return hints;
  }
  if (!report.passed) {
    hints.push("ReviewReport 未通过当前平台通过线。");
  }
  Object.entries(report.scores)
    .filter(([, score]) => score < 7)
    .slice(0, 3)
    .forEach(([key, score]) => hints.push(`${scoreLabels[key as keyof ReviewReport["scores"]]} ${score} 分，低于 7 分守门线。`));
  if (report.rewrite_instruction.needed) {
    hints.push(`Reviewer 建议${rewriteTypeLabels[report.rewrite_instruction.rewrite_type] ?? "返工"}。`);
  }
  if (continuityReport && !continuityReport.passed) {
    hints.push("连续性检查未通过，优先处理人物、事实或伏笔冲突。");
  }
  return hints;
}

function scoreTone(score: number): "teal" | "blue" | "amber" | "rose" {
  if (score >= 9) {
    return "teal";
  }
  if (score >= 7) {
    return "blue";
  }
  if (score >= 5) {
    return "amber";
  }
  return "rose";
}

function scoreBarClass(score: number): string {
  if (score >= 9) {
    return "bg-teal-500";
  }
  if (score >= 7) {
    return "bg-sky-500";
  }
  if (score >= 5) {
    return "bg-amber-500";
  }
  return "bg-rose-500";
}

function priorityTone(priority: string): "rose" | "amber" | "slate" {
  if (priority === "high") {
    return "rose";
  }
  if (priority === "medium") {
    return "amber";
  }
  return "slate";
}

function promptBundleLabel(run: AgentRun): string {
  return (
    firstString(
      run.structured.prompt_bundle,
      readNestedString(run.structured, ["_engineering", "prompt_bundle"]),
      readNestedString(run.structured, ["metadata", "prompt_bundle"]),
      readNestedString(run.structured, ["run_metadata", "prompt_bundle"]),
      readPromptBundleFromNotes(run.raw_notes),
    ) ?? "未记录"
  );
}

function firstString(...values: Array<unknown>): string | null {
  for (const value of values) {
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }
  return null;
}

function readNestedString(value: Record<string, unknown>, path: string[]): string | null {
  let current: unknown = value;
  for (const key of path) {
    if (!current || typeof current !== "object" || !(key in current)) {
      return null;
    }
    current = (current as Record<string, unknown>)[key];
  }
  return typeof current === "string" ? current : null;
}

function readPromptBundleFromNotes(notes: string): string | null {
  return notes.match(/b-quality-[\w.-]+/)?.[0] ?? null;
}
