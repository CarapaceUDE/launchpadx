import { ExternalLink, Loader2, RefreshCw } from "lucide-react";
import type { CodexRateLimitsStatus } from "../../types";
import {
    CODEX_USAGE_URL,
    barClass,
    buildUsageViews,
    creditsSummary,
    formatPlanType,
    formatRateLimitReachedType,
    isRateLimitReached,
    toneClass,
} from "../../lib/rateLimits";
import { TARGET_APP_NAME } from "../../lib/branding";

export function LaunchPadXRateLimitsPanel({
    status,
    loading,
    onRefresh,
}: {
    status: CodexRateLimitsStatus | null;
    loading?: boolean;
    onRefresh: () => void;
}) {
    const views = buildUsageViews(status);
    const reached = isRateLimitReached(status);
    const credits = creditsSummary(status?.rateLimits);
    const plan = formatPlanType(status?.planType ?? status?.rateLimits?.planType);
    const reachedLabel = formatRateLimitReachedType(
        status?.rateLimits?.rateLimitReachedType,
        status?.rateLimits,
    );

    return (
        <div className="space-y-3" data-testid="codex-rate-limits-panel">
            <div className="flex items-center justify-between gap-2">
                <div>
                    <p className="text-[13px] font-medium text-foreground">Usage limits</p>
                    <p className="text-[11px] text-muted-foreground">
                        Shared agentic usage from your signed-in cloud account
                    </p>
                </div>
                <button
                    type="button"
                    data-testid="refresh-rate-limits"
                    onClick={onRefresh}
                    disabled={loading}
                    aria-busy={loading}
                    className="inline-flex h-8 items-center gap-1.5 rounded-md border border-input bg-background px-2.5 text-[11px] font-medium text-foreground hover:bg-muted/70 disabled:cursor-not-allowed disabled:opacity-60"
                >
                    {loading ? (
                        <Loader2 className="h-3 w-3 animate-spin" />
                    ) : (
                        <RefreshCw className="h-3 w-3" />
                    )}
                    Refresh
                </button>
            </div>

            {reached ? (
                <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-[12px] text-destructive">
                    {TARGET_APP_NAME} reports a usage limit reached
                    {reachedLabel ? ` (${reachedLabel})` : ""}.
                </p>
            ) : null}

            {views.length > 0 ? (
                <div className="space-y-3">
                    {views.map((view) => {
                        const fill = view.usedPercent ?? 0;
                        return (
                            <div key={view.key} data-testid={`rate-limit-${view.key}`}>
                                <div className="mb-1 flex items-baseline justify-between gap-2 text-[12px]">
                                    <span className="font-medium text-foreground">{view.label}</span>
                                    <span
                                        className={toneClass(view.tone)}
                                        title={view.resetLabel ? `Resets ${view.resetLabel}` : undefined}
                                    >
                                        {view.remainingPercent}% left
                                        {view.resetDateTime ? ` · resets ${view.resetDateTime}` : ""}
                                    </span>
                                </div>
                                <div className="h-2 overflow-hidden rounded-full bg-muted/80">
                                    <div
                                        className={`h-full rounded-full transition-all ${barClass(view.tone)}`}
                                        style={{ width: `${Math.max(4, fill)}%` }}
                                    />
                                </div>
                            </div>
                        );
                    })}
                </div>
            ) : (
                <p className="text-[12px] text-muted-foreground">
                    {status?.requiresAuth
                        ? "Run `codex login` on this machine to load account usage."
                        : status?.error
                          ? status.error
                          : `Usage data is not available yet. Refresh after ${TARGET_APP_NAME} CLI is installed and signed in.`}
                </p>
            )}

            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-muted-foreground">
                {plan ? <span>Plan: {plan}</span> : null}
                {credits ? <span>{credits}</span> : null}
                {status?.rateLimitResetCredits?.availableCount != null ? (
                    <span>{status.rateLimitResetCredits.availableCount} reset credits</span>
                ) : null}
            </div>

            <a
                href={CODEX_USAGE_URL}
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center gap-1 text-[11px] font-medium text-primary hover:underline"
            >
                View usage on chatgpt.com
                <ExternalLink className="h-3 w-3" />
            </a>
        </div>
    );
}

export function LaunchPadXRateLimitsCompact({
    status,
    loading,
}: {
    status: CodexRateLimitsStatus | null;
    loading?: boolean;
}) {
    const views = buildUsageViews(status);
    const reached = isRateLimitReached(status);

    if (loading && views.length === 0) {
        return (
            <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground">
                <Loader2 className="h-3 w-3 animate-spin" />
                Loading usage...
            </div>
        );
    }

    if (views.length === 0) {
        return null;
    }

    return (
        <div
            className="flex w-full flex-col gap-0.5 text-[10px] leading-snug"
            data-testid="codex-rate-limits-compact"
        >
            {reached ? (
                <span className="font-semibold text-destructive">Limit reached</span>
            ) : null}
            {views.map((view) => (
                <span
                    key={view.key}
                    className={toneClass(view.tone)}
                    title={view.resetLabel ? `Resets ${view.resetLabel}` : undefined}
                >
                    {view.shortLabel} {view.remainingPercent}% left
                    {view.resetDateTime ? (
                        <span className="text-muted-foreground"> · resets {view.resetDateTime}</span>
                    ) : null}
                </span>
            ))}
        </div>
    );
}