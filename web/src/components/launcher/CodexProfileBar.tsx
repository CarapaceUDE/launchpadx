import { FileCog, Undo2 } from "lucide-react";

export function CodexProfileBar({
  onWrite,
  onRevert,
  canWrite,
}: {
  onWrite: () => void;
  onRevert: () => void;
  canWrite: boolean;
}) {
  return (
    <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
      <button
        onClick={onRevert}
        className="inline-flex h-[40px] items-center justify-center gap-2 rounded-lg border border-input bg-card px-4 text-[13px] font-semibold text-foreground transition-colors hover:bg-muted/70"
      >
        <Undo2 className="h-4 w-4" />
        Revert to Codex Profile
      </button>
      <button
        onClick={onWrite}
        disabled={!canWrite}
        className="inline-flex h-[40px] items-center justify-center gap-2 rounded-lg border border-primary/30 bg-primary/10 px-4 text-[13px] font-semibold text-foreground transition-colors hover:bg-primary/15 disabled:cursor-not-allowed disabled:border-border disabled:bg-border/40 disabled:text-muted-foreground"
      >
        <FileCog className="h-4 w-4" />
        Write Codex Config
      </button>
    </div>
  );
}