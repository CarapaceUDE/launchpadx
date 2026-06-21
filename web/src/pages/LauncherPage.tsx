import { useCallback, useEffect, useState } from "react";
import type { LauncherConfig, ModelInfo } from "../types";
import {
  useLauncher,
  configToCodexForm,
  type NavKey,
  type CodexConfigForm,
} from "../context/LauncherContext";
import { Sidebar } from "../components/launcher/Sidebar";
import { LaunchPanel } from "../components/launcher/LaunchPanel";
import { CodexProfileBar } from "../components/launcher/CodexProfileBar";
import { EndpointCard } from "../components/launcher/EndpointCard";
import { GeneralSettingsCard } from "../components/launcher/GeneralSettingsCard";
import { ModelsPanel } from "../components/launcher/ModelsPanel";
import { SettingsPanel } from "../components/launcher/SettingsPanel";
import { LogsPanel } from "../components/launcher/LogsPanel";
import { AboutPanel } from "../components/launcher/AboutPanel";
import { ModelDetailsModal } from "../components/launcher/ModelDetailsModal";
import { buildOpenAiBaseUrl } from "../lib/endpoint";

export function LauncherPage() {
  const {
    running,
    statusMessage,
    models,
    config,
    refreshing,
    launch,
    stop,
    writeCodexConfig,
    revertCodexConfig,
    refreshModels,
    updateConfig,
    openDirectoryPicker,
  } = useLauncher();

  const [activeNav, setActiveNav] = useState<NavKey>("launcher");
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
  const selectedModel = config.codexModel ?? "";
  const modelNames = models.map((m) => m.name);

  const canStart = Boolean(ip && port && selectedModel);
  const canWrite = Boolean(config.ollamaIp && config.codexModel);
  const startBlockedReason = !selectedModel
    ? models.length === 0
      ? "Refresh models from your endpoint, then select one above to enable Start."
      : "Select a model above to enable Start."
    : undefined;

  const handleEndpointChange = useCallback(
    (patch: { ip?: string; port?: string; scheme?: "http" | "https" }) => {
      const nextIp = patch.ip ?? ip;
      const nextPort = patch.port !== undefined ? parseInt(patch.port, 10) || 11434 : parseInt(port, 10) || 11434;
      const nextScheme = patch.scheme ?? scheme;

      if (patch.ip !== undefined) updateConfig("ollamaIp", patch.ip);
      if (patch.port !== undefined) updateConfig("ollamaPort", nextPort);
      if (patch.scheme !== undefined) updateConfig("ollamaScheme", patch.scheme);

      const baseUrl = buildOpenAiBaseUrl(nextIp, nextPort, nextScheme);
      if (baseUrl) updateConfig("openaiBaseUrl", baseUrl);
    },
    [ip, port, scheme, updateConfig],
  );

  const handleBrowseDir = useCallback(async () => {
    const path = await openDirectoryPicker();
    if (path) updateConfig("workingDirectory", path);
  }, [openDirectoryPicker, updateConfig]);

  const handleViewDetails = useCallback(
    (name: string) => {
      const model = models.find((m) => m.name === name);
      if (model) setDetailsModel(model);
    },
    [models],
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

  return (
    <div className="flex min-h-screen bg-background">
      <Sidebar
        running={running}
        statusMessage={statusMessage}
        modelCount={models.length}
        onRefresh={refreshModels}
        activeNav={activeNav}
        onNavChange={setActiveNav}
      />

      <main className="flex-1 overflow-y-auto">
        <div className="mx-auto max-w-[1100px] p-6">
          {activeNav === "launcher" ? (
            <div className="space-y-5">
              <header className="mb-1">
                <h1 className="text-[22px] font-semibold tracking-tight text-foreground">Launcher</h1>
                <p className="text-[13px] text-muted-foreground">
                  Manage your local Codex server and model configuration.
                </p>
              </header>

              <div className="space-y-3">
                <LaunchPanel
                  running={running}
                  onToggle={running ? stop : launch}
                  canStart={canStart}
                  startBlockedReason={startBlockedReason}
                  statusStripText={statusMessage}
                  models={modelNames}
                  selectedModel={selectedModel}
                  onSelectModel={(v) => updateConfig("codexModel", v)}
                  onRefreshModels={refreshModels}
                  refreshing={refreshing}
                  onViewModelDetails={handleViewDetails}
                  modelStatusHint={statusMessage}
                />

                <CodexProfileBar
                  onWrite={writeCodexConfig}
                  onRevert={revertCodexConfig}
                  canWrite={canWrite}
                />
              </div>

              <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
                <EndpointCard
                  ip={ip}
                  port={port}
                  scheme={scheme}
                  onChange={handleEndpointChange}
                  onRefresh={refreshModels}
                  refreshing={refreshing}
                />
                <GeneralSettingsCard
                  autoStart={autoStart}
                  onAutoStartChange={(v) => updateConfig("autoStart", v)}
                  workingDir={workingDir}
                  onWorkingDirChange={(v) => updateConfig("workingDirectory", v)}
                  onBrowseDir={handleBrowseDir}
                  apiKey={apiKey}
                  onApiKeyChange={(v) => updateConfig("apiKey", v)}
                  codexConfig={codexForm}
                  onCodexConfigChange={handleCodexFormChange}
                />
              </div>

            </div>
          ) : (
            <div className="mt-2">
              {activeNav === "models" && (
                <ModelsPanel modelCount={models.length} onRefresh={refreshModels} />
              )}
              {activeNav === "settings" && <SettingsPanel />}
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