import { RefreshCw } from "lucide-react";
import { useLauncher } from "../../context/LauncherContext";

export function LocalModelsCatalog({
  modelCount,
  onRefresh,
}: {
  modelCount: number;
  onRefresh: () => void;
}) {
  const { models, config, refreshing } = useLauncher();
  const selected = config.codexModel ?? "";

  return (
    <div className="border-t border-border pt-4" data-testid="local-models-catalog">
      <div className="mb-3 flex items-center justify-between gap-2">
        <div>
          <p className="text-[13px] font-medium text-foreground">Available models ({modelCount})</p>
          <p className="mt-0.5 text-[11px] text-muted-foreground">
            Browse models cached from your endpoint. Select the active model on the Launchpad provider
            card.
          </p>
        </div>
        <button
          type="button"
          onClick={onRefresh}
          disabled={refreshing}
          aria-busy={refreshing}
          className="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-md border border-input bg-background px-2.5 text-[11px] font-medium text-foreground hover:bg-muted/70 disabled:opacity-60"
        >
          <RefreshCw className={`h-3 w-3 ${refreshing ? "animate-spin" : ""}`} />
          Refresh
        </button>
      </div>

      {models.length === 0 ? (
        <p className="text-[12px] text-muted-foreground">
          No models detected. Check endpoint settings above, then refresh.
        </p>
      ) : (
        <div className="themed-scrollbar max-h-[220px] space-y-1 overflow-y-auto pr-1">
          {models.map((model) => {
            const isSelected = model.name === selected;
            return (
              <div
                key={model.name}
                data-testid={`catalog-model-${model.name}`}
                className={[
                  "flex items-center gap-2 rounded-lg border px-3 py-2 text-[12px]",
                  isSelected
                    ? "border-primary/25 bg-primary/5 text-foreground"
                    : "border-transparent bg-muted/20 text-foreground/85",
                ].join(" ")}
              >
                <span className="truncate font-medium">{model.name}</span>
                {isSelected ? (
                  <span className="shrink-0 rounded-full bg-primary/15 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-primary">
                    Launchpad
                  </span>
                ) : null}
                {model.size > 0 ? (
                  <span className="ml-auto shrink-0 text-[11px] text-muted-foreground">
                    {(model.size / (1024 * 1024 * 1024)).toFixed(1)} GB
                  </span>
                ) : null}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}