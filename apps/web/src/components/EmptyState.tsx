import type { ReactNode } from "react";

export function EmptyState({ title, action }: { title: string; action?: ReactNode }) {
  return (
    <div className="flex min-h-40 flex-col items-center justify-center gap-3 border border-dashed border-border bg-white px-4 text-center">
      <p className="text-sm font-medium text-slate-600">{title}</p>
      {action}
    </div>
  );
}
