import { Box, RefreshCw, ChevronDown } from "lucide-react";
import { Card, FormField } from "./primitives";

export function ModelCard({
  models,
  selected,
  onSelect,
  onRefresh,
  refreshing,
  onViewDetails,
  statusHint,
}: {
  models: string[];
  selected: string;
  onSelect: (v: string) => void;
  onRefresh: () => void;
  refreshing: boolean;
  onViewDetails?: (name: string) => void;
  statusHint?: string;
}) {
  const empty = models.length === 0;

  return (
    <Card icon={<Box className="h-4 w-4" />} title="Model Configuration">
      <FormField
        label="Select Model"
        hint={
          statusHint && empty
            ? statusHint
            : empty
              ? "Click Refresh in Endpoint Configuration or the sidebar."
              : undefined
        }
      >
        <div className="flex gap-2">
          <div className="relative flex-1">
            <select
              value={selected}
              onChange={(e) => onSelect(e.target.value)}
              disabled={empty}
              className="themed-native-select h-[38px] w-full px-3 pr-9 text-sm focus:ring-4 disabled:bg-muted/60 disabled:text-muted-foreground"
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
            onClick={onRefresh}
            className="inline-flex h-[38px] shrink-0 items-center gap-1.5 rounded-md border border-input bg-background px-3 text-[13px] font-medium text-foreground hover:bg-muted/70"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${refreshing ? "animate-spin" : ""}`} />
            Refresh
          </button>

          {onViewDetails && (
            <button
              type="button"
              disabled={!selected}
              onClick={() => onViewDetails(selected)}
              className="inline-flex h-[38px] shrink-0 items-center rounded-md border border-input bg-background px-3 text-[13px] font-medium text-foreground hover:bg-muted/70 disabled:cursor-not-allowed disabled:bg-muted/60 disabled:text-muted-foreground"
            >
              View Details
            </button>
          )}
        </div>
      </FormField>
    </Card>
  );
}