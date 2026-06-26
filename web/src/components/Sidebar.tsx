import type { LaunchStatus, ModelInfo } from "../types";
import StatusPill from "./StatusPill";

interface SidebarProps {
    launchStatus: LaunchStatus;
    models: ModelInfo[];
    statusMessage: string;
    activeNav: string;
    onNavClick: (item: string) => void;
    onRefreshModels: () => void;
    onToggleDarkMode: () => void;
    darkMode: boolean;
}

export default function Sidebar({
    launchStatus, models, statusMessage,
    activeNav, onNavClick, onRefreshModels,
    onToggleDarkMode, darkMode,
}: SidebarProps) {
    const status = launchStatus === "running" ? "running"
        : launchStatus === "launching" || launchStatus === "stopping" ? "launching" : "stopped";

    const navItems = [
        { id: "launcher", icon: "\u{1F680}", label: "Launcher" },
        { id: "models", icon: "\u{1F9EA}", label: "Models" },
        { id: "settings", icon: "\u{1F527}", label: "Settings" },
        { id: "logs", icon: "\u{1F4DD}", label: "Logs" },
        { id: "about", icon: "\u{2139}", label: "About" },
    ];

    return (
        <aside className="sidebar">
            <div className="sidebar-brand">Codex Launcher</div>
            <nav className="sidebar-nav">
                {navItems.map((item) => (
                    <button
                        key={item.id}
                        className={`nav-item ${activeNav === item.id ? "active" : ""}`}
                        onClick={() => onNavClick(item.id)}
                    >
                        <span className="nav-icon">{item.icon}</span>
                        {item.label}
                    </button>
                ))}
            </nav>
            <div className="sidebar-footer">
                <div className="status-card">
                    <div className="status-label">Codex Status</div>
                    <div className="status-row">
                        <span className={`status-dot ${status}`} />
                        <span className="status-text">{
                            launchStatus === "running" ? "Running"
                                 : launchStatus === "stopping" ? "Stopping..."
                                 : launchStatus === "launching" ? "Starting..."
                                : "Stopped"
                        }</span>
                    </div>
                    <div className="status-message">{statusMessage || "Ready"}</div>
                </div>
                <button className="refresh-btn-sidebar" onClick={onRefreshModels}>
                    Refresh Models
                    {models.length > 0 && (
                        <span className="badge">{models.length}</span>
                    )}
                </button>
                <label className="toggle-switch" style={{ marginTop: 8 }}>
                    <input
                        type="checkbox"
                        checked={darkMode}
                        onChange={onToggleDarkMode}
                    />
                    <span className="toggle-track" />
                </label>
            </div>
        </aside>
    );
}
