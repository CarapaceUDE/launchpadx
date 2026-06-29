import { useState } from "react";
import { ChevronDown, Cloud, Eye, EyeOff, Server, Settings } from "lucide-react";
import { Card, FormField, TextInput, ToggleRow } from "./primitives";
import { ThemeToggle } from "./ThemeToggle";
import type { CodexConfigForm } from "../../context/LauncherContext";
import {
  providerModeDescription,
  providerModeLabel,
  type ProviderMode,
} from "../../lib/codexProfile";
import type { CodexRateLimitsStatus } from "../../types";
import { CodexRateLimitsPanel } from "./CodexRateLimitsPanel";
import { LocalModelsCatalog } from "./LocalModelsCatalog";

const PROVIDERS: ProviderMode[] = ["codex", "local"];

export function ProviderSettingsPanel({
  provider,
  onProviderChange,
  autoStart,
  onAutoStartChange,
  workingDir,
  onWorkingDirChange,
  ip,
  port,
  scheme,
  onEndpointChange,
  baseUrl,
  apiKey,
  onApiKeyChange,
  codexConfig,
  onCodexConfigChange,
  onRefreshModels,
  refreshing,
  modelCount,
  rateLimitsStatus,
  rateLimitsLoading,
  onRefreshRateLimits,
  autoSwitchOnRateLimit,
  onAutoSwitchOnRateLimitChange,
}: {
  provider: ProviderMode;
  onProviderChange: (mode: ProviderMode) => void;
  autoStart: boolean;
  onAutoStartChange: (v: boolean) => void;
  workingDir: string;
  onWorkingDirChange: (v: string) => void;
  ip: string;
  port: string;
  scheme: "http" | "https";
  onEndpointChange: (patch: { ip?: string; port?: string; scheme?: "http" | "https" }) => void;
  baseUrl: string;
  apiKey: string;
  onApiKeyChange: (v: string) => void;
  codexConfig: CodexConfigForm;
  onCodexConfigChange: (patch: Partial<CodexConfigForm>) => void;
  onRefreshModels: () => void;
  refreshing: boolean;
  modelCount: number;
  rateLimitsStatus: CodexRateLimitsStatus | null;
  rateLimitsLoading?: boolean;
  onRefreshRateLimits: () => void;
  autoSwitchOnRateLimit: boolean;
  onAutoSwitchOnRateLimitChange: (v: boolean) => void;
}) {
  const [showKey, setShowKey] = useState(false);
  const [advancedOpen, setAdvancedOpen] = useState(false);

  return (
    <div className="space-y-5" data-testid="provider-settings-panel">
      <header>
        <h1 className="text-[22px] font-semibold tracking-tight text-foreground">Settings</h1>
        <p className="mt-1 text-[13px] text-muted-foreground">
          Provider-specific options. Changes save automatically.
        </p>
      </header>

      <Card className="!p-5">
        <FormField label="Provider" hint="Switch which provider's settings you are editing">
          <div className="relative max-w-sm">
            <select
              data-testid="settings-provider-select"
              value={provider}
              onChange={(e) => onProviderChange(e.target.value as ProviderMode)}
              className="themed-native-select h-[38px] w-full px-3 pr-9 text-sm focus:ring-4"
            >
              {PROVIDERS.map((p) => (
                <option key={p} value={p}>
                  {providerModeLabel(p)}
                </option>
              ))}
            </select>
            <ChevronDown className="pointer-events-none absolute right-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          </div>
        </FormField>

        <p className="mt-3 flex items-start gap-2 text-[12px] text-muted-foreground">
          {provider === "codex" ? (
            <Cloud className="mt-0.5 h-3.5 w-3.5 shrink-0" />
          ) : (
            <Server className="mt-0.5 h-3.5 w-3.5 shrink-0" />
          )}
          {providerModeDescription(provider)}
        </p>
      </Card>

      {provider === "codex" ? (
        <Card icon={<Cloud className="h-4 w-4" />} title="Codex Account settings">
          <div className="space-y-4">
            <ToggleRow
              label="Auto-start Codex"
              description="Start Codex when this app opens"
              checked={autoStart}
              onChange={onAutoStartChange}
              testId="auto-start-toggle"
            />
            <ToggleRow
              label="Auto-switch to local on rate limit"
              description="When Codex app-server reports rateLimitReachedType, stop Codex, apply your local API settings, and restart automatically"
              checked={autoSwitchOnRateLimit}
              onChange={onAutoSwitchOnRateLimitChange}
              testId="auto-switch-rate-limit-toggle"
            />
            <FormField
              label="Working Directory"
              hint="Absolute path to the project directory Codex should use"
            >
              <TextInput
                data-testid="working-directory"
                value={workingDir}
                onChange={(e) => onWorkingDirChange(e.target.value)}
                placeholder="C:\projects\my-app"
              />
            </FormField>

            <div className="border-t border-border pt-4">
              <CodexRateLimitsPanel
                status={rateLimitsStatus}
                loading={rateLimitsLoading}
                onRefresh={onRefreshRateLimits}
              />
            </div>
          </div>
        </Card>
      ) : (
        <Card icon={<Server className="h-4 w-4" />} title="Local API settings">
          <div className="space-y-4">
            <div className="grid grid-cols-1 gap-x-4 gap-y-[14px] sm:grid-cols-2">
              <FormField label="IP Address">
                <TextInput
                  data-testid="endpoint-ip"
                  value={ip}
                  onChange={(e) => onEndpointChange({ ip: e.target.value })}
                  placeholder="127.0.0.1"
                />
              </FormField>
              <FormField label="Port">
                <TextInput
                  data-testid="endpoint-port"
                  value={port}
                  onChange={(e) => onEndpointChange({ port: e.target.value })}
                  placeholder="11434"
                />
              </FormField>
              <FormField label="Scheme">
                <div className="relative">
                  <select
                    data-testid="endpoint-scheme"
                    value={scheme}
                    onChange={(e) =>
                      onEndpointChange({ scheme: e.target.value as "http" | "https" })
                    }
                    className="themed-native-select h-[38px] w-full px-3 pr-9 text-sm focus:ring-4"
                  >
                    <option value="http">http</option>
                    <option value="https">https</option>
                  </select>
                  <ChevronDown className="pointer-events-none absolute right-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                </div>
              </FormField>
              <FormField label="Base URL" hint="Generated from IP, port, and scheme">
                <TextInput
                  data-testid="endpoint-base-url"
                  value={baseUrl}
                  readOnly
                  className="cursor-not-allowed bg-muted/50 font-mono text-[13px] text-muted-foreground"
                />
              </FormField>
            </div>

            <FormField label="API Key (optional)">
              <div className="relative">
                <TextInput
                  data-testid="api-key"
                  type={showKey ? "text" : "password"}
                  value={apiKey}
                  onChange={(e) => onApiKeyChange(e.target.value)}
                  placeholder="sk-..."
                  className="pr-10"
                />
                <button
                  type="button"
                  onClick={() => setShowKey((s) => !s)}
                  className="absolute right-2 top-1/2 grid h-7 w-7 -translate-y-1/2 place-items-center rounded text-muted-foreground transition-colors hover:bg-muted/70 hover:text-foreground"
                >
                  {showKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
            </FormField>

            <LocalModelsCatalog modelCount={modelCount} onRefresh={onRefreshModels} />
          </div>
        </Card>
      )}

      <Card icon={<Settings className="h-4 w-4" />} title="Advanced Codex configuration">
        <button
          type="button"
          onClick={() => setAdvancedOpen((o) => !o)}
          className="flex w-full items-center justify-between gap-2 py-1 text-[13px] font-medium text-muted-foreground transition-colors hover:text-foreground"
        >
          <span>Show advanced fields</span>
          <ChevronDown
            className={`h-4 w-4 transition-transform ${advancedOpen ? "rotate-180" : ""}`}
          />
        </button>

        {advancedOpen ? (
          <div className="mt-4 space-y-3 border-t border-border pt-4">
            <div className="grid grid-cols-1 gap-x-3 gap-y-3 sm:grid-cols-2">
              <FormField label="Provider ID">
                <TextInput
                  value={codexConfig.codexProviderId}
                  onChange={(e) => onCodexConfigChange({ codexProviderId: e.target.value })}
                />
              </FormField>
              <FormField label="Provider Name">
                <TextInput
                  value={codexConfig.codexProviderName}
                  onChange={(e) => onCodexConfigChange({ codexProviderName: e.target.value })}
                />
              </FormField>
              <FormField label="Codex Config Path">
                <TextInput
                  value={codexConfig.codexConfigPath}
                  onChange={(e) => onCodexConfigChange({ codexConfigPath: e.target.value })}
                />
              </FormField>
              <FormField
                label="Codex command override"
                hint="Leave blank for auto-discovery from running Codex and common install paths"
              >
                <TextInput
                  value={codexConfig.codexCommand}
                  onChange={(e) => onCodexConfigChange({ codexCommand: e.target.value })}
                  placeholder="Auto-detect"
                />
              </FormField>
            </div>
            <FormField label="Codex Args (comma-separated)">
              <TextInput
                value={codexConfig.codexArgs}
                onChange={(e) => onCodexConfigChange({ codexArgs: e.target.value })}
              />
            </FormField>
          </div>
        ) : null}
      </Card>

      <div className="lg:hidden">
        <ThemeToggle variant="card" />
      </div>
    </div>
  );
}