import { useState } from "react";
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

const issueDimensionLabels: Record<ReviewReport["issues"][number]["dimension"], string> = {
  continuity: "连续性",
  pacing: "情节推进",
  cliffhanger: "章尾钩子",
  opening_hook: "开头吸引力",
  payoff: "爽点回报",
  character: "人物表现",
  dialogue: "对话自然度",
  platform_fit: "平台适配",
};

const severityLabels: Record<string, string> = {
  high: "高",
  medium: "中",
  low: "低",
};

const priorityLabels: Record<string, string> = {
  high: "高优先级",
  medium: "中优先级",
  low: "低优先级",
};

const rewriteTypeLabels: Record<string, string> = {
  none: "无需返工",
  partial: "局部返工",
  full: "整章重写",
  opening: "开头重写",
  ending: "结尾重写",
  style: "语言润色",
};

const rewriteActionLabels: Record<string, string> = {
  partial: "按建议局部重写",
  full: "整章重写",
  opening: "重写开头",
  ending: "重写结尾",
  style: "语言润色",
};

const issueSeverityOrder: Record<string, number> = {
  high: 0,
  medium: 1,
  low: 2,
};

const issueDimensionOrder: Record<string, number> = {
  continuity: 0,
  pacing: 1,
  cliffhanger: 2,
  opening_hook: 3,
  payoff: 4,
  character: 5,
  dialogue: 6,
  platform_fit: 7,
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
          {rewritePending ? "重写中" : rewriteActionLabel(report)}
        </Button>
      </div>

      {!report ? (
        <div className="rounded-md border border-dashed border-border p-4 text-sm text-slate-500">
          本章还没有审稿。生成正文后可以运行 Reviewer Agent。
        </div>
      ) : (
        <>
          <ReviewSummaryCard report={report} />
          <ScoreGrid scores={report.scores} />
          <RewriteInstructionCard report={report} />
          <ReviewIssues issues={report.issues} />
          <SuggestionChecklist suggestions={report.suggestions} />
          <PanelList title="优点" values={report.strengths} tone="teal" emptyText="暂无特别标记的优点。" />
        </>
      )}

      <ContinuityCard report={continuityReport} isLoading={continuityLoading} />
    </div>
  );
}

function ReviewSummaryCard({ report }: { report: ReviewReport }) {
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 flex items-start justify-between gap-3">
        <div>
          <div className="text-xs text-slate-500">总分</div>
          <div className={`mt-1 text-3xl font-semibold tabular-nums ${report.total_score < 75 ? "text-rose-700" : "text-teal-700"}`}>
            {report.total_score}
          </div>
        </div>
        <div className="flex flex-wrap justify-end gap-2">
          <Badge tone={report.passed ? "teal" : "rose"}>{report.passed ? "通过" : "需返工"}</Badge>
          <Badge tone={report.rewrite_instruction.needed ? "amber" : "teal"}>
            {report.rewrite_instruction.needed ? "建议返工" : "无需返工"}
          </Badge>
          <Badge tone={priorityTone(report.rewrite_instruction.priority)}>
            {priorityLabels[report.rewrite_instruction.priority] ?? report.rewrite_instruction.priority}
          </Badge>
        </div>
      </div>
      <div className="grid gap-2 text-xs leading-5 text-slate-600">
        <div>审稿时间：{new Date(report.created_at).toLocaleString("zh-CN")}</div>
        <div>{"通过线：总分 >= 75，节奏 >= 7，连续性 >= 8，章尾钩子 >= 7"}</div>
      </div>
    </div>
  );
}

