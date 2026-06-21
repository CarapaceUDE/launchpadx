import { useState, useRef, useEffect } from "react";
import type { LauncherConfig, CodexProcessInfo, ModelInfo } from "../types";

interface ConfigFormProps {
    config: LauncherConfig;
    onUpdate: <K extends keyof LauncherConfig>(key: K, value: LauncherConfig[K]) => void;
    onWriteCodexConfig: () => void;
    onRevertCodexConfig: () => void;
    onRefreshModels: () => void;
    models: ModelInfo[];
    codexInfo?: CodexProcessInfo;
    onKillCodex: () => void;
    onToggleAutoStart?: () => void;
}

export default function ConfigForm({
    config,
    onUpdate,
    onWriteCodexConfig,
    onRevertCodexConfig,
    onRefreshModels,
    models,
    codexInfo,
    onKillCodex,
    onToggleAutoStart,
}: ConfigFormProps) {
    const [openSections, setOpenSections] = useState<Record<string, boolean>>({
          "Connection": true,
          "Preferences": true,
          "Codex Config": false,
      });

    const [ipField, setIpField] = useState(config.ollamaIp || "");
    const [baseUrlField, setBaseUrlField] = useState(config.openaiBaseUrl || "");
    const hasSynced = useRef(false);

      // Sync local form fields when config changes externally (IP/URL only)
    useEffect(() => {
        if (!hasSynced.current) {
            hasSynced.current = true;
            return;
          }
        const newIp = config.ollamaIp || "";
        setIpField((prev) => (prev !== newIp ? newIp : prev));
        const newBaseUrl = config.openaiBaseUrl || "";
        setBaseUrlField((prev) => (prev !== newBaseUrl ? newBaseUrl : prev));
      }, [config.ollamaIp, config.openaiBaseUrl]);

    const toggleSection = (section: string) => {
        setOpenSections((prev) => ({ ...prev, [section]: !prev[section] }));
      };

    const buildUrl = (ip: string, port: number | undefined, scheme: string) => {
        if (!ip) return "";
        const p = port || 11434;
        return `${scheme}://${ip}:${p}/v1`;
      };

    const handleIpChange = (val: string) => {
        setIpField(val);
        const url = buildUrl(val, config.ollamaPort, config.ollamaScheme || "http");
        setBaseUrlField(url);
        onUpdate("ollamaIp", val);
        if (url) onUpdate("openaiBaseUrl", url);
      };

    const handleBaseUrlChange = (val: string) => {
        setBaseUrlField(val);
        if (val) {
            try {
                const url = new URL(val);
                const scheme = url.protocol.replace(":", "");
                let port = url.port ? parseInt(url.port) : (scheme === "https" ? 443 : 80);
                setIpField(url.hostname);
                onUpdate("ollamaIp", url.hostname);
                onUpdate("ollamaPort", port);
                onUpdate("ollamaScheme", scheme);
                onUpdate("openaiBaseUrl", val);
              } catch {
                onUpdate("openaiBaseUrl", val);
              }
          } else {
            setIpField("");
            onUpdate("openaiBaseUrl", undefined);
          }
      };

    const handlePortChange = (val: string) => {
        const parsed = parseInt(val);
        const port = (parsed >= 1 && parsed <= 65535) ? parsed : 0;
        onUpdate("ollamaPort", port);
        const url = buildUrl(ipField, port, config.ollamaScheme || "http");
        setBaseUrlField(url);
        onUpdate("openaiBaseUrl", url);
      };

    const handleSchemeChange = (scheme: string) => {
        onUpdate("ollamaScheme", scheme);
        const url = buildUrl(ipField, config.ollamaPort, scheme);
        setBaseUrlField(url);
        onUpdate("openaiBaseUrl", url);
      };

    const Section = ({ title, children }: { title: string; children: React.ReactNode }) => {
        const isOpen = openSections[title] ?? false;
        return (
              <div className="config-section">
                  <div className="section-header" onClick={() => toggleSection(title)}>
                      <span>{title}</span>
                      <span className={`chevron ${isOpen ? "expanded" : ""}`}>
                          {"\u25B6"}
                      </span>
                  </div>
                  {isOpen && <div className="section-body">{children}</div>}
              </div>
          );
      };

    const codexIsRunning = codexInfo?.running || false;
    const codexPid = codexInfo?.pid ?? null;
    const hasBackup = codexInfo !== null;

    return (
          <>
              <Section title="Connection">
                  <div className="field-group">
                      <label className="field-label">API Address</label>
                      <div className="field-row">
                          <input
                            className="field-input"
                            type="text"
                            value={ipField}
                            onChange={(e) => handleIpChange(e.target.value)}
                            placeholder="127.0.0.1"
                          />
                          <input
                            className="field-input small"
                            type="number"
                            value={config.ollamaPort || 11434}
                            onChange={(e) => handlePortChange(e.target.value)}
                          />
                          <select
                            className="field-select"
                            value={config.ollamaScheme || "http"}
                            onChange={(e) => handleSchemeChange(e.target.value)}
                          >
                              <option value="http">http</option>
                              <option value="https">https</option>
                          </select>
                      </div>
                  </div>
                  <div className="field-group">
                      <label className="field-label">Codex API Address</label>
                      <div className="field-row">
                          <input
                            className="field-input"
                            type="text"
                            value={config.codexApiPort || 4000}
                            onChange={(e) => onUpdate("codexApiPort", parseInt(e.target.value) || 0)}
                          />
                          <select
                            className="field-select"
                            value={config.codexApiScheme || "http"}
                            onChange={(e) => onUpdate("codexApiScheme", e.target.value)}
                          >
                              <option value="http">http</option>
                              <option value="https">https</option>
                          </select>
                      </div>
                  </div>
                  <div className="field-group">
                      <label className="field-label">CLI Args (comma-separated)</label>
                      <input
                        className="field-input"
                        type="text"
                        value={(config.codexArgs || []).join(", ")}
                        onChange={(e) => onUpdate("codexArgs", e.target.value.split(",").map((s) => s.trim()).filter(Boolean))}
                        placeholder="arg1, arg2"
                      />
                  </div>

                  <div style={{ marginTop: 16, display: "flex", flexDirection: "column", gap: 8 }}>
                      <button
                        className="btn"
                        onClick={onWriteCodexConfig}
                        disabled={codexIsRunning}
                        title={codexIsRunning ? "Stop Codex first, then write config" : ""}
                      >
                          {codexIsRunning ? "\u26A0 Write Config (requires restart)" : "\u{1F4DD} Write Codex Config"}
                      </button>

                      {hasBackup && (
                          <div style={{ display: "flex", gap: 8, marginTop: 4 }}>
                              <button
                                className="btn secondary"
                                onClick={onRevertCodexConfig}
                                title="Revert to the first backup of the Codex config"
                                disabled={codexIsRunning}
                              >
                                  {codexIsRunning ? "\u26A0 Revert (requires restart)" : "\u{21BA} Revert to First Backup"}
                              </button>

                              {codexPid && (
                                  <button
                                    className="btn secondary"
                                    onClick={onKillCodex}
                                    style={{ background: "#ff6b6b", color: "white" }}
                                  >
                                      {codexIsRunning ? `\u{1F5D1} Kill Process (${codexPid})` : "Process stopped"}
                                  </button>
                              )}
                              {onToggleAutoStart && (
                                  <button
                                    className="btn secondary"
                                    onClick={onToggleAutoStart}
                                  >
                                    Toggle Auto-Start
                                  </button>
                              )}
                          </div>
                      )}

                      {codexIsRunning && (
                          <div style={{
                            fontSize: 11,
                            color: "var(--warning-text)",
                            marginTop: 4,
                          }}>
                              <span>{"\u26A0"} Codex is running. Changes take effect after restart.</span>
                          </div>
                      )}
                  </div>
              </Section>

              <Section title="Model">
                  <div className="model-field">
                      <select
                        className="field-select"
                        value={config.codexModel || ""}
                        onChange={(e) => onUpdate("codexModel", e.target.value)}
                      >
                          <option value="">-- Select or refresh models --</option>
                          {models.map((m) => (
                              <option key={m.name} value={m.name}>{m.name}</option>
                          ))}
                      </select>
                      <button className="btn secondary small" onClick={onRefreshModels}>
                        Refresh
                      </button>
                  </div>
              </Section>
          </>
      );
}