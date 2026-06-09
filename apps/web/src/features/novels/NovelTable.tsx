import { Link } from "react-router-dom";
import { Download, ExternalLink } from "lucide-react";
import type { NovelListItem } from "../../types/domain";
import { clampScore, formatDateTime, novelStatusLabels, platformLabels } from "../../lib/format";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";

export function NovelTable({ novels, onExport }: { novels: NovelListItem[]; onExport: (novel: NovelListItem) => void }) {
  return (
    <div className="overflow-x-auto bg-white">
      <table className="min-w-[860px] w-full border-collapse">
        <thead className="table-head">
          <tr>
            <th className="px-4 py-3">书名</th>
            <th className="px-3 py-3">题材</th>
            <th className="px-3 py-3">平台</th>
            <th className="px-3 py-3">状态</th>
            <th className="px-3 py-3 text-right">章节</th>
            <th className="px-3 py-3 text-right">最近评分</th>
            <th className="px-3 py-3">更新时间</th>
            <th className="px-4 py-3 text-right">操作</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-line text-sm">
          {novels.map((novel) => (
            <tr key={novel.id} className="hover:bg-slate-50">
              <td className="px-4 py-3">
                <Link to={`/novels/${novel.id}`} className="font-medium text-ink hover:text-accent">
                  {novel.title}
                </Link>
              </td>
              <td className="px-3 py-3 text-slate-600">{novel.genre}</td>
              <td className="px-3 py-3">
                <Badge tone={novel.target_platform === "fanqie" ? "amber" : novel.target_platform === "qidian" ? "blue" : "slate"}>
                  {platformLabels[novel.target_platform]}
                </Badge>
              </td>
              <td className="px-3 py-3">
                <Badge tone={novel.status === "active" ? "teal" : "slate"}>{novelStatusLabels[novel.status]}</Badge>
              </td>
              <td className="px-3 py-3 text-right tabular-nums text-slate-700">{novel.chapter_count}</td>
              <td className="px-3 py-3 text-right tabular-nums text-slate-700">{clampScore(novel.recent_score)}</td>
              <td className="px-3 py-3 text-slate-500">{formatDateTime(novel.updated_at)}</td>
              <td className="px-4 py-3">
                <div className="flex justify-end gap-2">
                  <Button size="sm" variant="ghost" onClick={() => onExport(novel)} title="导出 Markdown">
                    <Download className="h-4 w-4" />
                  </Button>
                  <Link to={`/novels/${novel.id}`}>
                    <Button size="sm" variant="secondary">
                      <ExternalLink className="h-4 w-4" />
                      打开
                    </Button>
                  </Link>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
