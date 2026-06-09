import { countWords } from "../../lib/format";
import { Badge } from "../../components/ui/Badge";

export function DraftDiffPanel({ savedContent, draftContent }: { savedContent: string; draftContent: string }) {
  const diff = buildWordDiff(savedContent, draftContent);
  const added = diff.filter((item) => item.kind === "added").length;
  const removed = diff.filter((item) => item.kind === "removed").length;

  if (savedContent === draftContent) {
    return (
      <div className="flex h-full items-center justify-center rounded-md border border-dashed border-border bg-white text-sm text-slate-500">
        当前草稿和已保存正文一致
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto rounded-md border border-border bg-white">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-line px-4 py-3">
        <div className="flex flex-wrap gap-2">
          <Badge tone="teal">新增 {added}</Badge>
          <Badge tone="rose">删除 {removed}</Badge>
          <Badge tone="slate">草稿 {countWords(draftContent)} 字</Badge>
        </div>
      </div>
      <div className="p-4 text-sm leading-8">
        {diff.map((item, index) => {
          if (item.kind === "added") {
            return (
              <span key={`${item.text}-${index}`} className="mx-0.5 rounded bg-teal-50 px-1 text-teal-800">
                {item.text}
              </span>
            );
          }
          if (item.kind === "removed") {
            return (
              <span key={`${item.text}-${index}`} className="mx-0.5 rounded bg-rose-50 px-1 text-rose-800 line-through">
                {item.text}
              </span>
            );
          }
          return <span key={`${item.text}-${index}`}>{item.text}</span>;
        })}
      </div>
    </div>
  );
}

type WordDiff = { kind: "same" | "added" | "removed"; text: string };

function buildWordDiff(before: string, after: string): WordDiff[] {
  const left = tokenize(before);
  const right = tokenize(after);
  const table = Array.from({ length: left.length + 1 }, () => Array<number>(right.length + 1).fill(0));

  for (let i = left.length - 1; i >= 0; i -= 1) {
    for (let j = right.length - 1; j >= 0; j -= 1) {
      table[i][j] = left[i] === right[j] ? table[i + 1][j + 1] + 1 : Math.max(table[i + 1][j], table[i][j + 1]);
    }
  }

  const result: WordDiff[] = [];
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

function tokenize(value: string): string[] {
  return value.match(/[\u4e00-\u9fa5]|[A-Za-z0-9_]+|\s+|[^\sA-Za-z0-9_\u4e00-\u9fa5]/g) ?? [];
}
