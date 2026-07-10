import { useEffect, useRef, useState } from "react";
import { RefreshCw, Trash2 } from "lucide-react";
import { useLaunchPadX, type LogEntry } from "../../context/LaunchPadXContext";

function isWatchLog(message: string) {
  return message.includes("RATE_LIMIT_WATCH") || message.includes("CONNECTION_WATCH");
}

function levelColor(level: string, message: string) {
  if (isWatchLog(message)) return "text-amber-400";
  switch (level.toUpperCase()) {
    case "ERROR":
      return "text-red-500";
    case "WARN":
      return "text-yellow-500";
    case "FATAL":
      return "text-red-600";
    default:
      return "text-muted-foreground";
  }
}

function levelBg(level: string, message: string) {
  if (isWatchLog(message)) return "bg-amber-500/15";
  switch (level.toUpperCase()) {
    case "ERROR":
      return "bg-red-500/10";
    case "WARN":
      return "bg-yellow-500/10";
    case "FATAL":
      return "bg-red-500/15";
    default:
      return "bg-muted/30";
  }
}

export function LogsPanel() {
  const { getAppLogs } = useLaunchPadX();
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const [loading, setLoading] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const initialized = useRef(false);

  const fetchLogs = async () => {
    setLoading(true);
    try {
      setLogs(await getAppLogs());
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (initialized.current) return;
    initialized.current = true;
    void fetchLogs();
    const interval = setInterval(() => void fetchLogs(), 10000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (autoScroll && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [logs, autoScroll]);

  return (
    <div className="card-surface space-y-4 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-[14px] font-semibold text-foreground">Application Logs</h3>
          <p className="text-[11px] text-muted-foreground">
            Watch logs use highlighted <code className="text-[10px]">RATE_LIMIT_WATCH</code> and{" "}
            <code className="text-[10px]">CONNECTION_WATCH</code> entries (also saved to JSONL discovery files).
          </p>
        </div>
        <div className="flex items-center gap-3">
          <label className="flex cursor-pointer select-none items-center gap-1.5 text-[11px] text-muted-foreground">
            <input
              type="checkbox"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.target.checked)}
              className="h-3 w-3 accent-primary"
            />
            Auto-scroll
          </label>
          <button
            onClick={() => setLogs([])}
            className="text-muted-foreground transition-colors hover:text-foreground"
            title="Clear logs"
          >
            <Trash2 className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>

      <div
        ref={containerRef}
        className="themed-scrollbar max-h-[250px] space-y-1 overflow-y-auto pr-1 font-mono text-[11px]"
      >
        {logs.length === 0 ? (
          <p className="py-6 text-center italic text-muted-foreground/50">No log entries available.</p>
        ) : (
          logs.map((entry, i) => (
            <div
              key={i}
              className={`rounded px-2.5 py-1.5 leading-relaxed ${levelBg(entry.level, entry.message)}`}
            >
              <span className={`mr-2 font-semibold ${levelColor(entry.level, entry.message)}`}>
                [{entry.level}]
              </span>
              <span className="text-foreground/80">{entry.message}</span>
            </div>
          ))
        )}
      </div>

      <button
        onClick={() => void fetchLogs()}
        disabled={loading}
        className="flex w-full items-center justify-center gap-1.5 rounded-lg border border-input bg-background px-3 py-2 text-[11px] text-muted-foreground transition-colors hover:bg-muted/70 disabled:cursor-not-allowed disabled:opacity-50"
      >
        <RefreshCw className={`h-3 w-3 ${loading ? "animate-spin" : ""}`} />
        {loading ? "Loading..." : "Refresh Logs"}
      </button>
    </div>
  );
}