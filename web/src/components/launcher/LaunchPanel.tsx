import { Play, Square, Info } from "lucide-react";
import { Card, StatusPill } from "./primitives";

export function LaunchPanel({
  running,
  onToggle,
  canStart,
  statusStripText,
}: {
  running: boolean;
  onToggle: () => void;
  canStart: boolean;
  statusStripText: string;
}) {
  return (
    <Card className="!p-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-[16px] font-semibold tracking-tight text-foreground">Launch Codex</h2>
          <p className="mt-1 text-[13px] text-muted-foreground">
            Start or stop the Codex-compatible local API server
          </p>
        </div>
        <StatusPill running={running} />
      </div>

      <button
        onClick={onToggle}
        disabled={!running && !canStart}
        className={[
          "mt-5 flex w-full items-center justify-center gap-2 rounded-lg px-5 py-3 text-[14px] font-semibold transition-all",
          running
            ? "bg-[color:var(--color-warning-fg)] text-white hover:opacity-90"
            : "bg-primary text-primary-foreground hover:bg-primary-hover",
          "disabled:cursor-not-allowed disabled:bg-border disabled:text-muted-foreground",
        ].join(" ")}
      >
        {running ? (
          <Square className="h-4 w-4" fill="currentColor" />
        ) : (
          <Play className="h-4 w-4" fill="currentColor" />
        )}
        {running ? "Stop Codex Server" : "Start Codex Server"}
      </button>

      <div className="mt-4 flex items-center gap-2 rounded-lg border border-border bg-secondary/50 px-3.5 py-2.5">
        <Info className="h-4 w-4 text-muted-foreground" />
        <span className="text-[13px] text-muted-foreground">{statusStripText}</span>
      </div>
    </Card>
  );
}