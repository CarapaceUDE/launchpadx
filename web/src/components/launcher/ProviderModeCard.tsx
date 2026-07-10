import { useState, type KeyboardEvent, type ReactNode } from "react";
import { Cloud, Loader2, Play, RefreshCw, Server, Settings, Square } from "lucide-react";
import { Card, type ServerPillState } from "./primitives";
import { StatusStrip } from "./StatusStrip";
import { MarqueeSelect } from "./MarqueeSelect";
import type { LauncherOperation } from "../../lib/operationStatus";
import {
  activeProviderMode,
  activeProviderSummary,
  profileStillOnLocalProvider,
  providerModeDescription,
  providerModeLabel,
  type CodexProfileState,
  type ProviderMode,
} from "../../lib/lpadProfile";
import {
  blocksProviderSwitch,
  codexAccountSwitchWarnings,
} from "../../lib/providerGuards";
import type { CodexRateLimitsStatus } from "../../types";
import { LaunchPadXRateLimitsCompact } from "./LaunchPadXRateLimitsPanel";
import { APP_NAME, TARGET_APP_NAME } from "../../lib/branding";

function ProviderSegment({
  mode,
  selected,
  switching,
  switchingTo,
  disabled,
  onSelect,
  onOpenSettings,
  children,
}: {
  mode: ProviderMode;
  selected: boolean;
  switching?: boolean;
  switchingTo?: ProviderMode | null;
  disabled?: boolean;
  onSelect: () => void;
  onOpenSettings: () => void;
  children?: ReactNode;
}) {
  const busy = switching && switchingTo === mode;
  const Icon = mode === "codex" ? Cloud : Server;

  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      if (!disabled && !switching) onSelect();
    }
  };

  const activate = () => {
    if (!disabled && !switching) onSelect();
  };

  return (
    <div
      role="tab"
      tabIndex={disabled ? -1 : 0}
      data-testid={`provider-mode-${mode}`}
      aria-selected={selected}
      aria-busy={busy}
      onKeyDown={handleKeyDown}
      className={[
        "relative flex min-w-0 flex-col gap-2 rounded-lg border px-3 py-3 text-left transition-all outline-none focus-visible:ring-2 focus-visible:ring-primary/50 sm:px-3.5",
        selected
          ? "border-primary/60 bg-primary/10 text-foreground shadow-[inset_3px_0_0_0_var(--color-primary)] ring-2 ring-primary/35"
          : "border-transparent text-muted-foreground hover:border-border/80 hover:bg-card/60 hover:text-foreground",
        disabled ? "opacity-60" : "",
        busy ? "opacity-80" : "",
      ].join(" ")}
    >
      <button
        type="button"
        data-testid={`provider-activate-${mode}`}
        onClick={activate}
        disabled={disabled || switching}
        className={[
          "w-full rounded-md text-left transition-colors",
          disabled || switching ? "cursor-not-allowed" : "cursor-pointer hover:bg-muted/30",
        ].join(" ")}
      >
        <div className="flex items-start justify-between gap-2 pr-0">
          <span className="flex min-w-0 items-center gap-2 text-[13px] font-semibold">
            {busy ? (
              <Loader2 className="h-4 w-4 shrink-0 animate-spin text-primary" />
            ) : (
              <Icon className={`h-4 w-4 shrink-0 ${selected ? "text-primary" : ""}`} />
            )}
            {providerModeLabel(mode)}
            {selected ? (
              <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-primary">
                Active
              </span>
            ) : null}
          </span>
        </div>
        <span className="mt-1 block text-[11px] leading-snug opacity-80">
          {providerModeDescription(mode)}
        </span>
      </button>

      <button
        type="button"
        data-testid={`provider-settings-${mode}`}
        aria-label="Provider settings"
        onClick={onOpenSettings}
        className="absolute right-2.5 top-2.5 grid h-6 w-6 place-items-center rounded-md text-muted-foreground transition-colors hover:bg-muted/80 hover:text-foreground"
      >
        <Settings className="h-3.5 w-3.5" />
      </button>

      {children ? <div className="mt-0.5 w-full min-w-0">{children}</div> : null}
    </div>
  );
}

