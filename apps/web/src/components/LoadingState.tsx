export function LoadingState({ label = "加载中" }: { label?: string }) {
  return (
    <div className="flex min-h-40 items-center justify-center text-sm text-slate-500">
      <span className="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-slate-300 border-t-accent" />
      {label}
    </div>
  );
}