function ScoreGrid({ scores }: { scores: ReviewReport["scores"] }) {
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 text-sm font-semibold">评分维度</div>
      <div className="grid grid-cols-2 gap-2">
        {Object.entries(scores).map(([key, value]) => (
          <div key={key} className="rounded-md bg-slate-50 p-2">
            <div className="flex items-center justify-between gap-2">
              <div className="text-xs text-slate-500">{scoreLabels[key as keyof ReviewReport["scores"]]}</div>
              <Badge tone={scoreTone(value)} className="min-h-5 px-1.5">
                {scoreLevel(value)}
              </Badge>
            </div>
            <div className="mt-1 text-sm font-semibold tabular-nums">{value}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

function RewriteInstructionCard({ report }: { report: ReviewReport }) {
  const instruction = report.rewrite_instruction;
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <div className="text-sm font-semibold">返工指令</div>
        <div className="flex flex-wrap gap-2">
          <Badge tone={instruction.needed ? "amber" : "teal"}>{rewriteTypeLabel(instruction.rewrite_type)}</Badge>
          <Badge tone={priorityTone(instruction.priority)}>{priorityLabels[instruction.priority] ?? instruction.priority}</Badge>
        </div>
      </div>
      <CompactList title="返工目标" values={instruction.goals} emptyText={instruction.needed ? "暂无明确返工目标。" : "本章达到当前连载通过线。"} />
      <CompactList title="必须保留" values={instruction.preserve} />
      <CompactList title="必须修改" values={instruction.change} />
      <CompactList title="避免事项" values={instruction.avoid} />
    </div>
  );
}

function ReviewIssues({ issues }: { issues: ReviewReport["issues"] }) {
  const sortedIssues = [...issues].sort(compareReviewIssues);
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 text-sm font-semibold">问题</div>
      {sortedIssues.length === 0 ? (
        <p className="text-sm leading-6 text-slate-500">暂无明确问题。本章达到当前连载通过线。</p>
      ) : (
        <div className="space-y-3">
          {sortedIssues.map((issue, index) => (
            <div key={`${issue.dimension}-${issue.location}-${index}`} className="border-b border-line pb-3 last:border-0 last:pb-0">
              <div className="mb-1 flex flex-wrap items-center gap-2">
                <Badge tone={severityTone(issue.severity)}>{severityLabels[issue.severity] ?? issue.severity}</Badge>
                <span className="text-xs text-slate-500">{issueDimensionLabels[issue.dimension] ?? issue.dimension}</span>
                <span className="text-xs text-slate-500">{issue.location || "整章"}</span>
              </div>
              <p className="text-sm leading-6 text-slate-700">{issue.description}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function SuggestionChecklist({ suggestions }: { suggestions: string[] }) {
  const [checked, setChecked] = useState<Record<string, boolean>>({});
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 text-sm font-semibold">修改建议</div>
      {suggestions.length === 0 ? (
        <p className="text-sm leading-6 text-slate-500">暂无修改建议。本章达到当前连载通过线。</p>
      ) : (
        <div className="space-y-2">
          {suggestions.map((suggestion, index) => {
            const key = `${index}-${suggestion}`;
            return (
              <label key={key} className="flex items-start gap-2 text-sm leading-6 text-slate-700">
                <input
                  type="checkbox"
                  checked={checked[key] ?? false}
                  onChange={(event) => setChecked((current) => ({ ...current, [key]: event.target.checked }))}
                  className="mt-1 h-4 w-4 rounded border-border text-accent"
                />
                <span className={checked[key] ? "text-slate-400 line-through" : undefined}>{suggestion}</span>
              </label>
            );
          })}
        </div>
      )}
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
        本章还没有连续性检查结果。生成或重写章节后会自动检查。
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
                <Badge tone={severityTone(issue.severity)}>{issue.severity ? (severityLabels[issue.severity] ?? issue.severity) : "问题"}</Badge>
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

function PanelList({
  title,
  values,
  tone,
  emptyText,
}: {
  title: string;
  values: string[];
  tone: "teal" | "blue" | "amber";
  emptyText?: string;
}) {
  return (
    <div className="rounded-md border border-border bg-white p-4 shadow-soft">
      <div className="mb-3 text-sm font-semibold">{title}</div>
      {values.length === 0 ? (
        <p className="text-sm leading-6 text-slate-500">{emptyText ?? "暂无内容。"}</p>
      ) : (
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
      )}
    </div>
  );
}

function CompactList({ title, values, emptyText }: { title: string; values: string[]; emptyText?: string }) {
  if (values.length === 0 && !emptyText) {
    return null;
  }
  return (
    <div className="mb-3 last:mb-0">
      <div className="mb-1 text-xs font-semibold text-slate-600">{title}</div>
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

function severityTone(severity?: string): "rose" | "amber" | "slate" {
  if (severity === "high") {
    return "rose";
  }
  if (severity === "medium") {
    return "amber";
  }
  return "slate";
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

function scoreLevel(score: number): string {
  if (score >= 9) {
    return "强";
  }
  if (score >= 7) {
    return "达标";
  }
  if (score >= 5) {
    return "偏弱";
  }
  return "严重";
}

function rewriteTypeLabel(value: string): string {
  return rewriteTypeLabels[value] ?? "自定义返工";
}

function rewriteActionLabel(report?: ReviewReport | null): string {
  if (!report?.rewrite_instruction.needed) {
    return "重写";
  }
  return rewriteActionLabels[report.rewrite_instruction.rewrite_type] ?? "按建议重写";
}

function compareReviewIssues(left: ReviewReport["issues"][number], right: ReviewReport["issues"][number]): number {
  const severity = (issueSeverityOrder[left.severity] ?? 99) - (issueSeverityOrder[right.severity] ?? 99);
  if (severity !== 0) {
    return severity;
  }
  return (issueDimensionOrder[left.dimension] ?? 99) - (issueDimensionOrder[right.dimension] ?? 99);
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
