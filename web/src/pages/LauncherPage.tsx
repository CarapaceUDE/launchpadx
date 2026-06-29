import { useCallback, useEffect, useMemo, useState } from "react";
import type { LauncherConfig, ModelInfo } from "../types";
import {
  useLauncher,
  configToCodexForm,
  type NavKey,
  type CodexConfigForm,
} from "../context/LauncherContext";
import { Sidebar } from "../components/launcher/Sidebar";
import { ProviderModeCard } from "../components/launcher/ProviderModeCard";
import { ProviderSettingsPanel } from "../components/launcher/ProviderSettingsPanel";
import { ModelsPanel } from "../components/launcher/ModelsPanel";
import { LogsPanel } from "../components/launcher/LogsPanel";
import { AboutPanel } from "../components/launcher/AboutPanel";
import { ModelDetailsModal } from "../components/launcher/ModelDetailsModal";
import { buildOpenAiBaseUrl } from "../lib/endpoint";
import { reconcileModelSelection } from "../lib/modelSelection";
import { activeProviderMode, type ProviderMode } from "../lib/codexProfile";
import { canStartCodex, localActivationRequirements } from "../lib/providerGuards";
import { APP_NAME } from "../lib/branding";
import { ConnectionBanner } from "../components/launcher/ConnectionBanner";
import { FailoverBanner } from "../components/launcher/FailoverBanner";

