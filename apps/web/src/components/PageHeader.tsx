import type { ReactNode } from "react";

export function PageHeader({ title, meta, actions }: { title: string; meta?: ReactNode; actions?: ReactNode }) {
  return (
    <header className="flex min-h-14 flex-wrap items-center justify-between gap-3 border-b border-line bg-white px-4">
      <div className="min-w-0">
        <h1 className="truncate text-base font-semibold text-ink">{title}</h1>
        {meta ? <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-slate-500">{meta}</div> : null}
      </div>
      {actions ? <div className="flex shrink-0 items-center gap-2">{actions}</div> : null}
    </header>
  );
}
