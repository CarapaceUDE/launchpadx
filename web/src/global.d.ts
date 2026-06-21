import type { LauncherConfig, CodexProcessInfo } from './types';

interface LauncherResponse<T> {
    ok: boolean;
    data: T;
    error: string | null;
}

interface CodexIPC {
    invoke<T>(cmd: string, payload?: Record<string, unknown>): Promise<T>;
    launch(): Promise<LauncherResponse<{ message: string }>>;
    stop(): Promise<LauncherResponse<{ message: string }>>;
    saveConfig(cfg: LauncherConfig): Promise<LauncherResponse<{ message: string }>>;
    loadConfig(): Promise<LauncherResponse<LauncherConfig>>;
    healthCheck(): Promise<LauncherResponse<{ running: boolean; apiReady: boolean; error?: string; endpoint?: string }>>;
    listModels(): Promise<LauncherResponse<{ models: { name: string; size: number; digest: string; modified: string }[]; fetchedFrom: string }>>;
    refreshModels(): Promise<LauncherResponse<{ models: { name: string; size: number; digest: string; modified: string }[]; fetchedFrom: string }>>;
    writeCodexConfig(): Promise<LauncherResponse<{ message: string }>>;
    revertCodexConfig(): Promise<LauncherResponse<{ message: string }>>;
    detectCodex(): Promise<LauncherResponse<CodexProcessInfo>>;
    killCodexByPid(pid: number): Promise<LauncherResponse<{ message: string }>>;
    openDirectoryPicker(): Promise<LauncherResponse<{ path: string }>>;
    getAppLogs(): Promise<LauncherResponse<{ logs: string }>>;
    saveSettings(settings: Record<string, unknown>): Promise<LauncherResponse<{ message: string }>>;
    toggleAutoStart(): Promise<LauncherResponse<{ message: string; enabled: boolean }>>;
}

declare global {
    interface Window {
        codexIPC: CodexIPC;
        codexRPC: CodexIPC;
        }
}

export {};
