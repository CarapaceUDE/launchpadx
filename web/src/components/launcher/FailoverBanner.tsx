import { AlertTriangle, Copy, X } from "lucide-react";
import type { FailoverAlert, SessionCheckpoint } from "../../types";
import { TARGET_APP_NAME } from "../../lib/branding";
import { formatRateLimitReachedType } from "../../lib/rateLimits";

export function FailoverBanner({
  alert,
  checkpoint,
  busy,
  onFailover,
  onDismiss,
  onCopyResume,
}: {
  alert: FailoverAlert;
  checkpoint?: SessionCheckpoint | null;
  busy?: boolean;
  onFailover: () => void;
  onDismiss: () => void;
  onCopyResume: () => void;
}) {
  const reachedLabel =
    alert.source === "app_server_rate_limits"
      ? formatRateLimitReachedType(alert.matchedPattern) ?? alert.matchedPattern
      : alert.matchedPattern;

  return (
    <div
      className="mb-4 rounded-lg border border-warning-fg/30 bg-warning-bg/15 px-4 py-3"
      role="alert"
      data-testid="failover-banner"
    >
      <div className="flex items-start gap-3">
        <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-warning-fg" />
        <div className="min-w-0 flex-1 space-y-2">
          <div>
            <p className="text-sm font-semibold text-foreground">
              Cloud account limit detected
            </p>
            <p className="mt-1 text-[13px] leading-snug text-muted-foreground">
              {alert.source === "app_server_rate_limits"
                ? `App-server reports ${reachedLabel} via account/rateLimits/read. Switch to your local provider, restart ${TARGET_APP_NAME}, and resume with the saved context.`
                : `Matched "${alert.matchedPattern}". Switch to your configured local profile, restart ${TARGET_APP_NAME}, and resume with the saved context.`}
            </p>
            {alert.snippet ? (
              <p className="mt-2 truncate rounded-md border border-border/70 bg-background/60 px-2.5 py-1.5 text-[12px] text-foreground/80">
                {alert.snippet}
              </p>
            ) : null}
          </div>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              data-testid="failover-switch-button"
              onClick={onFailover}
              disabled={busy}
              className="inline-flex items-center rounded-md bg-primary px-3 py-1.5 text-[12px] font-medium text-primary-foreground transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
            >
              Switch to local &amp; restart
            </button>
            {checkpoint?.resumePrompt ? (
              <button
                type="button"
                data-testid="failover-copy-resume-button"
                onClick={onCopyResume}
                className="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-3 py-1.5 text-[12px] font-medium text-foreground transition-colors hover:bg-muted/50"
              >
                <Copy className="h-3.5 w-3.5" />
                Copy resume prompt
              </button>
            ) : null}
            <button
              type="button"
              data-testid="failover-dismiss-button"
              onClick={onDismiss}
              className="inline-flex items-center gap-1.5 rounded-md px-2 py-1.5 text-[12px] font-medium text-muted-foreground transition-colors hover:bg-muted/40 hover:text-foreground"
            >
              <X className="h-3.5 w-3.5" />
              Dismiss
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}