import { RefreshCw, Settings } from "lucide-react";
import { useLauncher } from "../../context/LauncherContext";

export function SettingsPanel() {
  const { config, statusMessage, models, running, refreshModels } = useLauncher();

  const ip = config.ollamaIp ?? "";
  const port = String(config.ollamaPort ?? 11434);
  const scheme = config.ollamaScheme ?? "http";
  const model = config.codexModel ?? "";
  const apiKey = config.apiKey ?? "";
  const workingDir = config.workingDirectory ?? "";

  const rows = [
    { label: "Endpoint", value: `${scheme}://${ip}:${port}` },
    { label: "Model", value: model || "Not selected" },
    { label: "API Key", value: apiKey ? "Configured (redacted)" : "(not set)" },
    { label: "Working Dir", value: workingDir || "(default)" },
  ].filter((r) => r.value);

  return (
    <div className="card-surface space-y-5 p-6">
      <div>
        <h3 className="mb-3 text-[14px] font-semibold text-foreground">Current Configuration</h3>
        <div className="space-y-2 text-[12px]">
          {rows.map((row) => (
            <div key={row.label} className="flex items-center gap-2">
              <Settings className="h-3.5 w-3.5 shrink-0 text-primary/60" />
              <span className="min-w-[70px] font-medium text-muted-foreground">{row.label}:</span>
              <span className="break-all font-mono text-[11px] text-foreground">{row.value}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="border-t border-border pt-4">
        <h3 className="mb-3 text-[14px] font-semibold text-foreground">Server Status</h3>
        <div className="space-y-2 text-[12px]">
          <div className="flex items-center gap-2">
            <span
              className={`h-2 w-2 rounded-full ${running ? "bg-success" : "bg-destructive/50"}`}
            />
            <span className="text-muted-foreground">{running ? "Running" : "Stopped"}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-muted-foreground">Models:</span>
            <span className="text-foreground">{models.length} detected</span>
          </div>
        </div>
      </div>

      {statusMessage && statusMessage !== "Codex API is stopped" && (
        <p className="text-center text-[12px] text-muted-foreground">{statusMessage}</p>
      )}

      <p className="text-center text-[12px] text-muted-foreground">
        Configuration is saved automatically when you change settings on the Launcher page.
      </p>

      <button
        onClick={() => void refreshModels()}
        className="flex w-full items-center justify-center gap-2 rounded-lg border border-input bg-background px-3 py-2 text-xs text-foreground transition-colors hover:bg-muted/70"
      >
        <RefreshCw className="h-3 w-3" />
        Refresh Models
      </button>
    </div>
  );
}
