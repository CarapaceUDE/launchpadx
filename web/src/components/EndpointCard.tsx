import { useState } from "react";
import type { LauncherConfig } from "../types";

interface EndpointCardProps {
    config: LauncherConfig;
    onUpdate: <K extends keyof LauncherConfig>(key: K, value: LauncherConfig[K]) => void;
    baseImageUrl?: string;
}

export default function EndpointCard({ config, onUpdate, baseImageUrl }: EndpointCardProps) {
    const [ipInput, setIpInput] = useState(config.ollamaIp || "");
    const [schemeInput, setSchemeInput] = useState(config.ollamaScheme || "http");
    const [portInput, setPortInput] = useState((config.ollamaPort || 11434).toString());

    const handleIpChange = (val: string) => {
        setIpInput(val);
        const port = parseInt(portInput) || 11434;
        const scheme = schemeInput;
        const url = val ? `${scheme}://${val}:${port}/v1` : "";
        onUpdate("ollamaIp", val);
        onUpdate("openaiBaseUrl", url || undefined);
    };

    const handlePortChange = (val: string) => {
        setPortInput(val);
        const port = parseInt(val) || 0;
        const scheme = schemeInput;
        const url = ipInput ? `${scheme}://${ipInput}:${port}/v1` : "";
        onUpdate("ollamaPort", port);
        onUpdate("openaiBaseUrl", url || undefined);
    };

    const handleSchemeChange = (scheme: string) => {
        setSchemeInput(scheme);
        const port = parseInt(portInput) || 11434;
        const url = ipInput ? `${scheme}://${ipInput}:${port}/v1` : "";
        onUpdate("ollamaScheme", scheme);
        onUpdate("openaiBaseUrl", url || undefined);
    };

    const derivedUrl = baseImageUrl || ipInput
        ? `${schemeInput}://${ipInput}:${parseInt(portInput) || 11434}/v1`
        : "";

    return (
         <div className="card">
              <div className="card-header">
                  <span className="card-icon">{`\u{1F310}`}</span>
                  <div>
                      <div className="card-title">Endpoint Configuration</div>
                      <div className="card-subtitle">Connect to your Ollama-compatible API endpoint</div>
                  </div>
              </div>
              <div className="form-field">
                  <label className="form-label">API IP</label>
                  <input
                      className="form-input"
                      type="text"
                      value={ipInput}
                      onChange={(e) => handleIpChange(e.target.value)}
                      placeholder="127.0.0.1"
                  />
              </div>
              <div className="form-row">
                  <div className="form-field" style={{ flex: 1 }}>
                      <label className="form-label">Port</label>
                      <input
                          className="form-input small"
                          type="number"
                          value={portInput}
                          onChange={(e) => handlePortChange(e.target.value)}
                      />
                  </div>
                  <div className="form-field" style={{ flex: "none", width: 110 }}>
                      <label className="form-label">Scheme</label>
                      <select
                          className="form-select"
                          value={schemeInput}
                          onChange={(e) => handleSchemeChange(e.target.value)}
                      >
                          <option value="http">http</option>
                          <option value="https">https</option>
                      </select>
                  </div>
              </div>
              <div className="form-field">
                  <label className="form-label">Base URL</label>
                  <input
                      className="form-input readonly"
                      type="text"
                      value={derivedUrl}
                      readOnly
                  />
                  <span className="helper-text">Generated from IP, port, and scheme</span>
              </div>
          </div>
      );
}
