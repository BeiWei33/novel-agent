import type { HTMLAttributes, ReactNode } from "react";
import { cn } from "../../lib/cn";

interface SectionProps extends HTMLAttributes<HTMLElement> {
  title?: string;
  actions?: ReactNode;
}

export function Section({ title, actions, children, className, ...props }: SectionProps) {
  return (
    <section className={cn("border-b border-line bg-white", className)} {...props}>
      {(title || actions) && (
        <div className="flex min-h-12 items-center justify-between gap-3 border-b border-line px-4">
          {title ? <h2 className="text-sm font-semibold text-ink">{title}</h2> : <div />}
          {actions}
        </div>
      )}
      {children}
    </section>
  );
}
