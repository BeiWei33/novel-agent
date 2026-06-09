import type { ReactNode } from "react";
import { AlertTriangle, CheckCircle2, Loader2 } from "lucide-react";
import { cn } from "../lib/cn";

type StatusTone = "info" | "success" | "danger";

interface StatusBannerProps {
  tone?: StatusTone;
  title: string;
  children?: ReactNode;
}

const toneStyles: Record<StatusTone, string> = {
  info: "border-sky-200 bg-sky-50 text-sky-900",
  success: "border-teal-200 bg-teal-50 text-teal-900",
  danger: "border-rose-200 bg-rose-50 text-rose-900",
};

export function StatusBanner({ tone = "info", title, children }: StatusBannerProps) {
  const Icon = tone === "danger" ? AlertTriangle : tone === "success" ? CheckCircle2 : Loader2;
  return (
    <div className={cn("flex items-start gap-3 border-b px-4 py-3 text-sm", toneStyles[tone])}>
      <Icon className={cn("mt-0.5 h-4 w-4 shrink-0", tone === "info" && "animate-spin")} />
      <div className="min-w-0">
        <div className="font-semibold">{title}</div>
        {children ? <div className="mt-1 leading-6 opacity-90">{children}</div> : null}
      </div>
    </div>
  );
}