export function ProviderModeCard({
  profile,
  activeMode,
  canActivateLocal,
  localBlockedReason,
  switching,
  switchingTo,
  onSelectMode,
  onOpenProviderSettings,
  endpointPreview,
  modelPreview,
  models,
  selectedModel,
  onSelectModel,
  onRefreshModels,
  refreshing,
  selectingModel,
  serverState,
  operation,
  onToggleLaunch,
  canStart,
  startBlockedReason,
  statusStripText,
  statusVariant,
  rateLimitsStatus,
  rateLimitsLoading,
}: {
  profile: CodexProfileState;
  activeMode: ProviderMode;
  canActivateLocal: boolean;
  localBlockedReason?: string;
  switching?: boolean;
  switchingTo?: ProviderMode | null;
  onSelectMode: (mode: ProviderMode) => void;
  onOpenProviderSettings: (mode: ProviderMode) => void;
  endpointPreview?: string;
  modelPreview?: string;
  models: string[];
  selectedModel: string;
  onSelectModel: (v: string) => void;
  onRefreshModels: () => void;
  refreshing: boolean;
  selectingModel?: boolean;
  serverState: ServerPillState;
  operation: LauncherOperation;
  onToggleLaunch: () => void;
  canStart: boolean;
  startBlockedReason?: string;
  statusStripText: string;
  statusVariant?: "default" | "success" | "error";
  rateLimitsStatus: CodexRateLimitsStatus | null;
  rateLimitsLoading?: boolean;
}) {
  const [confirmCodex, setConfirmCodex] = useState(false);
  const resolvedMode = activeProviderMode(profile);
  const summary = activeProviderSummary(resolvedMode, profile, modelPreview, endpointPreview);
  const switchBlock = blocksProviderSwitch(operation);
  const switchBlocked = !switchBlock.ok;
  const switchBlockedReason = switchBlock.ok ? undefined : switchBlock.message;
  const codexRunning = serverState === "running";
  const codexSwitchWarnings = codexAccountSwitchWarnings(profile, codexRunning);

  const isLaunching = operation === "launching" || operation === "waiting_for_codex";
  const isStopping = operation === "stopping";
  const launchBusy = isLaunching || isStopping;
  const running = serverState === "running";

  const launchLabel = isStopping
    ? "Stopping..."
    : isLaunching
      ? "Starting..."
      : running
        ? `Stop ${TARGET_APP_NAME}`
        : `Start ${TARGET_APP_NAME}`;

  const handleSelect = (mode: ProviderMode) => {
    if (switching || switchBlocked) return;
    if (mode === "codex") {
      if (resolvedMode !== "codex" || profileStillOnLocalProvider(profile)) {
        setConfirmCodex(true);
      }
      return;
    }
    if (mode === resolvedMode) {
      if (mode === "local" && canActivateLocal) onSelectMode("local");
      return;
    }
    if (!canActivateLocal) return;
    onSelectMode("local");
  };

  const handleConfirmCodex = () => {
    setConfirmCodex(false);
    onSelectMode("codex");
  };

  const modelOptions = models.map((m) => ({ value: m, label: m }));

  return (
    <>
      <Card className="!p-4 sm:!p-5" data-testid="provider-mode-card">
        <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div className="min-w-0 flex-1">
            <h2 className="text-[16px] font-semibold tracking-tight text-foreground">{APP_NAME}</h2>
            <p className="mt-1 text-[13px] text-muted-foreground">
              Choose a model provider, then start {TARGET_APP_NAME} when you are ready.
            </p>
          </div>

          <div className="flex w-full min-w-0 flex-col gap-2 sm:w-auto sm:max-w-full sm:flex-row sm:flex-wrap sm:items-center sm:justify-end">
            <StatusStrip
              message={statusStripText}
              operation={operation}
              variant={statusVariant}
              compact
              serverState={serverState}
            />
            <button
              type="button"
              data-testid="launch-toggle"
              onClick={onToggleLaunch}
              disabled={launchBusy || (!running && !canStart)}
              aria-busy={launchBusy}
              title={running ? `Stop ${TARGET_APP_NAME}` : `Start ${TARGET_APP_NAME}`}
              className={[
                "inline-flex h-[34px] w-full shrink-0 items-center justify-center gap-1.5 rounded-md px-3.5 text-[13px] font-semibold transition-colors sm:w-auto",
                running && !launchBusy
                  ? "bg-[color:var(--color-warning-fg)] text-white hover:opacity-90"
                  : "bg-primary text-primary-foreground hover:bg-primary-hover",
                "disabled:cursor-not-allowed disabled:bg-border disabled:text-muted-foreground",
                launchBusy ? "opacity-90" : "",
              ].join(" ")}
            >
              {launchBusy ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : running ? (
                <Square className="h-3.5 w-3.5" fill="currentColor" />
              ) : (
                <Play className="h-3.5 w-3.5" fill="currentColor" />
              )}
              {launchLabel}
            </button>
          </div>
        </div>

        <div
          role="tablist"
          aria-label="Model provider"
          className="grid grid-cols-1 gap-2 rounded-xl border border-border bg-muted/40 p-1 xl:grid-cols-2 xl:gap-1"
        >
          <ProviderSegment
            mode="codex"
            selected={resolvedMode === "codex"}
            switching={switching}
            switchingTo={switchingTo}
            disabled={switching || switchBlocked}
            onSelect={() => handleSelect("codex")}
            onOpenSettings={() => onOpenProviderSettings("codex")}
          >
            <LaunchPadXRateLimitsCompact status={rateLimitsStatus} loading={rateLimitsLoading} />
          </ProviderSegment>

          <ProviderSegment
            mode="local"
            selected={resolvedMode === "local"}
            switching={switching}
            switchingTo={switchingTo}
            disabled={switching || switchBlocked}
            onSelect={() => handleSelect("local")}
            onOpenSettings={() => onOpenProviderSettings("local")}
          >
            <MarqueeSelect
              testId="model-select"
              value={selectedModel}
              options={modelOptions}
              placeholder="Model..."
              emptyLabel="No models"
              disabled={selectingModel}
              busy={selectingModel}
              onChange={onSelectModel}
            />
            <button
              type="button"
              data-testid="refresh-models"
              onClick={onRefreshModels}
              disabled={refreshing}
              aria-busy={refreshing}
              title="Refresh models"
              className="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md border border-input bg-background text-foreground hover:bg-muted/70 disabled:cursor-not-allowed disabled:opacity-60"
            >
              <RefreshCw className={`h-3 w-3 ${refreshing ? "animate-spin" : ""}`} />
            </button>
          </ProviderSegment>
        </div>

        {switchBlocked && switchBlockedReason ? (
          <p className="mt-3 text-[12px] text-warning-fg">{switchBlockedReason}</p>
        ) : null}

        {!canActivateLocal && localBlockedReason && resolvedMode !== "local" ? (
          <p className="mt-3 text-[12px] text-warning-fg">{localBlockedReason}</p>
        ) : null}

        {!running && !canStart && !launchBusy && startBlockedReason ? (
          <p className="mt-3 text-[12px] text-warning-fg">{startBlockedReason}</p>
        ) : null}

        <div
          data-testid="provider-mode-status"
          className="mt-4 rounded-lg border border-border bg-muted/30 px-3.5 py-2.5 text-[12px] leading-relaxed"
          aria-live="polite"
          title={summary}
        >
          <span className="block break-words font-medium text-foreground">{summary}</span>
          {activeMode !== resolvedMode ? (
            <span className="ml-2 text-warning-fg">· Pending apply</span>
          ) : null}
        </div>
      </Card>

      {confirmCodex && (
        <div
          data-testid="provider-confirm-dialog"
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
          onClick={() => setConfirmCodex(false)}
        >
          <div
            className="w-full max-w-md rounded-xl border border-border bg-card p-6 shadow-xl"
            onClick={(e) => e.stopPropagation()}
            role="alertdialog"
            aria-labelledby="provider-confirm-title"
            aria-describedby="provider-confirm-description"
          >
            <h3 id="provider-confirm-title" className="text-[16px] font-semibold text-foreground">
              Switch to Cloud Account?
            </h3>
            <p
              id="provider-confirm-description"
              className="mt-2 text-[13px] leading-relaxed text-muted-foreground"
            >
              {TARGET_APP_NAME} will use your account sign-in again. Local API settings are removed
              from your profile, and your previous settings are restored when available.
            </p>
            {codexSwitchWarnings.length > 0 ? (
              <ul className="mt-3 space-y-1.5 text-[12px] leading-relaxed text-warning-fg">
                {codexSwitchWarnings.map((warning) => (
                  <li key={warning} className="flex gap-2">
                    <span aria-hidden="true">•</span>
                    <span>{warning}</span>
                  </li>
                ))}
              </ul>
            ) : null}
            <div className="mt-5 flex justify-end gap-2">
              <button
                type="button"
                data-testid="provider-confirm-cancel"
                onClick={() => setConfirmCodex(false)}
                className="inline-flex h-[36px] items-center rounded-md border border-input bg-background px-4 text-[13px] font-semibold text-foreground transition-colors hover:bg-muted/70"
              >
                Cancel
              </button>
              <button
                type="button"
                data-testid="provider-confirm-switch"
                onClick={handleConfirmCodex}
                className="inline-flex h-[36px] items-center rounded-md bg-primary px-4 text-[13px] font-semibold text-primary-foreground transition-colors hover:bg-primary-hover"
              >
                Switch provider
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}