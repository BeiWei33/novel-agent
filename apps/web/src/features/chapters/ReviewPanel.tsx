import { SearchCheck, Wand2 } from "lucide-react";
import type { ContinuityReport, ReviewReport } from "../../types/domain";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { LoadingState } from "../../components/LoadingState";

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

const continuityIssueTypeLabels: Record<string, string> = {
  character: "人物",
  world: "世界",
  timeline: "时间线",
  fact: "事实",
  foreshadowing: "伏笔",
  other: "其他",
};

const foreshadowingStatusLabels: Record<string, string> = {
  planted: "已埋下",
  advanced: "推进",
  paid_off: "已回收",
  contradicted: "冲突",
};

export function ReviewPanel({
  report,
  isLoading,
  continuityReport,
  continuityLoading,
  onReview,
  onRewrite,
  reviewPending,
  rewritePending,
}: {
  report?: ReviewReport | null;
  isLoading?: boolean;
  continuityReport?: ContinuityReport | null;
  continuityLoading?: boolean;
  onReview: () => void;
  onRewrite: () => void;
  reviewPending?: boolean;
  rewritePending?: boolean;
}) {
  if (isLoading) {
    return <LoadingState label="读取审稿报告" />;
  }

  return (
    <div className="space-y-4 p-3">
      <div className="flex gap-2">
        <Button variant="primary" onClick={onReview} disabled={reviewPending}>
          <SearchCheck className="h-4 w-4" />
          {reviewPending ? "审稿中" : "审稿"}
        </Button>
        <Button variant="secondary" onClick={onRewrite} disabled={rewritePending || !report?.rewrite_instruction.needed}>
          <Wand2 className="h-4 w-4" />
          {rewritePending ? "重写中" : "重写"}
        </Button>
      </div>

      {!report ? (
        <div className="rounded-md border border-dashed border-border p-4 text-sm text-slate-500">暂无审稿报告</div>
      ) : (
        <>
          <div className="rounded-md border border-border bg-white p-4 shadow-soft">
            <div className="mb-3 flex items-center justify-between">
              <div className="text-sm font-semibold">总分</div>
              <Badge tone={report.passed ? "teal" : "rose"}>{report.total_score}</Badge>
            </div>
            <div className="grid grid-cols-2 gap-2">
              {Object.entries(report.scores).map(([key, value]) => (
                <div key={key} className="rounded-md bg-slate-50 p-2">
                  <div className="text-xs text-slate-500">{scoreLabels[key as keyof ReviewReport["scores"]]}</div>
                  <div className="mt-1 text-sm font-semibold tabular-nums">{value}</div>
                </div>
              ))}
            </div>
          </div>

          <PanelList title="优点" values={report.strengths} tone="teal" />
          <div className="rounded-md border border-border bg-white p-4 shadow-soft">
            <div className="mb-3 text-sm font-semibold">问题</div>
            <div className="space-y-3">
              {report.issues.map((issue) => (
                <div key={`${issue.dimension}-${issue.location}`} className="border-b border-line pb-3 last:border-0 last:pb-0">
                  <div className="mb-1 flex items-center gap-2">
                    <Badge tone={issue.severity === "high" ? "rose" : issue.severity === "medium" ? "amber" : "slate"}>
                      {issue.severity}
                    </Badge>
                    <span className="text-xs text-slate-500">{issue.location}</span>
                  </div>
                  <p className="text-sm leading-6 text-slate-700">{issue.description}</p>
                </div>
              ))}
            </div>
          </div>
          <PanelList title="修改建议" values={report.suggestions} tone="blue" />
          {report.rewrite_instruction.needed ? (
            <PanelList title="返工目标" values={report.rewrite_instruction.goals} tone="amber" />
          ) : null}
        </>
      )}

      <ContinuityCard report={continuityReport} isLoading={continuityLoading} />
    </div>
  );
}

function ContinuityCard({ report, isLoading }: { report?: ContinuityReport | null; isLoading?: boolean }) {
  if (isLoading) {
    return (
      <div className="rounded-md border border-border bg-white shadow-soft">
        <LoadingState label="读取连续性报告" />
      </div>
    );
  }

  if (!report) {
    return (
      <div className="rounded-md border border-dashed border-border p-4 text-sm text-slate-500">
        暂无连续性报告
      </div>
    );
  }

  return (
    <>
      <div className="rounded-md border border-border bg-white p-4 shadow-soft">
        <div className="mb-3 flex items-center justify-between gap-2">
          <div className="text-sm font-semibold">连续性报告</div>
          <Badge tone={report.passed ? "teal" : "rose"}>{report.passed ? "通过" : "需处理"}</Badge>
        </div>
        <div className="grid grid-cols-2 gap-2">
          <ContinuityMetric label="问题" value={report.issues.length} tone={report.issues.length > 0 ? "rose" : "teal"} />
          <ContinuityMetric label="新事实" value={report.new_facts.length} tone="blue" />
          <ContinuityMetric label="人物状态" value={report.character_state_updates.length} tone="amber" />
          <ContinuityMetric label="伏笔" value={report.foreshadowing_updates.length} tone="slate" />
        </div>
        {report.raw_notes ? <p className="mt-3 text-xs leading-5 text-slate-500">{report.raw_notes}</p> : null}
      </div>

      {report.issues.length > 0 ? <ContinuityIssues issues={report.issues} /> : <ContinuityEmptyIssues />}
      {report.new_facts.length > 0 ? <ContinuityFacts report={report} /> : null}
      {report.character_state_updates.length > 0 ? <CharacterStateUpdates report={report} /> : null}
      {report.foreshadowing_updates.length > 0 ? <ForeshadowingUpdates report={report} /> : null}
    </>
  );
}

