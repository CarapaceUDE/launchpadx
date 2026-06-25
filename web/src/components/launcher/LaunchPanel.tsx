import { Play, Square, Info, RefreshCw, ChevronDown } from "lucide-react";
import { Card, StatusPill, FormField } from "./primitives";

export function LaunchPanel({
  running,
  onToggle,
  canStart,
  startBlockedReason,
  statusStripText,
  models,
  selectedModel,
  onSelectModel,
  onRefreshModels,
  refreshing,
  onViewModelDetails,
  modelStatusHint,
}: {
  running: boolean;
  onToggle: () => void;
  canStart: boolean;
  startBlockedReason?: string;
  statusStripText: string;
  models: string[];
  selectedModel: string;
  onSelectModel: (v: string) => void;
  onRefreshModels: () => void;
  refreshing: boolean;
  onViewModelDetails?: (name: string) => void;
  modelStatusHint?: string;
}) {
  const empty = models.length === 0;

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
        data-testid="launch-toggle"
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

      <div className="mt-4 border-t border-border pt-4">
        <FormField
          label="Model"
          hint={
            modelStatusHint && empty
              ? modelStatusHint
              : empty
                ? "Refresh models from Endpoint Configuration below."
                : undefined
          }
        >
          <div className="flex gap-2">
            <div className="relative flex-1">
              <select
                data-testid="model-select"
                value={selectedModel}
                onChange={(e) => onSelectModel(e.target.value)}
                disabled={empty}
                className="h-[38px] w-full appearance-none rounded-md border border-input bg-background px-3 pr-9 text-sm text-foreground focus:border-primary focus:outline-none focus:ring-4 focus:ring-primary/15 disabled:bg-muted/60 disabled:text-muted-foreground"
              >
                {empty ? (
                  <option value="">No models detected</option>
                ) : (
                  <>
                    <option value="" disabled>
                      Select a model...
                    </option>
                    {models.map((m) => (
                      <option key={m} value={m}>
                        {m}
                      </option>
                    ))}
                  </>
                )}
              </select>
              <ChevronDown className="pointer-events-none absolute right-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            </div>

            <button
              type="button"
              data-testid="refresh-models"
              onClick={onRefreshModels}
              disabled={refreshing}
              className="inline-flex h-[38px] shrink-0 items-center gap-1.5 rounded-md border border-input bg-background px-3 text-[13px] font-medium text-foreground hover:bg-muted/70 disabled:cursor-not-allowed disabled:opacity-60"
            >
              <RefreshCw className={`h-3.5 w-3.5 ${refreshing ? "animate-spin" : ""}`} />
              Refresh
            </button>

            {onViewModelDetails && (
              <button
                type="button"
                disabled={!selectedModel}
                onClick={() => onViewModelDetails(selectedModel)}
                className="inline-flex h-[38px] shrink-0 items-center rounded-md border border-input bg-background px-3 text-[13px] font-medium text-foreground hover:bg-muted/70 disabled:cursor-not-allowed disabled:bg-muted/60 disabled:text-muted-foreground"
              >
                Details
              </button>
            )}
          </div>
        </FormField>
      </div>

      {!running && !canStart && startBlockedReason && (
        <p className="mt-3 text-[12px] text-warning-fg">{startBlockedReason}</p>
      )}

      <div className="mt-4 flex items-center gap-2 rounded-lg border border-border bg-secondary/50 px-3.5 py-2.5">
        <Info className="h-4 w-4 shrink-0 text-muted-foreground" />
        <span className="text-[13px] text-muted-foreground" data-testid="status-strip">
          {statusStripText}
        </span>
      </div>
    </Card>
  );
}