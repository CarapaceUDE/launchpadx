import { AlertTriangle, CheckCircle2, RefreshCw, X } from "lucide-react";
import type { ConnectionAlert } from "../../types";

export function ConnectionBanner({
  alert,
  busy,
  onRefreshEndpoint,
  onDismiss,
}: {
  alert: ConnectionAlert;
  busy?: boolean;
  onRefreshEndpoint?: () => void;
  onDismiss: () => void;
}) {
  const isInfo = alert.severity === "info";
  const Icon = isInfo ? CheckCircle2 : AlertTriangle;
  const border = isInfo ? "border-emerald-500/30 bg-emerald-500/10" : "border-warning-fg/30 bg-warning-bg/15";
  const iconClass = isInfo ? "text-emerald-500" : "text-warning-fg";

  return (
    <div
      className={`mb-4 rounded-lg border px-4 py-3 ${border}`}
      role="alert"
      data-testid="connection-banner"
      data-kind={alert.kind}
    >
      <div className="flex items-start gap-3">
        <Icon className={`mt-0.5 h-5 w-5 shrink-0 ${iconClass}`} />
        <div className="min-w-0 flex-1 space-y-2">
          <div>
            <p className="text-sm font-semibold text-foreground">{alert.title}</p>
            <p className="mt-1 text-[13px] leading-snug text-muted-foreground">{alert.message}</p>
            {alert.endpointHealth?.endpointUrl ? (
              <p className="mt-2 text-[12px] text-foreground/70">
                Endpoint: {alert.endpointHealth.endpointUrl}
                {alert.endpointHealth.latencyMs != null
                  ? ` · ${alert.endpointHealth.latencyMs} ms`
                  : ""}
                {alert.endpointHealth.modelCount != null
                  ? ` · ${alert.endpointHealth.modelCount} model(s)`
                  : ""}
              </p>
            ) : null}
          </div>
          <div className="flex flex-wrap gap-2">
            {onRefreshEndpoint ? (
              <button
                type="button"
                data-testid="connection-refresh-endpoint-button"
                onClick={onRefreshEndpoint}
                disabled={busy}
                className="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-3 py-1.5 text-[12px] font-medium text-foreground transition-colors hover:bg-muted/50 disabled:cursor-not-allowed disabled:opacity-50"
              >
                <RefreshCw className={`h-3.5 w-3.5 ${busy ? "animate-spin" : ""}`} />
                Re-check endpoint
              </button>
            ) : null}
            <button
              type="button"
              data-testid="connection-dismiss-button"
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