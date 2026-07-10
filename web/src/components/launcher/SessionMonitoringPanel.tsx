import { useCallback, useEffect, useRef, useState } from "react";
import {
  Activity,
  AlertTriangle,
  ChevronDown,
  Copy,
  FileText,
  GitBranch,
  Loader2,
  RefreshCw,
  Save,
  Wifi,
  WifiOff,
} from "lucide-react";
import { Card } from "./primitives";
import { DiscoveryLogViewer } from "./DiscoveryLogViewer";
import type {
  CodexSessionDetail,
  CodexThreadSummary,
  DiscoveryLogEntry,
  FailoverStatus,
  SessionCheckpoint,
} from "../../types";
import type { ServerPillState } from "./primitives";
import { TARGET_APP_NAME } from "../../lib/branding";

const POLL_MS = 10_000;
const DISCOVERY_POLL_MS = 30_000;

function discoveryFingerprint(entries: DiscoveryLogEntry[]): string {
  return entries.map((entry) => `${entry.at}|${entry.event}|${entry.stream}`).join("\n");
}

function formatWhen(iso?: string | null): string {
  if (!iso) return "—";
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleString();
}

function truncate(value: string, max = 120): string {
  if (value.length <= max) return value;
  return `${value.slice(0, max)}…`;
}

function SessionRow({ session }: { session: CodexSessionDetail }) {
  const [open, setOpen] = useState(false);
  const hasPreview = Boolean(session.preview?.content || session.previewError);

  return (
    <li
      data-testid={`session-row-${session.sessionId}`}
      className="rounded-lg border border-border bg-muted/20 text-[12px]"
    >
      <button
        type="button"
        className="flex w-full items-start justify-between gap-2 px-3 py-2 text-left"
        onClick={() => hasPreview && setOpen((value) => !value)}
        disabled={!hasPreview}
      >
        <div className="min-w-0 flex-1">
          <div className="font-mono text-foreground">{session.sessionId}</div>
          <div className="mt-0.5 text-muted-foreground">
            Created {formatWhen(session.createdAt)}
            {session.preview ? (
              <span>
                {" "}
                · {session.preview.role}
                {session.preview.done ? " · done" : " · in progress"}
              </span>
            ) : null}
          </div>
        </div>
        {hasPreview ? (
          <ChevronDown
            className={`mt-0.5 h-4 w-4 shrink-0 text-muted-foreground transition-transform ${open ? "rotate-180" : ""}`}
          />
        ) : null}
      </button>
      {open && session.preview ? (
        <div
          data-testid={`session-preview-${session.sessionId}`}
          className="border-t border-border/70 px-3 py-2"
        >
          <pre className="themed-scrollbar max-h-40 overflow-auto whitespace-pre-wrap font-mono text-[11px] leading-relaxed text-foreground/90">
            {session.preview.content}
          </pre>
        </div>
      ) : null}
      {open && session.previewError ? (
        <p className="border-t border-border/70 px-3 py-2 text-[11px] text-warning-fg">
          Preview unavailable: {session.previewError}
        </p>
      ) : null}
    </li>
  );
}

function ThreadRow({ thread }: { thread: CodexThreadSummary }) {
  return (
    <li
      data-testid={`thread-row-${thread.id}`}
      className="rounded-lg border border-border bg-muted/20 px-3 py-2 text-[12px]"
    >
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <span className="font-medium text-foreground">{thread.name ?? "Untitled thread"}</span>
        {thread.status ? (
          <span className="rounded-full bg-muted px-2 py-0.5 text-[10px] font-medium uppercase tracking-wide text-muted-foreground">
            {thread.status}
          </span>
        ) : null}
      </div>
      <div className="mt-1 font-mono text-[11px] text-muted-foreground">{thread.id}</div>
      <div className="mt-1 text-muted-foreground">
        {thread.model ? `${thread.model} · ` : ""}
        {thread.createdAt ? formatWhen(thread.createdAt) : "—"}
        {thread.path ? ` · ${truncate(thread.path, 48)}` : ""}
      </div>
    </li>
  );
}

