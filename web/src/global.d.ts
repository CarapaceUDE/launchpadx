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
    launch(cfg?: LauncherConfig): Promise<LauncherResponse<{ ok?: boolean; pid?: number; message?: string }>>;
    
    stop(): Promise<LauncherResponse<{ ok?: boolean; message?: string }>>;
    saveConfig(cfg: LauncherConfig): Promise<LauncherResponse<{ ok?: boolean; message?: string }>>;
    loadConfig(): Promise<LauncherResponse<LauncherConfig>>;
    healthCheck(cfg?: LauncherConfig): Promise<LauncherResponse<{ running: boolean; apiReady: boolean; endpointReady?: boolean; pid?: number | null; method?: string | null; error?: string }>>;
    listModels(): Promise<LauncherResponse<{ models: { name: string; size: number; digest: string; modified: string }[] }>>;
    refreshModels(cfg?: LauncherConfig): Promise<LauncherResponse<{ models: { name: string; size: number; digest: string; modified: string }[]; message?: string; endpoint?: string; fetchedFrom?: string }>>;
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
