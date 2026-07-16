import type { CodexRateLimits, CodexRateLimitsStatus, RateLimitWindow } from "../types";

export type UsageTone = "ok" | "warn" | "danger" | "muted";
export type UsageWindowSlot = "primary" | "secondary";

export interface UsageWindowEntry {
    slot: UsageWindowSlot;
    window: RateLimitWindow;
}

export interface WindowUsageView {
    key: string;
    slot: UsageWindowSlot;
    label: string;
    shortLabel: string;
    windowDurationMins: number | null;
    usedPercent: number | null;
    remainingPercent: number | null;
    resetsAt: number | null;
    resetLabel: string | null;
    resetDateTime: string | null;
    tone: UsageTone;
    available: boolean;
}

export function formatWindowLabel(windowDurationMins?: number | null): string {
    if (windowDurationMins != null) {
        if (windowDurationMins <= 360) {
            const hours = Math.max(1, Math.round(windowDurationMins / 60));
            return hours === 1 ? "1-hour" : `${hours}-hour`;
        }
        if (windowDurationMins === 10_080) return "Weekly";
        if (windowDurationMins === 43_200) return "Monthly";
        if (windowDurationMins % 10_080 === 0) {
            const weeks = windowDurationMins / 10_080;
            return weeks === 1 ? "Weekly" : `${weeks}-week`;
        }
        if (windowDurationMins % 1_440 === 0) {
            const days = windowDurationMins / 1_440;
            return days === 1 ? "Daily" : `${days}-day`;
        }
        return `${windowDurationMins}m window`;
    }

    return "Usage window";
}

export function formatWindowShortLabel(windowDurationMins?: number | null): string {
    if (windowDurationMins != null) {
        if (windowDurationMins <= 360) {
            const hours = Math.max(1, Math.round(windowDurationMins / 60));
            return `${hours}h`;
        }
        if (windowDurationMins === 10_080) return "weekly";
        if (windowDurationMins === 43_200) return "monthly";
        if (windowDurationMins % 10_080 === 0) {
            const weeks = windowDurationMins / 10_080;
            return weeks === 1 ? "weekly" : `${weeks}w`;
        }
        if (windowDurationMins % 1_440 === 0) {
            const days = windowDurationMins / 1_440;
            return days === 1 ? "daily" : `${days}d`;
        }
        return `${windowDurationMins}m`;
    }

    return "usage";
}

export function usageWindowKey(window: RateLimitWindow, slot: UsageWindowSlot, index: number): string {
    if (window.windowDurationMins != null) {
        return `w-${window.windowDurationMins}`;
    }
    return `w-${slot}-${index}`;
}

/** Gather every populated bucket, dedupe by duration, shortest window first. */
export function collectUsageWindows(limits?: CodexRateLimits | null): UsageWindowEntry[] {
    if (!limits) return [];

    const candidates: UsageWindowEntry[] = [];
    if (limits.primary?.usedPercent != null) {
        candidates.push({ slot: "primary", window: limits.primary });
    }
    if (limits.secondary?.usedPercent != null) {
        candidates.push({ slot: "secondary", window: limits.secondary });
    }

    const byDuration = new Map<number, UsageWindowEntry>();
    const withoutDuration: UsageWindowEntry[] = [];

    for (const entry of candidates) {
        const mins = entry.window.windowDurationMins;
        if (mins == null) {
            withoutDuration.push(entry);
            continue;
        }
        if (!byDuration.has(mins)) {
            byDuration.set(mins, entry);
        }
    }

    const sorted = [...byDuration.values()].sort(
        (a, b) => (a.window.windowDurationMins ?? 0) - (b.window.windowDurationMins ?? 0),
    );

    return [...sorted, ...withoutDuration];
}

export function formatRateLimitReachedType(
    type?: string | null,
    limits?: CodexRateLimits | null,
): string | null {
    if (!type) return null;

    if (type === "primary" || type === "secondary") {
        const slotted = type === "primary" ? limits?.primary : limits?.secondary;
        if (slotted?.windowDurationMins != null) {
            return `${formatWindowLabel(slotted.windowDurationMins)} window`;
        }

        const windows = collectUsageWindows(limits);
        if (windows.length === 1 && windows[0].window.windowDurationMins != null) {
            return `${formatWindowLabel(windows[0].window.windowDurationMins)} window`;
        }
    }

    const labels: Record<string, string> = {
        rate_limit_reached: "usage limit",
        workspace_owner_credits_depleted: "workspace credits depleted",
        workspace_member_credits_depleted: "member credits depleted",
        workspace_owner_usage_limit_reached: "workspace usage limit",
        workspace_member_usage_limit_reached: "member usage limit",
    };

    return labels[type] ?? type.replace(/_/g, " ");
}

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

export function windowUsageView(entry: UsageWindowEntry, index: number): WindowUsageView {
    const { slot, window } = entry;
    const usedPercent = window.usedPercent ?? null;
    const remaining = remainingPercent(window);
    const windowDurationMins = window.windowDurationMins ?? null;

    return {
        key: usageWindowKey(window, slot, index),
        slot,
        label: formatWindowLabel(windowDurationMins),
        shortLabel: formatWindowShortLabel(windowDurationMins),
        windowDurationMins,
        usedPercent,
        remainingPercent: remaining,
        resetsAt: window.resetsAt ?? null,
        resetLabel: formatResetTime(window.resetsAt),
        resetDateTime: formatResetDateTime(window.resetsAt),
        tone: usageTone(remaining),
        available: usedPercent != null,
    };
}

export function buildUsageViews(status?: CodexRateLimitsStatus | null): WindowUsageView[] {
    return collectUsageWindows(status?.rateLimits).map((entry, index) => windowUsageView(entry, index));
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
            const resetSuffix = view.resetDateTime ? ` · resets ${view.resetDateTime}` : "";
            return `${view.shortLabel} ${remaining}% left${resetSuffix}`;
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