import type { LaunchStatus, ModelInfo } from "../types";
import StatusPill from "./StatusPill";

interface LaunchPanelProps {
    launchStatus: LaunchStatus;
    models: ModelInfo[];
    healthError: string | null;
    modelsError: string | null;
    onLaunch: () => void;
    onStop: () => void;
    onRefreshModels: () => void;
}

export default function LaunchPanel({ launchStatus, models, healthError, modelsError, onLaunch, onStop, onRefreshModels }: LaunchPanelProps) {
    const isRunning = launchStatus === "running" || launchStatus === "launching" || launchStatus === "stopping";

    return (
            <div className="card">
                <div className="card-header">
                    <span className="card-icon">{'\u{1F680}'}</span>
                    <div>
                        <div className="card-title">Launch Codex</div>
                        <div className="card-subtitle">Start or stop the Codex-compatible local API server</div>
                    </div>
                </div>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
                    <span style={{ fontSize: 13, color: "var(--text-secondary)" }}>Status</span>
                    <StatusPill status={isRunning ? (launchStatus === "launching" ? "launching" : launchStatus === "stopping" ? "launching" : "running") : "stopped"} />
                </div>
                <button
                className={'launch-btn ' + (isRunning ? 'stop' : 'start')}
                onClick={isRunning ? onStop : onLaunch}
                disabled={launchStatus === 'launching' || launchStatus === 'stopping'}
                 >
                    {'' + (launchStatus === 'launching' ? '\u{23F3}' : isRunning ? '\u{23F8}' : '\u{25B6}')}
                    {' '}
                    {launchStatus === 'launching' ? 'Starting...' : launchStatus === 'stopping' ? 'Stopping...' : isRunning ? 'Stop Codex Server' : 'Start Codex Server'}
                </button>
                <div style={{ marginTop: 12, fontSize: 12, color: "var(--text-secondary)", textAlign: "center" }}>
                    {launchStatus === 'idle' && '\u{1F50C} Ollama / Codex API is stopped'}
                    {launchStatus === 'launching' && '\u{1F504} Launching Codex...'}
                    {launchStatus === 'stopping' && '\u{1F5D1} Stopping Codex...'}
                    {launchStatus === 'running' && '\u{2705} Codex is running'}
                    {launchStatus === 'stopped' && '\u{1F7E2} Codex has been stopped'}
                    {launchStatus === 'error' && '\u{274C} Error launching Codex'}
                </div>
                {healthError && (
                    <div style={{ marginTop: 8, padding: '8px 12px', background: 'var(--warning-bg)', color: 'var(--warning-text)', borderRadius: 'var(--radius)', fontSize: 12 }}>
                        {'\u{26A0} '} Health check failed: {healthError}
                    </div>
                )}
                <div style={{ marginTop: 16 }}>
                    <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
                        <select
                        className="form-select"
                        value={models.length > 0 ? models[0].name : ""}
                        onChange={(e) => console.log("Selected:", e.target.value)}
                        disabled={models.length === 0}
                         >
                             {models.length === 0 && <option value="">No models detected</option>}
                             {models.map((m) => <option key={m.digest} value={m.name}>{m.name}</option>)}
                        </select>
                        <button className="btn btn-secondary btn-sm" onClick={onRefreshModels}>
                        Refresh
                        </button>
                    </div>
                    <div className="helper-text">
                        {modelsError ? modelsError :
                        models.length === 0
                            ? 'No models detected - start Ollama or check endpoint settings.'
                            : (models.length + ' model' + (models.length === 1 ? '' : 's') + ' available')}
                    </div>
                </div>
            </div>
        );
}