import type { CodexRateLimits, CodexRateLimitsStatus, RateLimitWindow } from "../types";

export type UsageTone = "ok" | "warn" | "danger" | "muted";

export interface WindowUsageView {
    key: "primary" | "secondary";
    label: string;
    usedPercent: number | null;
    remainingPercent: number | null;
    resetsAt: number | null;
    resetLabel: string | null;
    resetDateTime: string | null;
    tone: UsageTone;
    available: boolean;
}

const PRIMARY_LABEL = "5-hour";
const SECONDARY_LABEL = "Weekly";

export function remainingPercent(window?: RateLimitWindow | null): number | null {
    if (window?.usedPercent == null) return null;
    return Math.max(0, Math.min(100, 100 - window.usedPercent));
}

export function usageTone(remaining: number | null): UsageTone {
    if (remaining == null) return "muted";
    if (remaining <= 10) return "danger";
    if (remaining <= 30) return "warn";
    return "ok";
}

export function resetTimestampMs(resetsAt?: number | null): number | null {
    if (!resetsAt) return null;
    return resetsAt > 1_000_000_000_000 ? resetsAt : resetsAt * 1000;
}

export function formatResetDateTime(resetsAt?: number | null, now = Date.now()): string | null {
    const resetMs = resetTimestampMs(resetsAt);
    if (!resetMs) return null;

    const date = new Date(resetMs);
    const nowDate = new Date(now);
    const sameYear = date.getFullYear() === nowDate.getFullYear();

    return new Intl.DateTimeFormat(undefined, {
        month: "short",
        day: "numeric",
        ...(sameYear ? {} : { year: "numeric" }),
        hour: "numeric",
        minute: "2-digit",
    }).format(date);
}

export function formatResetTime(resetsAt?: number | null, now = Date.now()): string | null {
    const resetMs = resetTimestampMs(resetsAt);
    if (!resetMs) return null;
    const deltaMs = resetMs - now;
    if (deltaMs <= 0) return "now";

    const totalMinutes = Math.ceil(deltaMs / 60_000);
    if (totalMinutes < 60) return `in ${totalMinutes}m`;

    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;
    if (hours < 24) {
        return minutes > 0 ? `in ${hours}h ${minutes}m` : `in ${hours}h`;
    }

    return new Intl.DateTimeFormat(undefined, {
        month: "short",
        day: "numeric",
        hour: "numeric",
        minute: "2-digit",
    }).format(new Date(resetMs));
}

export function windowUsageView(
    key: "primary" | "secondary",
    label: string,
    window?: RateLimitWindow | null,
): WindowUsageView {
    const usedPercent = window?.usedPercent ?? null;
    const remaining = remainingPercent(window);
    return {
        key,
        label,
        usedPercent,
        remainingPercent: remaining,
        resetsAt: window?.resetsAt ?? null,
        resetLabel: formatResetTime(window?.resetsAt),
        resetDateTime: formatResetDateTime(window?.resetsAt),
        tone: usageTone(remaining),
        available: usedPercent != null,
    };
}

export function buildUsageViews(status?: CodexRateLimitsStatus | null): WindowUsageView[] {
    const limits = status?.rateLimits;
    if (!limits) return [];

    return [
        windowUsageView("primary", PRIMARY_LABEL, limits.primary),
        windowUsageView("secondary", SECONDARY_LABEL, limits.secondary),
    ].filter((view) => view.available);
}

export function compactUsageLine(status?: CodexRateLimitsStatus | null): string | null {
    const views = buildUsageViews(status);
    if (views.length === 0) {
        if (status?.requiresAuth) return "Sign in to cloud CLI";
        if (status?.error) return "Usage unavailable";
        return null;
    }

    return views
        .map((view) => {
            const remaining = view.remainingPercent ?? 0;
            const shortLabel = view.key === "primary" ? "5h" : "weekly";
            const resetSuffix = view.resetDateTime ? ` · resets ${view.resetDateTime}` : "";
            return `${shortLabel} ${remaining}% left${resetSuffix}`;
        })
        .join(" · ");
}

export function isRateLimitReached(status?: CodexRateLimitsStatus | null): boolean {
    return Boolean(status?.rateLimits?.rateLimitReachedType);
}

export function formatPlanType(planType?: string | null): string | null {
    if (!planType) return null;
    return planType.replace(/_/g, " ");
}

export function creditsSummary(limits?: CodexRateLimits | null): string | null {
    const credits = limits?.credits;
    if (!credits?.hasCredits) return null;
    if (credits.unlimited) return "Unlimited credits";
    if (credits.balance) return `$${credits.balance} credits`;
    return null;
}

export const CODEX_USAGE_URL = "https://chatgpt.com/codex/settings/usage";

export function toneClass(tone: UsageTone): string {
    switch (tone) {
        case "danger":
            return "text-destructive";
        case "warn":
            return "text-warning-fg";
        case "ok":
            return "text-foreground";
        default:
            return "text-muted-foreground";
    }
}

export function barClass(tone: UsageTone): string {
    switch (tone) {
        case "danger":
            return "bg-destructive";
        case "warn":
            return "bg-warning-fg";
        case "ok":
            return "bg-primary";
        default:
            return "bg-muted-foreground/40";
    }
}