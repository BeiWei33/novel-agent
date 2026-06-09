import { SearchCheck, Wand2 } from "lucide-react";
import type { ReviewReport } from "../../types/domain";
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

export function ReviewPanel({
  report,
  isLoading,
  onReview,
  onRewrite,
  reviewPending,
  rewritePending,
}: {
  report?: ReviewReport | null;
  isLoading?: boolean;
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
