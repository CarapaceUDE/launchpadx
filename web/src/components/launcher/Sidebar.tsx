import { Rocket, Activity, Settings, FileText, Info, RefreshCw, ChevronRight } from "lucide-react";
import { AppIcon } from "./AppIcon";
import { ThemeToggle } from "./ThemeToggle";
import type { NavKey } from "../../context/LauncherContext";
import { APP_NAME, LICENSE_NOTICE, LICENSE_URL } from "../../lib/branding";

const items: { key: NavKey; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { key: "launcher", label: "Launchpad", icon: Rocket },
  { key: "sessions", label: "Sessions", icon: Activity },
  { key: "settings", label: "Settings", icon: Settings },
  { key: "logs", label: "Logs", icon: FileText },
  { key: "about", label: "About", icon: Info },
];

interface SidebarProps {
  modelCount: number;
  refreshing: boolean;
  onRefresh: () => void;
  activeNav: NavKey;
  onNavChange: (nav: NavKey) => void;
}

export function Sidebar({
  modelCount,
  refreshing,
  onRefresh,
  activeNav,
  onNavChange,
}: SidebarProps) {
  return (
    <>
      <aside
        data-testid="app-sidebar"
        className="sidebar-gradient hidden shrink-0 flex-col self-stretch text-white md:flex md:w-[72px] lg:w-[260px]"
      >
        <div className="px-3 pt-6 pb-5 lg:px-6 lg:pt-7 lg:pb-6">
          <div className="flex items-center justify-center gap-2.5 lg:justify-start">
            <div className="h-9 w-9 shrink-0 overflow-hidden rounded-lg ring-1 ring-white/15">
              <AppIcon size={36} className="h-full w-full" />
            </div>
            <div className="hidden min-w-0 lg:block">
              <div className="text-[15px] font-semibold leading-tight tracking-tight text-white">
                Codex
              </div>
              <div className="text-[12px] leading-tight text-white/70">Launchpad</div>
            </div>
          </div>
        </div>

        <nav className="flex-1 px-2 lg:px-3">
          <ul className="space-y-1">
            {items.map((it) => {
              const Icon = it.icon;
              const isActive = activeNav === it.key;
              return (
                <li key={it.key}>
                  <button
                    data-testid={`nav-${it.key}`}
                    title={it.label}
                    onClick={() => onNavChange(it.key)}
                    className={[
                      "flex w-full items-center rounded-lg text-sm transition-colors",
                      "justify-center gap-0 px-2 py-2.5 lg:justify-start lg:gap-3 lg:px-3",
                      isActive
                        ? "bg-white/12 text-white shadow-[inset_0_0_0_1px_rgba(255,255,255,0.08)]"
                        : "text-white/80 hover:bg-white/8 hover:text-white",
                    ].join(" ")}
                  >
                    <Icon className="h-4 w-4 shrink-0" />
                    <span className="hidden font-medium lg:inline">{it.label}</span>
                    {isActive ? (
                      <ChevronRight className="ml-auto hidden h-3 w-3 text-white/40 lg:block" />
                    ) : null}
                  </button>
                </li>
              );
            })}
          </ul>
        </nav>

        <div className="space-y-3 px-2 pb-5 lg:px-4">
          <button
            data-testid="sidebar-refresh-models"
            onClick={onRefresh}
            disabled={refreshing}
            aria-busy={refreshing}
            title="Refresh models"
            className="flex w-full items-center justify-center rounded-lg border border-white/10 bg-white/5 px-2 py-2.5 text-sm text-white/85 transition-colors hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-60 lg:justify-between lg:px-3.5"
          >
            <span className="flex items-center gap-2">
              <RefreshCw className={`h-3.5 w-3.5 ${refreshing ? "animate-spin" : ""}`} />
              <span className="hidden lg:inline">
                {refreshing ? "Refreshing..." : "Refresh Models"}
              </span>
            </span>
            <span className="rounded-full bg-white/15 px-2 py-0.5 text-[11px] font-semibold text-white">
              {modelCount}
            </span>
          </button>

          <div className="hidden lg:block">
            <ThemeToggle />
          </div>

          <div className="hidden border-t border-white/10 pt-3 lg:block">
            <p className="text-center text-[10px] leading-relaxed text-white/45">
              <a
                href={LICENSE_URL}
                title={`${APP_NAME} license`}
                className="text-white/60 underline-offset-2 transition-colors hover:text-white/85 hover:underline"
              >
                {LICENSE_NOTICE}
              </a>
            </p>
          </div>
        </div>
      </aside>

      <nav
        data-testid="mobile-nav"
        className="sidebar-gradient fixed inset-x-0 bottom-0 z-40 border-t border-white/10 px-2 pb-[max(0.5rem,env(safe-area-inset-bottom))] pt-2 md:hidden"
      >
        <ul className="grid grid-cols-5 gap-1">
          {items.map((it) => {
            const Icon = it.icon;
            const isActive = activeNav === it.key;
            return (
              <li key={it.key}>
                <button
                  data-testid={`mobile-nav-${it.key}`}
                  onClick={() => onNavChange(it.key)}
                  aria-current={isActive ? "page" : undefined}
                  className={[
                    "flex w-full flex-col items-center gap-0.5 rounded-lg px-1 py-2 text-[10px] font-medium transition-colors",
                    isActive
                      ? "bg-white/12 text-white"
                      : "text-white/75 hover:bg-white/8 hover:text-white",
                  ].join(" ")}
                >
                  <Icon className="h-4 w-4 shrink-0" />
                  <span className="max-w-full truncate">{it.label}</span>
                </button>
              </li>
            );
          })}
        </ul>
      </nav>
    </>
  );
}