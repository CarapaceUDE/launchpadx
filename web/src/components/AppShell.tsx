import type { ReactNode } from "react";
import Sidebar from "./Sidebar";
import type { LaunchStatus, ModelInfo } from "../types";

interface AppShellProps {
    children: ReactNode;
    darkMode: boolean;
    onToggleDarkMode: () => void;
    launchStatus: LaunchStatus;
    models: ModelInfo[];
    statusMessage: string;
    activeNav: string;
    onNavClick: (item: string) => void;
}

export default function AppShell({
    children, darkMode, onToggleDarkMode,
    launchStatus, models, statusMessage,
    activeNav, onNavClick,
}: AppShellProps) {
    return (
         <div className="app-shell">
             <Sidebar
                launchStatus={launchStatus}
                models={models}
                statusMessage={statusMessage}
                activeNav={activeNav}
                onNavClick={onNavClick}
                onRefreshModels={onNavClick.bind(null, "models")}
                onToggleDarkMode={onToggleDarkMode}
                darkMode={darkMode}
             />
             <main className="main-content">
                 {children}
             </main>
         </div>
     );
}
