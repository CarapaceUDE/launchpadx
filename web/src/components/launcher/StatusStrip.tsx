import { CheckCircle2, Info, Loader2, AlertCircle } from "lucide-react";
import type { LauncherOperation } from "../../lib/operationStatus";
import { isBusyOperation } from "../../lib/operationStatus";
import type { ServerPillState } from "./primitives";

export function StatusStrip({
  message,
  operation = "idle",
  variant = "default",
  compact = false,
  serverState,
}: {
  message: string;
  operation?: LauncherOperation;
  variant?: "default" | "success" | "error";
  compact?: boolean;
  serverState?: ServerPillState;
}) {
  const busy = isBusyOperation(operation);

  const Icon = busy
    ? Loader2
    : variant === "success"
      ? CheckCircle2
      : variant === "error"
        ? AlertCircle
        : Info;

  if (compact) {
    return (
      <div
        className={[
          "inline-flex h-[34px] w-full min-w-0 max-w-full items-center gap-1.5 rounded-md border px-2.5 transition-colors sm:w-auto sm:max-w-[min(320px,55vw)]",
          busy
            ? "border-primary/25 bg-primary/5"
            : variant === "error"
              ? "border-warning-fg/25 bg-warning-bg/10"
              : "border-border bg-muted/40",
        ].join(" ")}
        role="status"
        aria-live="polite"
        aria-busy={busy}
        aria-label={message}
        title={message}
        data-testid="status-strip"
        data-operation={operation}
        data-state={serverState}
      >
        <Icon
          className={[
            "h-3.5 w-3.5 shrink-0",
            busy ? "animate-spin text-primary" : "text-muted-foreground",
            variant === "success" && !busy ? "text-success" : "",
            variant === "error" && !busy ? "text-warning-fg" : "",
          ]
            .filter(Boolean)
            .join(" ")}
        />
        <span className="truncate text-[12px] font-medium leading-none text-foreground/90">
          {message}
        </span>
        {serverState ? (
          <span data-testid="status-pill" data-state={serverState} className="sr-only">
            {serverState}
          </span>
        ) : null}
      </div>
    );
  }

  return (
    <div
      className={[
        "mt-4 flex items-start gap-2.5 rounded-lg border px-3.5 py-2.5 transition-colors",
        busy
          ? "border-primary/25 bg-primary/5"
          : variant === "error"
            ? "border-warning-fg/25 bg-warning-bg/10"
            : "border-border bg-secondary/50",
      ].join(" ")}
      role="status"
      aria-live="polite"
      aria-busy={busy}
      data-testid="status-strip"
      data-operation={operation}
      data-state={serverState}
    >
      <Icon
        className={[
          "mt-0.5 h-4 w-4 shrink-0",
          busy ? "animate-spin text-primary" : "text-muted-foreground",
          variant === "success" && !busy ? "text-success" : "",
          variant === "error" && !busy ? "text-warning-fg" : "",
        ]
          .filter(Boolean)
          .join(" ")}
      />
      <span className="text-[13px] leading-snug text-foreground/90">{message}</span>
    </div>
  );
}