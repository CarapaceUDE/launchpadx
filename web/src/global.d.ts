import type { LauncherConfig, CodexProcessInfo } from './types';

interface LauncherResponse<T> {
    ok: boolean;
    data: T;
    error: string | null;
}

interface LogEntry {
    level: string;
    message: string;
}

interface CodexIPC {
    call<T>(method: string, params?: Record<string, unknown>): Promise<LauncherResponse<T>>;
    launch(): Promise<LauncherResponse<{ ok?: boolean; pid?: number; message?: string }>>;
    stop(): Promise<LauncherResponse<{ ok?: boolean; message?: string }>>;
    saveConfig(cfg: LauncherConfig): Promise<LauncherResponse<{ ok?: boolean; message?: string }>>;
    loadConfig(): Promise<LauncherResponse<LauncherConfig>>;
    healthCheck(): Promise<LauncherResponse<{ running: boolean; apiReady: boolean; error?: string; endpoint?: string }>>;
    listModels(): Promise<LauncherResponse<{ models: { name: string; size: number; digest: string; modified: string }[] }>>;
    refreshModels(): Promise<LauncherResponse<{ models: { name: string; size: number; digest: string; modified: string }[]; message?: string }>>;
    writeCodexConfig(): Promise<LauncherResponse<{ message?: string }>>;
    revertCodexConfig(): Promise<LauncherResponse<{ message?: string }>>;
    detectCodex(): Promise<LauncherResponse<CodexProcessInfo>>;
    killCodexByPid(pid: number): Promise<LauncherResponse<{ message?: string }>>;
    openDirectoryPicker(): Promise<LauncherResponse<{ path: string }>>;
    getAppLogs(): Promise<LauncherResponse<{ logs: LogEntry[] }>>;
    saveSettings(settings: Record<string, unknown>): Promise<LauncherResponse<{ message?: string }>>;
    toggleAutoStart(): Promise<LauncherResponse<{ message?: string; enabled: boolean }>>;
}

declare global {
    interface Window {
        codexIPC: CodexIPC;
        codexRPC: CodexIPC;
    }
}

export {};