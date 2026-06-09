import type { ChapterVersion } from "../../types/domain";
import { formatDateTime } from "../../lib/format";
import { Badge } from "../../components/ui/Badge";

export function VersionList({
  versions,
  selectedVersion,
  onSelect,
}: {
  versions: ChapterVersion[];
  selectedVersion?: number;
  onSelect: (version: ChapterVersion) => void;
}) {
  return (
    <div className="grid gap-3 overflow-x-auto p-3 md:grid-flow-col md:auto-cols-[280px]">
      {versions.length === 0 ? (
        <div className="text-sm text-slate-500">暂无版本</div>
      ) : (
        versions.map((version) => (
          <button
            key={version.id}
            type="button"
            data-testid={`chapter-version-${version.version}`}
            onClick={() => onSelect(version)}
            className={`rounded-md border bg-white p-3 text-left shadow-soft transition hover:border-accent ${
              selectedVersion === version.version ? "border-accent" : "border-border"
            }`}
          >
            <div className="mb-2 flex items-center justify-between">
              <span className="text-sm font-semibold">v{version.version}</span>
              <Badge tone={version.data.source === "manual_edit" ? "blue" : version.data.source === "rewrite" ? "amber" : "teal"}>
                {version.data.source}
              </Badge>
            </div>
            <p className="line-clamp-2 text-xs leading-5 text-slate-600">{version.summary || version.data.notes}</p>
            <div className="mt-3 flex items-center justify-between text-xs text-slate-500">
              <span>{version.word_count} 字</span>
              <span>{formatDateTime(version.created_at)}</span>
            </div>
          </button>
        ))
      )}
    </div>
  );
}