export function SessionMonitoringPanel({
  failoverStatus,
  serverState,
  onRefreshWatch,
  onCaptureCheckpoint,
  onCopyResume,
}: {
  failoverStatus: FailoverStatus;
  serverState: ServerPillState;
  onRefreshWatch: () => Promise<void>;
  onCaptureCheckpoint: () => Promise<void>;
  onCopyResume: () => Promise<void>;
}) {
  const [sessions, setSessions] = useState<CodexSessionDetail[]>([]);
  const [threads, setThreads] = useState<CodexThreadSummary[]>([]);
  const [checkpoints, setCheckpoints] = useState<SessionCheckpoint[]>([]);
  const [discoveryEntries, setDiscoveryEntries] = useState<DiscoveryLogEntry[]>([]);
  const [sessionsError, setSessionsError] = useState<string | null>(null);
  const [threadsError, setThreadsError] = useState<string | null>(null);
  const [threadsSource, setThreadsSource] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [capturing, setCapturing] = useState(false);
  const coreFetchRef = useRef(false);
  const discoveryFetchRef = useRef(false);

  const refreshDiscovery = useCallback(async () => {
    if (discoveryFetchRef.current) return;
    discoveryFetchRef.current = true;
    try {
      const discoveryResult = await window.codexRPC.getDiscoveryLogs({ limit: 60 });
      if (!discoveryResult.error) {
        const nextEntries = discoveryResult.data?.entries ?? [];
        setDiscoveryEntries((previous) =>
          discoveryFingerprint(previous) === discoveryFingerprint(nextEntries)
            ? previous
            : nextEntries,
        );
      }
    } finally {
      discoveryFetchRef.current = false;
    }
  }, []);

  const refreshData = useCallback(
    async (silent = false) => {
      if (coreFetchRef.current) return;
      coreFetchRef.current = true;
      if (!silent) {
        setLoading(true);
        setSessionsError(null);
        setThreadsError(null);
        setThreadsSource(null);
      }
      try {
        await onRefreshWatch();
        const [sessionsResult, threadsResult, checkpointsResult] = await Promise.all([
          window.codexRPC.listCodexSessionsDetailed(),
          window.codexRPC.listCodexThreads(),
          window.codexRPC.listSessionCheckpoints(),
        ]);

        if (sessionsResult.error) {
          if (!silent) {
            setSessionsError(sessionsResult.error);
            setSessions([]);
          }
        } else {
          const detail = sessionsResult.data;
          setSessions(detail.sessions ?? []);
          if (!silent && detail.error) setSessionsError(detail.error);
        }

        if (threadsResult.error) {
          if (!silent) {
            setThreadsError(threadsResult.error);
            setThreads([]);
          }
        } else {
          const status = threadsResult.data;
          if (!status.ok && status.error) {
            if (!silent) {
              setThreadsError(status.error);
              setThreads([]);
            }
          } else {
            setThreads(status.threads ?? []);
            setThreadsSource(status.source ?? null);
          }
        }

        if (!checkpointsResult.error) {
          setCheckpoints(checkpointsResult.data?.checkpoints ?? []);
        }

        if (!silent) {
          await refreshDiscovery();
        }
      } catch (error) {
        if (!silent) {
          setSessionsError(error instanceof Error ? error.message : String(error));
        }
      } finally {
        if (!silent) {
          setLoading(false);
        }
        coreFetchRef.current = false;
      }
    },
    [onRefreshWatch, refreshDiscovery],
  );

  useEffect(() => {
    void refreshData();
    const sessionsId = setInterval(() => void refreshData(true), POLL_MS);
    const discoveryId = setInterval(() => void refreshDiscovery(), DISCOVERY_POLL_MS);
    return () => {
      clearInterval(sessionsId);
      clearInterval(discoveryId);
    };
  }, [refreshData, refreshDiscovery]);

  const handleCapture = async () => {
    setCapturing(true);
    try {
      await onCaptureCheckpoint();
      await refreshData();
    } finally {
      setCapturing(false);
    }
  };

  const endpoint = failoverStatus.endpointHealth;
  const activeAlert = failoverStatus.activeAlert;
  const activeConnection = failoverStatus.activeConnectionAlert;

  return (
    <div className="space-y-5" data-testid="session-monitoring-panel">
      <header className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h1 className="text-[22px] font-semibold tracking-tight text-foreground">Sessions</h1>
          <p className="mt-1 text-[13px] text-muted-foreground">
            App-server threads, REST session previews, checkpoints, and discovery logs.
          </p>
        </div>
        <button
          type="button"
          data-testid="refresh-sessions-monitor"
          onClick={() => void refreshData()}
          disabled={loading}
          aria-busy={loading}
          className="inline-flex h-[36px] items-center gap-2 rounded-md border border-input bg-background px-3.5 text-[13px] font-medium text-foreground hover:bg-muted/70 disabled:cursor-not-allowed disabled:opacity-60"
        >
          {loading ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
          ) : (
            <RefreshCw className="h-3.5 w-3.5" />
          )}
          Refresh
        </button>
      </header>

      <Card icon={<Activity className="h-4 w-4" />} title="Watch status">
        <dl className="grid grid-cols-1 gap-3 text-[12px] sm:grid-cols-2">
          <div>
            <dt className="text-muted-foreground">{TARGET_APP_NAME} process</dt>
            <dd className="mt-0.5 font-medium text-foreground capitalize">{serverState}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Background watch</dt>
            <dd className="mt-0.5 font-medium text-foreground">
              {failoverStatus.watching ? "Active" : "Off"}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Auto-switch on limit</dt>
            <dd className="mt-0.5 font-medium text-foreground">
              {failoverStatus.autoSwitch ? "Enabled" : "Manual only"}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">Last poll</dt>
            <dd className="mt-0.5 font-medium text-foreground">
              {formatWhen(failoverStatus.lastPollAt)}
            </dd>
          </div>
          <div className="flex items-start gap-2">
            {failoverStatus.codexApiReady ? (
              <Wifi className="mt-0.5 h-3.5 w-3.5 text-success" />
            ) : (
              <WifiOff className="mt-0.5 h-3.5 w-3.5 text-muted-foreground" />
            )}
            <div>
              <dt className="text-muted-foreground">{TARGET_APP_NAME} API</dt>
              <dd className="mt-0.5 font-medium text-foreground">
                {failoverStatus.codexApiReady ? "Ready" : "Not ready"}
              </dd>
            </div>
          </div>
          <div>
            <dt className="text-muted-foreground">Local endpoint</dt>
            <dd className="mt-0.5 font-medium text-foreground">
              {endpoint?.reachable
                ? `Reachable (${endpoint.modelCount ?? 0} models)`
                : endpoint?.error ?? (failoverStatus.endpointReachable ? "Reachable" : "Unknown")}
            </dd>
          </div>
        </dl>
        {failoverStatus.lastError ? (
          <p className="mt-3 text-[12px] text-warning-fg">{failoverStatus.lastError}</p>
        ) : null}
      </Card>

      {(activeAlert && !activeAlert.dismissed) ||
      (activeConnection && !activeConnection.dismissed) ? (
        <Card icon={<AlertTriangle className="h-4 w-4" />} title="Active notices">
          <div className="space-y-2 text-[12px]">
            {activeAlert && !activeAlert.dismissed ? (
              <p className="rounded-md border border-warning-fg/30 bg-warning-bg/10 px-3 py-2 text-foreground">
                <span className="font-medium">Rate limit: </span>
                {activeAlert.source === "app_server_rate_limits"
                  ? `app-server ${activeAlert.matchedPattern}`
                  : activeAlert.matchedPattern}
              </p>
            ) : null}
            {activeConnection && !activeConnection.dismissed ? (
              <p className="rounded-md border border-border bg-muted/30 px-3 py-2 text-foreground">
                <span className="font-medium">{activeConnection.title}: </span>
                {activeConnection.message}
              </p>
            ) : null}
          </div>
        </Card>
      ) : null}

      <Card icon={<GitBranch className="h-4 w-4" />} title={`${TARGET_APP_NAME} threads (${threads.length})`}>
        {threadsError ? (
          <p className="text-[12px] text-muted-foreground">{threadsError}</p>
        ) : threads.length === 0 ? (
          <p className="text-[12px] text-muted-foreground">
            No threads found yet. Start a conversation in {TARGET_APP_NAME} and refresh.
          </p>
        ) : (
          <>
            {threadsSource ? (
              <p className="mb-2 text-[11px] text-muted-foreground">Source: {threadsSource}</p>
            ) : null}
            <ul className="space-y-2">
              {threads.map((thread) => (
                <ThreadRow key={thread.id} thread={thread} />
              ))}
            </ul>
          </>
        )}
      </Card>

      <Card title={`REST sessions (${sessions.length})`}>
        {sessionsError ? (
          <p className="text-[12px] text-muted-foreground">
            Could not list sessions: {sessionsError}. {TARGET_APP_NAME} must be running with REST /sessions
            available.
          </p>
        ) : sessions.length === 0 ? (
          <p className="text-[12px] text-muted-foreground">
            No sessions returned from the API. Expand a row when previews are available.
          </p>
        ) : (
          <ul className="space-y-2">
            {sessions.map((session) => (
              <SessionRow key={session.sessionId} session={session} />
            ))}
          </ul>
        )}
      </Card>

      <Card title={`Checkpoints (${checkpoints.length})`}>
        <div className="mb-3 flex flex-wrap gap-2">
          <button
            type="button"
            data-testid="capture-checkpoint"
            onClick={() => void handleCapture()}
            disabled={capturing}
            className="inline-flex h-8 items-center gap-1.5 rounded-md border border-input bg-background px-3 text-[11px] font-medium text-foreground hover:bg-muted/70 disabled:opacity-60"
          >
            {capturing ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <Save className="h-3 w-3" />
            )}
            Capture checkpoint
          </button>
          {failoverStatus.lastCheckpoint?.resumePrompt ? (
            <button
              type="button"
              data-testid="copy-checkpoint-resume"
              onClick={() => void onCopyResume()}
              className="inline-flex h-8 items-center gap-1.5 rounded-md border border-input bg-background px-3 text-[11px] font-medium text-foreground hover:bg-muted/70"
            >
              <Copy className="h-3 w-3" />
              Copy latest resume prompt
            </button>
          ) : null}
        </div>

        {checkpoints.length === 0 ? (
          <p className="text-[12px] text-muted-foreground">
            No checkpoints saved yet. Capture one before failover or provider switches.
          </p>
        ) : (
          <ul className="space-y-2">
            {checkpoints.slice(0, 10).map((checkpoint) => (
              <li
                key={checkpoint.id}
                data-testid={`checkpoint-row-${checkpoint.id}`}
                className="rounded-lg border border-border bg-muted/20 px-3 py-2 text-[12px]"
              >
                <div className="flex flex-wrap items-baseline justify-between gap-2">
                  <span className="font-medium text-foreground">{checkpoint.trigger}</span>
                  <span className="text-muted-foreground">{formatWhen(checkpoint.capturedAt)}</span>
                </div>
                <div className="mt-1 text-muted-foreground">
                  {checkpoint.providerMode} · {checkpoint.model ?? "no model"}
                  {checkpoint.sessionId ? ` · ${truncate(checkpoint.sessionId, 24)}` : ""}
                </div>
                {checkpoint.activeGoal ? (
                  <p className="mt-1 text-foreground">{truncate(checkpoint.activeGoal, 160)}</p>
                ) : null}
              </li>
            ))}
          </ul>
        )}
      </Card>

      <Card icon={<FileText className="h-4 w-4" />} title="Discovery logs">
        <DiscoveryLogViewer entries={discoveryEntries} />
        {failoverStatus.discoveryLogHint || failoverStatus.connectionLogHint ? (
          <p className="mt-3 text-[10px] text-muted-foreground">
            {failoverStatus.discoveryLogHint}
            {failoverStatus.connectionLogHint ? ` · ${failoverStatus.connectionLogHint}` : ""}
          </p>
        ) : null}
      </Card>
    </div>
  );
}