export function LauncherPage() {
  const {
    statusMessage,
    statusVariant,
    operation,
    serverState,
    models,
    config,
    refreshing,
    launch,
    stop,
    switchProvider,
    refreshModels,
    updateConfig,
    selectModel,
    setAutoStart,
    codexProfile,
    writingCodex,
    revertingCodex,
    failoverStatus,
    failoverToLocal,
    dismissFailoverAlert,
    dismissConnectionAlert,
    copyResumePrompt,
  } = useLauncher();

  const [activeNav, setActiveNav] = useState<NavKey>("launcher");
  const [settingsProvider, setSettingsProvider] = useState<ProviderMode>("local");
  const [detailsModel, setDetailsModel] = useState<ModelInfo | null>(null);
  const [codexForm, setCodexForm] = useState<CodexConfigForm>(configToCodexForm({}));

  useEffect(() => {
    if (Object.keys(config).length > 0) {
      setCodexForm(configToCodexForm(config));
    }
  }, [config]);

  const ip = config.ollamaIp ?? "127.0.0.1";
  const port = String(config.ollamaPort ?? 11434);
  const scheme = (config.ollamaScheme as "http" | "https") ?? "http";
  const autoStart = Boolean(config.autoStart);
  const workingDir = config.workingDirectory ?? "";
  const apiKey = config.apiKey ?? "";
  const modelNames = models.map((m) => m.name);
  const selectedModel = reconcileModelSelection(modelNames, config.codexModel) ?? "";
  const baseUrl = buildOpenAiBaseUrl(ip, parseInt(port, 10) || 11434, scheme);

  const providerMode = activeProviderMode(codexProfile);
  const switchingProvider = writingCodex || revertingCodex;
  const switchingTo = writingCodex ? "local" : revertingCodex ? "codex" : null;

  const localReq = localActivationRequirements(config, models.length);
  const canActivateLocal = localReq.ok;
  const localBlockedReason = localReq.ok ? undefined : localReq.message;

  const startCheck = canStartCodex(providerMode, config, models.length);
  const canStart = startCheck.canStart;
  const startBlockedReason = startCheck.reason;

  const handleEndpointChange = useCallback(
    (patch: { ip?: string; port?: string; scheme?: "http" | "https" }) => {
      const nextIp = patch.ip ?? ip;
      const nextPort = patch.port !== undefined ? parseInt(patch.port, 10) || 11434 : parseInt(port, 10) || 11434;
      const nextScheme = patch.scheme ?? scheme;

      if (patch.ip !== undefined) updateConfig("ollamaIp", patch.ip);
      if (patch.port !== undefined) updateConfig("ollamaPort", nextPort);
      if (patch.scheme !== undefined) updateConfig("ollamaScheme", patch.scheme);

      const nextBaseUrl = buildOpenAiBaseUrl(nextIp, nextPort, nextScheme);
      if (nextBaseUrl) updateConfig("openaiBaseUrl", nextBaseUrl);
    },
    [ip, port, scheme, updateConfig],
  );

  const handleCodexFormChange = useCallback(
    (patch: Partial<CodexConfigForm>) => {
      setCodexForm((prev) => ({ ...prev, ...patch }));
      if (patch.codexProviderId !== undefined) updateConfig("codexProviderId", patch.codexProviderId);
      if (patch.codexProviderName !== undefined)
        updateConfig("codexProviderName", patch.codexProviderName);
      if (patch.codexConfigPath !== undefined) updateConfig("codexConfigPath", patch.codexConfigPath);
      if (patch.codexCommand !== undefined) updateConfig("codexCommand", patch.codexCommand);
      if (patch.codexApiPort !== undefined) {
        updateConfig("codexApiPort", parseInt(patch.codexApiPort, 10) || 4000);
      }
      if (patch.codexApiScheme !== undefined) updateConfig("codexApiScheme", patch.codexApiScheme);
      if (patch.codexArgs !== undefined) {
        updateConfig(
          "codexArgs",
          patch.codexArgs
            .split(",")
            .map((s) => s.trim())
            .filter(Boolean),
        );
      }
      if (patch.codexApiKeyMode !== undefined) {
        updateConfig(
          "codexApiKeyMode",
          patch.codexApiKeyMode as LauncherConfig["codexApiKeyMode"],
        );
      }
    },
    [updateConfig],
  );

  const openProviderSettings = useCallback((mode: ProviderMode) => {
    setSettingsProvider(mode);
    setActiveNav("settings");
  }, []);

  const handleNavChange = useCallback((nav: NavKey) => {
    if (nav === "settings") {
      setSettingsProvider("local");
    }
    setActiveNav(nav);
  }, []);

  const endpointPreview = useMemo(() => baseUrl || undefined, [baseUrl]);

  return (
    <div className="flex min-h-screen min-w-0 flex-col overflow-x-hidden bg-background md:flex-row">
      <Sidebar
        modelCount={models.length}
        refreshing={refreshing}
        onRefresh={refreshModels}
        activeNav={activeNav}
        onNavChange={handleNavChange}
      />

      <main className="themed-scrollbar min-h-0 min-w-0 flex-1 overflow-y-auto pb-[calc(4.5rem+env(safe-area-inset-bottom))] md:pb-0">
        <div className="mx-auto w-full max-w-[1100px] p-4 sm:p-6">
          {activeNav === "launcher" ? (
            <div className="space-y-5">
              <header className="mb-1" data-testid="page-launcher">
                <h1 className="text-[22px] font-semibold tracking-tight text-foreground">{APP_NAME}</h1>
                <p className="text-[13px] text-muted-foreground">
                  Configure Codex, pick a model provider, and run Codex from one place.
                </p>
              </header>

              {failoverStatus.activeConnectionAlert &&
              !failoverStatus.activeConnectionAlert.dismissed ? (
                <ConnectionBanner
                  alert={failoverStatus.activeConnectionAlert}
                  busy={refreshing}
                  onRefreshEndpoint={() => void refreshModels()}
                  onDismiss={() => void dismissConnectionAlert()}
                />
              ) : null}

              {failoverStatus.activeAlert && !failoverStatus.activeAlert.dismissed ? (
                <FailoverBanner
                  alert={failoverStatus.activeAlert}
                  checkpoint={failoverStatus.lastCheckpoint}
                  busy={operation === "failover_switching"}
                  onFailover={() => void failoverToLocal()}
                  onDismiss={() => void dismissFailoverAlert()}
                  onCopyResume={() => void copyResumePrompt()}
                />
              ) : null}

              <ProviderModeCard
                profile={codexProfile}
                activeMode={providerMode}
                canActivateLocal={canActivateLocal}
                localBlockedReason={localBlockedReason}
                switching={switchingProvider}
                switchingTo={switchingTo}
                onSelectMode={(mode) => void switchProvider(mode)}
                onOpenProviderSettings={openProviderSettings}
                endpointPreview={endpointPreview}
                modelPreview={selectedModel || undefined}
                models={modelNames}
                selectedModel={selectedModel}
                onSelectModel={(v) => void selectModel(v)}
                onRefreshModels={refreshModels}
                refreshing={refreshing}
                selectingModel={operation === "selecting_model"}
                serverState={serverState}
                operation={operation}
                onToggleLaunch={serverState === "running" ? stop : launch}
                canStart={canStart}
                startBlockedReason={startBlockedReason}
                statusStripText={statusMessage}
                statusVariant={statusVariant}
              />
            </div>
          ) : (
            <div className="mt-2">
              {activeNav === "models" && (
                <ModelsPanel modelCount={models.length} onRefresh={refreshModels} />
              )}
              {activeNav === "settings" && (
                <ProviderSettingsPanel
                  provider={settingsProvider}
                  onProviderChange={setSettingsProvider}
                  autoStart={autoStart}
                  onAutoStartChange={(v) => void setAutoStart(v)}
                  workingDir={workingDir}
                  onWorkingDirChange={(v) => updateConfig("workingDirectory", v)}
                  ip={ip}
                  port={port}
                  scheme={scheme}
                  onEndpointChange={handleEndpointChange}
                  baseUrl={baseUrl || "—"}
                  apiKey={apiKey}
                  onApiKeyChange={(v) => updateConfig("apiKey", v)}
                  codexConfig={codexForm}
                  onCodexConfigChange={handleCodexFormChange}
                  onRefreshModels={refreshModels}
                  refreshing={refreshing}
                  modelCount={models.length}
                />
              )}
              {activeNav === "logs" && <LogsPanel />}
              {activeNav === "about" && <AboutPanel />}
            </div>
          )}
        </div>
      </main>

      {detailsModel && (
        <ModelDetailsModal model={detailsModel} onClose={() => setDetailsModel(null)} />
      )}
    </div>
  );
}