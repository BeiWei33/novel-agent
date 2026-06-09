import { countWords } from "../../lib/format";

export function MarkdownPreview({ content }: { content: string }) {
  const blocks = parseMarkdown(content);

  if (!content.trim()) {
    return (
      <div className="flex h-full items-center justify-center rounded-md border border-dashed border-border bg-white text-sm text-slate-500">
        暂无正文
      </div>
    );
  }

  return (
    <article className="h-full overflow-auto rounded-md border border-border bg-white px-6 py-5 text-slate-800">
      <div className="mb-4 border-b border-line pb-3 text-xs text-slate-500">{countWords(content)} 字</div>
      <div className="space-y-4">
        {blocks.map((block, index) => {
          if (block.kind === "heading") {
            return (
              <h2 key={`${block.text}-${index}`} className="text-xl font-semibold leading-8 text-ink">
                {block.text}
              </h2>
            );
          }
          if (block.kind === "divider") {
            return <hr key={`divider-${index}`} className="border-line" />;
          }
          return (
            <p key={`${block.text.slice(0, 12)}-${index}`} className="text-base leading-8">
              {block.text}
            </p>
          );
        })}
      </div>
    </article>
  );
}

type PreviewBlock = { kind: "heading" | "paragraph" | "divider"; text: string };

function parseMarkdown(content: string): PreviewBlock[] {
  return content
    .replace(/\r\n/g, "\n")
    .split(/\n{2,}/)
    .map((block) => block.trim())
    .filter(Boolean)
    .map((block) => {
      if (/^-{3,}$/.test(block)) {
        return { kind: "divider", text: "" };
      }
      const heading = block.match(/^#{1,3}\s+(.+)$/);
      if (heading) {
        return { kind: "heading", text: heading[1] };
      }
      return { kind: "paragraph", text: block };
    });
}
