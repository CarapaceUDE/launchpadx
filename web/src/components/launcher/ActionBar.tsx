import { LoaderCircle, Save, FileCog } from "lucide-react";

export function ActionBar({
  onSave,
  onWrite,
  canWrite,
  saving,
}: {
  onSave: () => void;
  onWrite: () => void;
  canWrite: boolean;
  saving?: boolean;
}) {
  return (
    <div className="flex items-center justify-end gap-3">
      <button
        onClick={onSave}
        disabled={saving}
        className="inline-flex h-[40px] items-center gap-2 rounded-md border border-input bg-background px-4 text-[13px] font-semibold text-foreground transition-colors hover:bg-muted/70 disabled:cursor-not-allowed disabled:bg-border disabled:text-muted-foreground"
      >
        {saving ? (
          <LoaderCircle className="h-4 w-4 animate-spin" />
        ) : (
          <Save className="h-4 w-4" />
        )}
        {saving ? "Saving..." : "Save Configuration"}
      </button>
      <button
        onClick={onWrite}
        disabled={!canWrite}
        className="inline-flex h-[40px] items-center gap-2 rounded-md bg-primary px-4 text-[13px] font-semibold text-primary-foreground transition-colors hover:bg-primary-hover disabled:cursor-not-allowed disabled:bg-border disabled:text-muted-foreground"
      >
        <FileCog className="h-4 w-4" />
        Write Codex Config
      </button>
    </div>
  );
}