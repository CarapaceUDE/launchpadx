import { useState } from "react";
import type { DiscoveryLogEntry } from "../../types";

type StreamFilter = "all" | "rateLimit" | "connection";

function formatWhen(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function streamLabel(stream: DiscoveryLogEntry["stream"]): string {
  return stream === "rateLimit" ? "Rate limit" : "Connection";
}

function summarizeDetails(entry: DiscoveryLogEntry): string {
  const details = entry.details;
  const candidates = [
    details.matchedPattern,
    details.rateLimitReachedType,
    details.error,
    details.note,
    details.message,
    details.snippet,
  ];
  for (const value of candidates) {
    if (typeof value === "string" && value.trim()) {
      return value.length > 120 ? `${value.slice(0, 120)}…` : value;
    }
  }
  const raw = JSON.stringify(details);
  return raw.length > 140 ? `${raw.slice(0, 140)}…` : raw;
}

function entryKey(entry: DiscoveryLogEntry): string {
  return `${entry.at}|${entry.event}|${entry.stream}|${entry.source}`;
}

export function DiscoveryLogViewer({ entries }: { entries: DiscoveryLogEntry[] }) {
  const [filter, setFilter] = useState<StreamFilter>("all");
  const filtered = entries.filter((entry) => {
    if (filter === "all") return true;
    return entry.stream === filter;
  });

  return (
    <div data-testid="discovery-log-viewer">
      <div className="mb-2 flex flex-wrap gap-2">
        {(
          [
            ["all", "All"],
            ["rateLimit", "Rate limit"],
            ["connection", "Connection"],
          ] as const
        ).map(([key, label]) => (
          <button
            key={key}
            type="button"
            data-testid={`discovery-filter-${key}`}
            onClick={() => setFilter(key)}
            className={[
              "rounded-full px-2.5 py-1 text-[11px] font-medium transition-colors",
              filter === key
                ? "bg-primary/15 text-primary"
                : "bg-muted/50 text-muted-foreground hover:text-foreground",
            ].join(" ")}
          >
            {label}
          </button>
        ))}
      </div>

      {filtered.length === 0 ? (
        <p className="text-[12px] text-muted-foreground">
          No discovery events yet. Background watch writes to rate-limit and connection JSONL
          files when limits, reconnects, or session errors occur.
        </p>
      ) : (
        <div className="themed-scrollbar max-h-[280px] overflow-y-auto rounded-md border border-border/70">
          <ul className="divide-y divide-border/60">
            {filtered.map((entry) => (
              <li
                key={entryKey(entry)}
                data-testid={`discovery-entry-${entry.event}`}
                className="grid grid-cols-[minmax(0,1fr)_auto] gap-x-3 gap-y-0.5 px-2.5 py-1.5 text-[11px] sm:grid-cols-[9.5rem_7.5rem_minmax(0,1fr)]"
              >
                <span className="truncate text-muted-foreground">{formatWhen(entry.at)}</span>
                <span className="truncate font-medium text-foreground">{entry.event}</span>
                <span className="col-span-2 truncate text-muted-foreground sm:col-span-1">
                  {streamLabel(entry.stream)}
                </span>
                <span
                  className="col-span-2 truncate font-mono text-[10px] text-foreground/80 sm:col-span-3"
                  title={summarizeDetails(entry)}
                >
                  {summarizeDetails(entry)}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}