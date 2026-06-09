import { NavLink, Outlet } from "react-router-dom";
import { Bot, BookOpen, ClipboardList, FilePlus2, ListChecks } from "lucide-react";
import { cn } from "../lib/cn";
import { AgentRail } from "./AgentRail";
import { ApiStatus } from "./ApiStatus";

const navItems = [
  { to: "/novels", label: "作品", icon: BookOpen },
  { to: "/novels/new", label: "新建", icon: FilePlus2 },
  { to: "/jobs", label: "任务", icon: ClipboardList },
  { to: "/agent-runs", label: "运行", icon: ListChecks },
];

export function AppShell() {
  return (
    <div className="grid min-h-screen grid-cols-[76px_minmax(0,1fr)] bg-slate-100 text-ink md:grid-cols-[224px_minmax(0,1fr)] xl:grid-cols-[224px_minmax(0,1fr)_320px]">
      <aside className="flex min-h-screen flex-col border-r border-line bg-white">
        <div className="flex h-14 items-center gap-2 border-b border-line px-4">
          <div className="flex h-8 w-8 items-center justify-center rounded-md bg-ink text-white">
            <Bot className="h-4 w-4" />
          </div>
          <div className="hidden md:block">
            <div className="text-sm font-semibold">novel-agent</div>
            <div className="text-xs text-slate-500">创作工作台</div>
          </div>
        </div>
        <nav className="space-y-1 p-2">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  cn(
                    "flex h-10 items-center justify-center gap-2 rounded-md px-2 text-sm font-medium text-slate-600 transition hover:bg-slate-100 md:justify-start",
                    isActive && "bg-teal-50 text-teal-700",
                  )
                }
                title={item.label}
              >
                <Icon className="h-4 w-4" />
                <span className="hidden md:inline">{item.label}</span>
              </NavLink>
            );
          })}
        </nav>
        <div className="mt-auto">
          <ApiStatus />
        </div>
      </aside>
      <main className="min-w-0">
        <Outlet />
      </main>
      <AgentRail />
    </div>
  );
}
