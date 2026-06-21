import { useState } from "react";
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
  const [confirmRevert, setConfirmRevert] = useState(false);

  const handleConfirmRevert = () => {
    setConfirmRevert(false);
    onRevert();
  };

  return (
    <>
      <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
        <button
          onClick={() => setConfirmRevert(true)}
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

      {confirmRevert && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
          onClick={() => setConfirmRevert(false)}
        >
          <div
            className="w-full max-w-md rounded-xl border border-border bg-card p-6 shadow-xl"
            onClick={(e) => e.stopPropagation()}
            role="alertdialog"
            aria-labelledby="revert-codex-profile-title"
            aria-describedby="revert-codex-profile-description"
          >
            <h3
              id="revert-codex-profile-title"
              className="text-[16px] font-semibold text-foreground"
            >
              Revert to Codex Profile?
            </h3>
            <p
              id="revert-codex-profile-description"
              className="mt-2 text-[13px] leading-relaxed text-muted-foreground"
            >
              This restores your Codex config to the state saved before the launcher modified it.
              Launcher provider settings will be removed from your Codex config.
            </p>
            <div className="mt-5 flex justify-end gap-2">
              <button
                type="button"
                onClick={() => setConfirmRevert(false)}
                className="inline-flex h-[36px] items-center rounded-md border border-input bg-background px-4 text-[13px] font-semibold text-foreground transition-colors hover:bg-muted/70"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleConfirmRevert}
                className="inline-flex h-[36px] items-center rounded-md bg-[color:var(--color-warning-fg)] px-4 text-[13px] font-semibold text-white transition-colors hover:opacity-90"
              >
                Revert
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}