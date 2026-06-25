import { useState } from "react";
import { SlidersHorizontal, Eye, EyeOff, Settings, ChevronDown } from "lucide-react";
import { Card, FormField, TextInput, ToggleRow } from "./primitives";
import type { CodexConfigForm } from "../../context/LauncherContext";

export function GeneralSettingsCard({
  autoStart,
  onAutoStartChange,
  workingDir,
  onWorkingDirChange,
  apiKey,
  onApiKeyChange,
  codexConfig,
  onCodexConfigChange,
}: {
  autoStart: boolean;
  onAutoStartChange: (v: boolean) => void;
  workingDir: string;
  onWorkingDirChange: (v: string) => void;
  apiKey: string;
  onApiKeyChange: (v: string) => void;
  codexConfig: CodexConfigForm;
  onCodexConfigChange: (patch: Partial<CodexConfigForm>) => void;
}) {
  const [showKey, setShowKey] = useState(false);
  const [advancedOpen, setAdvancedOpen] = useState(false);

  const apiKeyWarning =
    apiKey === "" || apiKey === "replace-with-your-api-key"
      ? "API key is missing or uses a placeholder value"
      : undefined;

  return (
    <Card icon={<SlidersHorizontal className="h-4 w-4" />} title="General Settings">
      <div className="space-y-4">
        <ToggleRow
          label="Auto-start Codex"
          description="Launch the server when this app opens"
          checked={autoStart}
          onChange={onAutoStartChange}
          testId="auto-start-toggle"
        />

        <FormField label="Working Directory" hint="Enter an absolute path to the project directory">
          <TextInput
            data-testid="working-directory"
            value={workingDir}
            onChange={(e) => onWorkingDirChange(e.target.value)}
            placeholder="C:\projects\my-app"
          />
        </FormField>

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
          {apiKeyWarning && (
            <p className="mt-1.5 text-[12px] text-warning-fg">{apiKeyWarning}</p>
          )}
          <p className="mt-1.5 text-[12px] text-muted-foreground">
            Stored locally in <code>config.json</code>. The launcher redacts this value in diagnostics and logs.
          </p>
        </FormField>

        <div className="border-t border-border pt-3">
          <button
            type="button"
            onClick={() => setAdvancedOpen((o) => !o)}
            className="flex w-full items-center justify-between gap-2 py-2 text-[13px] font-medium text-muted-foreground transition-colors hover:text-foreground"
          >
            <span className="flex items-center gap-2">
              <Settings className="h-4 w-4 text-muted-foreground/60" />
              Advanced Codex Configuration
            </span>
            <ChevronDown
              className={`h-4 w-4 text-muted-foreground/60 transition-transform ${advancedOpen ? "rotate-180" : ""}`}
            />
          </button>

          {advancedOpen && (
            <div className="mt-3 space-y-3 border-t border-border pt-3">
              <div className="grid grid-cols-2 gap-x-3 gap-y-3">
                <FormField label="Provider ID">
                  <TextInput
                    value={codexConfig.codexProviderId}
                    onChange={(e) => onCodexConfigChange({ codexProviderId: e.target.value })}
                    placeholder="codex-local-launcher"
                  />
                </FormField>
                <FormField label="Provider Name">
                  <TextInput
                    value={codexConfig.codexProviderName}
                    onChange={(e) => onCodexConfigChange({ codexProviderName: e.target.value })}
                    placeholder="Codex Local Launcher"
                  />
                </FormField>
                <FormField label="Codex Config Path">
                  <TextInput
                    value={codexConfig.codexConfigPath}
                    onChange={(e) => onCodexConfigChange({ codexConfigPath: e.target.value })}
                    placeholder="/path/to/config"
                  />
                </FormField>
                <FormField label="Codex Command">
                  <TextInput
                    value={codexConfig.codexCommand}
                    onChange={(e) => onCodexConfigChange({ codexCommand: e.target.value })}
                    placeholder="codex-app"
                  />
                </FormField>
                <FormField label="Codex API Port">
                  <TextInput
                    type="number"
                    value={codexConfig.codexApiPort}
                    onChange={(e) => onCodexConfigChange({ codexApiPort: e.target.value })}
                    placeholder="4000"
                  />
                </FormField>
                <FormField label="Codex API Scheme">
                  <select
                    value={codexConfig.codexApiScheme}
                    onChange={(e) => onCodexConfigChange({ codexApiScheme: e.target.value })}
                    className="h-[38px] w-full appearance-none rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-primary focus:outline-none focus:ring-4 focus:ring-primary/15"
                  >
                    <option value="http">http</option>
                    <option value="https">https</option>
                  </select>
                </FormField>
              </div>
              <FormField label="Codex Args (comma-separated)">
                <TextInput
                  value={codexConfig.codexArgs}
                  onChange={(e) => onCodexConfigChange({ codexArgs: e.target.value })}
                  placeholder="--verbose,--debug"
                />
              </FormField>
              <FormField label="API Key Mode">
                <select
                  value={codexConfig.codexApiKeyMode}
                  onChange={(e) => onCodexConfigChange({ codexApiKeyMode: e.target.value })}
                  className="h-[38px] w-full appearance-none rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-primary focus:outline-none focus:ring-4 focus:ring-primary/15"
                >
                  <option value="envKey">Environment Variable</option>
                  <option value="experimentalBearerToken">Experimental Bearer Token</option>
                  <option value="none">None</option>
                </select>
              </FormField>
            </div>
          )}
        </div>

        <div className="space-y-1.5 rounded-lg border border-dashed border-border bg-muted/30 px-3.5 py-2.5">
          <p className="text-[12px] text-muted-foreground">
            Configuration is saved automatically as you type.
          </p>
          <p className="text-[12px] text-muted-foreground">
            Models are discovered automatically when the endpoint is reachable.
          </p>
        </div>
      </div>
    </Card>
  );
}
