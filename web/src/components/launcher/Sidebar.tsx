import { Rocket, Boxes, Settings, FileText, Info, RefreshCw, ChevronRight } from "lucide-react";
import { ThemeToggle } from "./ThemeToggle";
import type { NavKey } from "../../context/LauncherContext";

const items: { key: NavKey; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { key: "launcher", label: "Launcher", icon: Rocket },
  { key: "models", label: "Models", icon: Boxes },
  { key: "settings", label: "Settings", icon: Settings },
  { key: "logs", label: "Logs", icon: FileText },
  { key: "about", label: "About", icon: Info },
];

interface SidebarProps {
  running: boolean;
  statusMessage: string;
  modelCount: number;
  onRefresh: () => void;
  activeNav: NavKey;
  onNavChange: (nav: NavKey) => void;
}

export function Sidebar({
  running,
  statusMessage,
  modelCount,
  onRefresh,
  activeNav,
  onNavChange,
}: SidebarProps) {
  return (
    <aside className="sidebar-gradient flex w-[300px] flex-col self-stretch text-white">
      <div className="px-6 pt-7 pb-6">
        <div className="flex items-center gap-2.5">
          <div className="grid h-9 w-9 place-items-center rounded-lg bg-white/10 ring-1 ring-white/15">
            <Rocket className="h-4 w-4" />
          </div>
          <div>
            <div className="text-[15px] font-semibold leading-tight tracking-tight text-white">
              Codex Local
            </div>
            <div className="text-[12px] leading-tight text-white/70">Launcher</div>
          </div>
        </div>
      </div>

      <nav className="flex-1 px-3">
        <ul className="space-y-1">
          {items.map((it) => {
            const Icon = it.icon;
            const isActive = activeNav === it.key;
            return (
              <li key={it.key}>
                <button
                  data-testid={`nav-${it.key}`}
                  onClick={() => onNavChange(it.key)}
                  className={[
                    "flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm transition-colors",
                    isActive
                      ? "bg-white/12 text-white shadow-[inset_0_0_0_1px_rgba(255,255,255,0.08)]"
                      : "text-white/80 hover:bg-white/8 hover:text-white",
                  ].join(" ")}
                >
                  <Icon className="h-4 w-4" />
                  <span className="font-medium">{it.label}</span>
                  {isActive && <ChevronRight className="ml-auto h-3 w-3 text-white/40" />}
                </button>
              </li>
            );
          })}
        </ul>
      </nav>

      <div className="space-y-3 px-4 pb-5">
        <div className="rounded-xl border border-white/10 bg-white/5 p-4 backdrop-blur-sm">
          <div className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-white/70">
            Codex Status
          </div>
          <div className="flex items-center gap-2">
            <span
              className={[
                "relative h-2.5 w-2.5 rounded-full",
                running ? "bg-success" : "bg-warning-fg",
              ].join(" ")}
            >
              {running && (
                <span className="absolute inset-0 animate-ping rounded-full bg-success/70" />
              )}
            </span>
            <span className="text-sm font-semibold text-white">
              {running ? "Running" : "Stopped"}
            </span>
          </div>
          <p className="mt-2 text-[12px] leading-snug text-white/70" data-testid="sidebar-status">
            {statusMessage}
          </p>
        </div>

        <button
          data-testid="sidebar-refresh-models"
          onClick={onRefresh}
          className="flex w-full items-center justify-between rounded-lg border border-white/10 bg-white/5 px-3.5 py-2.5 text-sm text-white/85 transition-colors hover:bg-white/10"
        >
          <span className="flex items-center gap-2">
            <RefreshCw className="h-3.5 w-3.5" />
            Refresh Models
          </span>
          <span className="rounded-full bg-white/15 px-2 py-0.5 text-[11px] font-semibold text-white">
            {modelCount}
          </span>
        </button>

        <ThemeToggle />
      </div>
    </aside>
  );
}