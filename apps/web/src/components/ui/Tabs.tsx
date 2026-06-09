import { cn } from "../../lib/cn";

interface TabItem<T extends string> {
  value: T;
  label: string;
}

interface TabsProps<T extends string> {
  value: T;
  items: TabItem<T>[];
  onChange: (value: T) => void;
  className?: string;
}

export function Tabs<T extends string>({ value, items, onChange, className }: TabsProps<T>) {
  return (
    <div className={cn("flex min-h-9 max-w-full items-center overflow-x-auto rounded-md border border-border bg-slate-50 p-1", className)}>
      {items.map((item) => (
        <button
          key={item.value}
          type="button"
          onClick={() => onChange(item.value)}
          className={cn(
            "h-7 shrink-0 whitespace-nowrap rounded px-2.5 text-xs font-medium text-slate-600 transition",
            value === item.value && "bg-white text-ink shadow-soft",
          )}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