function ContinuityMetric({
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

function ContinuityIssues({ issues }: { issues: ContinuityReport["issues"] }) {
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 text-sm font-semibold">连续性问题</div>
      <div className="space-y-3">
        {issues.map((issue, index) => {
          const type = issue.type ? (continuityIssueTypeLabels[issue.type] ?? issue.type) : "连续性";
          return (
            <div key={`${issue.type ?? "issue"}-${issue.location ?? index}`} className="border-b border-line pb-3 last:border-0 last:pb-0">
              <div className="mb-1 flex flex-wrap items-center gap-2">
                <Badge tone={severityTone(issue.severity)}>{issue.severity ?? "issue"}</Badge>
                <span className="text-xs text-slate-500">{type}</span>
                {issue.location ? <span className="text-xs text-slate-500">{issue.location}</span> : null}
              </div>
              <p className="text-sm leading-6 text-slate-700">{issue.description ?? compactJson(issue)}</p>
              {issue.suggestion ? <p className="mt-1 text-xs leading-5 text-slate-500">建议：{issue.suggestion}</p> : null}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function ContinuityEmptyIssues() {
  return (
    <div className="rounded-md border border-border bg-white p-4 text-sm text-slate-600 shadow-soft">
      未发现人物、事实、时间线或伏笔冲突。
    </div>
  );
}

function ContinuityFacts({ report }: { report: ContinuityReport }) {
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 text-sm font-semibold">确认事实</div>
      <div className="space-y-2">
        {report.new_facts.map((fact, index) => (
          <div key={`${fact.subject}-${fact.predicate}-${index}`} className="flex gap-2 text-sm leading-6 text-slate-700">
            <Badge tone="blue" className="mt-0.5 h-5 min-h-5 px-1.5">
              {fact.importance}
            </Badge>
            <span>
              {fact.subject} {fact.predicate} {fact.object}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

function CharacterStateUpdates({ report }: { report: ContinuityReport }) {
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 text-sm font-semibold">人物状态</div>
      <div className="space-y-3">
        {report.character_state_updates.map((update, index) => (
          <div key={`${update.character ?? "character"}-${index}`} className="border-b border-line pb-3 last:border-0 last:pb-0">
            <div className="mb-1 text-sm font-medium text-slate-700">{update.character ?? "角色"}</div>
            <p className="text-sm leading-6 text-slate-700">{characterStateText(update)}</p>
            {update.reason ? <p className="mt-1 text-xs leading-5 text-slate-500">{update.reason}</p> : null}
          </div>
        ))}
      </div>
    </div>
  );
}

function ForeshadowingUpdates({ report }: { report: ContinuityReport }) {
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 text-sm font-semibold">伏笔状态</div>
      <div className="space-y-3">
        {report.foreshadowing_updates.map((update, index) => (
          <div key={`${update.seed ?? "seed"}-${index}`} className="border-b border-line pb-3 last:border-0 last:pb-0">
            <div className="mb-1 flex flex-wrap items-center gap-2">
              <Badge tone={update.status === "contradicted" ? "rose" : update.status === "paid_off" ? "teal" : "slate"}>
                {update.status ? (foreshadowingStatusLabels[update.status] ?? update.status) : "伏笔"}
              </Badge>
              <span className="text-xs text-slate-500">{update.seed ?? "未命名伏笔"}</span>
            </div>
            {update.note ? <p className="text-sm leading-6 text-slate-700">{update.note}</p> : null}
          </div>
        ))}
      </div>
    </div>
  );
}

function PanelList({ title, values, tone }: { title: string; values: string[]; tone: "teal" | "blue" | "amber" }) {
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 text-sm font-semibold">{title}</div>
      <div className="space-y-2">
        {values.map((value) => (
          <div key={value} className="flex gap-2 text-sm leading-6 text-slate-700">
            <Badge tone={tone} className="mt-0.5 h-5 min-h-5 px-1.5">
              {title.slice(0, 1)}
            </Badge>
            <span>{value}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function severityTone(severity?: string): "rose" | "amber" | "slate" {
  if (severity === "high") {
    return "rose";
  }
  if (severity === "medium") {
    return "amber";
  }
  return "slate";
}

function characterStateText(update: ContinuityReport["character_state_updates"][number]): string {
  if (update.before || update.after) {
    return `${update.before ?? "-"} -> ${update.after ?? "-"}`;
  }
  if (update.state) {
    return update.state;
  }
  return compactJson(update);
}

function compactJson(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (value == null) {
    return "";
  }
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}
