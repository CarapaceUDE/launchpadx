import { CircleCheck, CircleX, RefreshCw } from "lucide-react";
import { useLauncher } from "../../context/LauncherContext";

export function ModelsPanel({
  modelCount,
  onRefresh,
}: {
  modelCount: number;
  onRefresh: () => void;
}) {
  const { models, config, refreshing, selectModel } = useLauncher();
  const selected = config.codexModel ?? "";

  return (
    <div className="card-surface space-y-4 p-6">
      <div className="flex items-center justify-between">
        <h3 className="text-[14px] font-semibold text-foreground">Models ({modelCount})</h3>
        <button
          onClick={onRefresh}
          disabled={refreshing}
          className="inline-flex items-center gap-1.5 rounded-md border border-input bg-background px-3 py-1.5 text-[12px] text-foreground transition-colors hover:bg-muted/70 disabled:cursor-not-allowed disabled:opacity-50"
        >
          <RefreshCw className={`h-3 w-3 ${refreshing ? "animate-spin" : ""}`} />
          {refreshing ? "Loading..." : "Refresh"}
        </button>
      </div>

      {models.length === 0 ? (
        <div className="py-6 text-center">
          <p className="text-[13px] text-muted-foreground">
            No models detected. Start your API server, then click Refresh.
          </p>
        </div>
      ) : (
        <div className="themed-scrollbar max-h-[300px] space-y-1 overflow-y-auto pr-1">
          {models.map((model) => {
            const isSelected = model.name === selected;
            return (
              <div
                key={model.name}
                onClick={() => void selectModel(model.name)}
                className={[
                  "flex cursor-pointer items-center gap-2 rounded-lg border px-3 py-2 text-[13px] transition-colors",
                  isSelected
                    ? "border-primary/30 bg-primary/10 text-primary ring-1 ring-primary/20"
                    : "border-transparent text-foreground/80 hover:bg-muted/50 hover:text-foreground",
                ].join(" ")}
              >
                {isSelected ? (
                  <CircleCheck className="h-3.5 w-3.5 shrink-0 text-primary" />
                ) : (
                  <CircleX className="h-3.5 w-3.5 shrink-0 text-muted-foreground/50" />
                )}
                <span className="truncate">{model.name}</span>
                {model.size > 0 && (
                  <span className="ml-auto shrink-0 text-[11px] text-muted-foreground">
                    {(model.size / (1024 * 1024 * 1024)).toFixed(1)} GB
                  </span>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}