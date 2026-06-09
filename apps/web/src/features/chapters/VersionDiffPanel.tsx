import { useEffect, useMemo, useState } from "react";
import type { ChapterVersion } from "../../types/domain";
import { Badge } from "../../components/ui/Badge";

type DiffKind = "same" | "added" | "removed";

interface DiffLine {
  kind: DiffKind;
  text: string;
}

export function VersionDiffPanel({ versions }: { versions: ChapterVersion[] }) {
  const orderedVersions = useMemo(() => [...versions].sort((a, b) => a.version - b.version), [versions]);
  const defaultFrom = orderedVersions[orderedVersions.length - 2]?.version ?? orderedVersions[0]?.version ?? 1;
  const defaultTo = orderedVersions[orderedVersions.length - 1]?.version ?? defaultFrom;
  const [fromVersion, setFromVersion] = useState(defaultFrom);
  const [toVersion, setToVersion] = useState(defaultTo);

  const from = orderedVersions.find((version) => version.version === fromVersion) ?? orderedVersions[0];
  const to = orderedVersions.find((version) => version.version === toVersion) ?? orderedVersions[orderedVersions.length - 1];
  const diff = useMemo(() => buildLineDiff(from?.content ?? "", to?.content ?? ""), [from?.content, to?.content]);
  const summary = useMemo(
    () => ({
      added: diff.filter((line) => line.kind === "added").length,
      removed: diff.filter((line) => line.kind === "removed").length,
    }),
    [diff],
  );

  useEffect(() => {
    if (orderedVersions.length < 2) {
      return;
    }
    const hasFrom = orderedVersions.some((version) => version.version === fromVersion);
    const hasTo = orderedVersions.some((version) => version.version === toVersion);
    if (!hasFrom) {
      setFromVersion(defaultFrom);
    }
    if (!hasTo) {
      setToVersion(defaultTo);
    }
  }, [defaultFrom, defaultTo, fromVersion, orderedVersions, toVersion]);

  if (orderedVersions.length < 2 || !from || !to) {
    return null;
  }

  return (
    <div className="border-t border-line bg-white p-4">
      <div className="mb-3 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h3 className="text-sm font-semibold text-ink">版本对比</h3>
          <div className="mt-1 flex gap-2 text-xs text-slate-500">
            <span>新增 {summary.added} 行</span>
            <span>删除 {summary.removed} 行</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <label className="flex items-center gap-2 text-xs text-slate-600">
            from
            <select value={fromVersion} onChange={(event) => setFromVersion(Number(event.target.value))} className="input h-8 w-24">
              {orderedVersions.map((version) => (
                <option key={version.id} value={version.version}>
                  v{version.version}
                </option>
              ))}
            </select>
          </label>
          <label className="flex items-center gap-2 text-xs text-slate-600">
            to
            <select value={toVersion} onChange={(event) => setToVersion(Number(event.target.value))} className="input h-8 w-24">
              {orderedVersions.map((version) => (
                <option key={version.id} value={version.version}>
                  v{version.version}
                </option>
              ))}
            </select>
          </label>
        </div>
      </div>
      <div className="max-h-96 overflow-auto rounded-md border border-border bg-slate-50 font-mono text-xs leading-6">
        {diff.map((line, index) => (
          <div
            key={`${line.kind}-${index}-${line.text.slice(0, 12)}`}
            className={
              line.kind === "added"
                ? "grid grid-cols-[36px_minmax(0,1fr)] bg-teal-50 text-teal-900"
                : line.kind === "removed"
                  ? "grid grid-cols-[36px_minmax(0,1fr)] bg-rose-50 text-rose-900"
                  : "grid grid-cols-[36px_minmax(0,1fr)] text-slate-700"
            }
          >
            <span className="select-none border-r border-white/70 px-2 text-center text-slate-400">
              {line.kind === "added" ? "+" : line.kind === "removed" ? "-" : ""}
            </span>
            <span className="whitespace-pre-wrap px-3">{line.text || " "}</span>
          </div>
        ))}
      </div>
      <div className="mt-3 flex flex-wrap gap-2">
        <Badge tone="teal">新增</Badge>
        <Badge tone="rose">删除</Badge>
        <Badge tone="slate">未变更</Badge>
      </div>
    </div>
  );
}

function buildLineDiff(before: string, after: string): DiffLine[] {
  const left = normalizeLines(before);
  const right = normalizeLines(after);
  const table = Array.from({ length: left.length + 1 }, () => Array<number>(right.length + 1).fill(0));

  for (let i = left.length - 1; i >= 0; i -= 1) {
    for (let j = right.length - 1; j >= 0; j -= 1) {
      table[i][j] = left[i] === right[j] ? table[i + 1][j + 1] + 1 : Math.max(table[i + 1][j], table[i][j + 1]);
    }
  }

  const result: DiffLine[] = [];
  let i = 0;
  let j = 0;
  while (i < left.length && j < right.length) {
    if (left[i] === right[j]) {
      result.push({ kind: "same", text: left[i] });
      i += 1;
      j += 1;
    } else if (table[i + 1][j] >= table[i][j + 1]) {
      result.push({ kind: "removed", text: left[i] });
      i += 1;
    } else {
      result.push({ kind: "added", text: right[j] });
      j += 1;
    }
  }
  while (i < left.length) {
    result.push({ kind: "removed", text: left[i] });
    i += 1;
  }
  while (j < right.length) {
    result.push({ kind: "added", text: right[j] });
    j += 1;
  }

  return result;
}

function normalizeLines(value: string): string[] {
  return value.replace(/\r\n/g, "\n").split("\n");
}
