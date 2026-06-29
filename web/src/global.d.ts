/// <reference types="vite/client" />

import type { CodexConfigInspection } from './lib/codexProfile';
import type {
    FailoverStatus,
    LauncherConfig,
    CodexProcessInfo,
    CodexRateLimitsStatus,
    CodexSessionListDetail,
    CodexThreadListStatus,
    DiscoveryLogEntry,
    SessionCheckpoint,
} from './types';

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
    writeCodexConfig(cfg?: LauncherConfig): Promise<LauncherResponse<{ message?: string; inspection?: CodexConfigInspection }>>;
    syncCodexConfig(cfg?: LauncherConfig): Promise<LauncherResponse<{ message?: string; inspection?: CodexConfigInspection }>>;
    inspectCodexConfig(): Promise<LauncherResponse<CodexConfigInspection>>;
    revertCodexConfig(cfg?: LauncherConfig): Promise<LauncherResponse<{ message?: string; inspection?: CodexConfigInspection }>>;
    detectCodex(): Promise<LauncherResponse<CodexProcessInfo>>;
    killCodexByPid(pid: number): Promise<LauncherResponse<{ message?: string }>>;

    getAppLogs(): Promise<LauncherResponse<{ logs: LogEntry[] }>>;
    saveSettings(settings: Record<string, unknown>): Promise<LauncherResponse<{ message?: string }>>;
    toggleAutoStart(): Promise<LauncherResponse<{ message?: string; enabled: boolean }>>;
    setAutoStart(enabled: boolean): Promise<LauncherResponse<{ message?: string; enabled: boolean }>>;
    getFailoverStatus(): Promise<LauncherResponse<FailoverStatus>>;
    dismissFailoverAlert(): Promise<LauncherResponse<{ ok?: boolean }>>;
    dismissConnectionAlert(): Promise<LauncherResponse<{ ok?: boolean }>>;
    failoverToLocal(profileName?: string): Promise<LauncherResponse<{ ok?: boolean; message?: string; profileName?: string; resumePrompt?: string; checkpoint?: SessionCheckpoint }>>;
    captureSessionCheckpoint(trigger?: string): Promise<LauncherResponse<{ ok?: boolean; checkpoint?: SessionCheckpoint | null }>>;
    listSessionCheckpoints(): Promise<LauncherResponse<{ checkpoints: SessionCheckpoint[] }>>;
    listCodexSessions(): Promise<LauncherResponse<{ sessions: { sessionId: string; createdAt?: string | null }[] }>>;
    listCodexSessionsDetailed(cfg?: LauncherConfig): Promise<LauncherResponse<CodexSessionListDetail>>;
    listCodexThreads(cfg?: LauncherConfig): Promise<LauncherResponse<CodexThreadListStatus>>;
    getDiscoveryLogs(options?: { limit?: number; stream?: 'all' | 'rateLimit' | 'connection' }): Promise<LauncherResponse<{ entries: DiscoveryLogEntry[] }>>;
    probeCodexApi(cfg?: LauncherConfig): Promise<LauncherResponse<{ codexApiBaseUrl: string; healthOk: boolean; restSessionsSupported: boolean; appServerWebSocketUrl: string; notes: string[] }>>;
    getCodexRateLimits(cfg?: LauncherConfig): Promise<LauncherResponse<CodexRateLimitsStatus>>;
}

declare global {
    interface Window {
        codexIPC: CodexIPC;
        codexRPC: CodexIPC;
    }
}

export {};
