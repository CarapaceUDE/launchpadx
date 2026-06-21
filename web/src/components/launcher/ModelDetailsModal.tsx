import { Box, Calendar, HardDrive, X } from "lucide-react";
import type { ModelInfo } from "../../types";

function formatSize(bytes: number) {
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export function ModelDetailsModal({
  model,
  onClose,
}: {
  model: ModelInfo;
  onClose: () => void;
}) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md rounded-xl border border-border bg-card p-6 shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="grid h-10 w-10 place-items-center rounded-lg bg-primary/10 text-primary">
              <Box className="h-5 w-5" />
            </span>
            <h3 className="text-[16px] font-semibold text-foreground">Model Details</h3>
          </div>
          <button
            onClick={onClose}
            className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="space-y-4">
          <div>
            <label className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">
              Name
            </label>
            <p className="mt-1 font-mono text-sm text-foreground">{model.name}</p>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="flex items-center gap-1.5 text-[11px] font-medium uppercase tracking-wider text-muted-foreground">
                <HardDrive className="h-3 w-3" />
                Size
              </label>
              <p className="mt-1 text-sm text-foreground">{formatSize(model.size)}</p>
            </div>
            <div>
              <label className="flex items-center gap-1.5 text-[11px] font-medium uppercase tracking-wider text-muted-foreground">
                <Calendar className="h-3 w-3" />
                Modified
              </label>
              <p className="mt-1 text-sm text-foreground">
                {model.modified ? new Date(model.modified).toLocaleDateString() : "N/A"}
              </p>
            </div>
          </div>

          <div>
            <label className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">
              Digest
            </label>
            <p className="mt-1 break-all font-mono text-[11px] text-muted-foreground">
              {model.digest || "N/A"}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}