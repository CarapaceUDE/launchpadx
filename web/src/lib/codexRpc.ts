import type { CodexConfigInspection } from "./codexProfile";
import type {
  CodexProcessInfo,
  FailoverStatus,
  LauncherConfig,
  SessionCheckpoint,
} from "../types";

export interface LauncherResponse<T> {
  ok: boolean;
  data: T;
  error: string | null;
}

interface LogEntry {
  level: string;
  message: string;
}

export interface CodexRpcClient {
  call<T>(method: string, params?: Record<string, unknown>): Promise<LauncherResponse<T>>;
  launch(
    cfg?: LauncherConfig,
  ): Promise<LauncherResponse<{ ok?: boolean; pid?: number; message?: string }>>;
  stop(): Promise<LauncherResponse<{ ok?: boolean; message?: string }>>;
  saveConfig(cfg: LauncherConfig): Promise<LauncherResponse<{ ok?: boolean; message?: string }>>;
  loadConfig(): Promise<LauncherResponse<LauncherConfig>>;
  healthCheck(
    cfg?: LauncherConfig,
  ): Promise<
    LauncherResponse<{
      running: boolean;
      apiReady: boolean;
      endpointReady?: boolean;
      pid?: number | null;
      method?: string | null;
      error?: string;
    }>
  >;
  listModels(): Promise<
    LauncherResponse<{ models: { name: string; size: number; digest: string; modified: string }[] }>
  >;
  refreshModels(
    cfg?: LauncherConfig,
  ): Promise<
    LauncherResponse<{
      models: { name: string; size: number; digest: string; modified: string }[];
      message?: string;
      endpoint?: string;
      fetchedFrom?: string;
    }>
  >;
  writeCodexConfig(
    cfg?: LauncherConfig,
  ): Promise<LauncherResponse<{ message?: string; inspection?: CodexConfigInspection }>>;
  syncCodexConfig(
    cfg?: LauncherConfig,
  ): Promise<LauncherResponse<{ message?: string; inspection?: CodexConfigInspection }>>;
  inspectCodexConfig(): Promise<LauncherResponse<CodexConfigInspection>>;
  revertCodexConfig(
    cfg?: LauncherConfig,
  ): Promise<LauncherResponse<{ message?: string; inspection?: CodexConfigInspection }>>;
  detectCodex(): Promise<LauncherResponse<CodexProcessInfo>>;
  killCodexByPid(pid: number): Promise<LauncherResponse<{ message?: string }>>;
  getAppLogs(): Promise<LauncherResponse<{ logs: LogEntry[] }>>;
  saveSettings(settings: Record<string, unknown>): Promise<LauncherResponse<{ message?: string }>>;
  toggleAutoStart(): Promise<LauncherResponse<{ message?: string; enabled: boolean }>>;
  setAutoStart(enabled: boolean): Promise<LauncherResponse<{ message?: string; enabled: boolean }>>;
  getFailoverStatus(): Promise<LauncherResponse<FailoverStatus>>;
  dismissFailoverAlert(): Promise<LauncherResponse<{ ok?: boolean }>>;
  failoverToLocal(
    profileName?: string,
  ): Promise<
    LauncherResponse<{
      ok?: boolean;
      message?: string;
      profileName?: string;
      resumePrompt?: string;
      checkpoint?: SessionCheckpoint;
    }>
  >;
  captureSessionCheckpoint(
    trigger?: string,
  ): Promise<LauncherResponse<{ ok?: boolean; checkpoint?: SessionCheckpoint | null }>>;
  listSessionCheckpoints(): Promise<LauncherResponse<{ checkpoints: SessionCheckpoint[] }>>;
  listCodexSessions(): Promise<
    LauncherResponse<{ sessions: { sessionId: string; createdAt?: string | null }[] }>
  >;
  probeCodexApi(
    cfg?: LauncherConfig,
  ): Promise<
    LauncherResponse<{
      codexApiBaseUrl: string;
      healthOk: boolean;
      restSessionsSupported: boolean;
      appServerWebSocketUrl: string;
      notes: string[];
    }>
  >;
}

export function createCodexRpcClient(): CodexRpcClient {
  async function call<T>(
    method: string,
    params: Record<string, unknown> = {},
  ): Promise<LauncherResponse<T>> {
    const response = await fetch("/rpc", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ method, params }),
    });

    if (!response.ok) {
      throw new Error(`RPC ${method} failed with HTTP ${response.status}`);
    }

    return (await response.json()) as LauncherResponse<T>;
  }

  return {
    call,
    launch: (cfg) => call("launch", (cfg ?? {}) as unknown as Record<string, unknown>),
    stop: () => call("stop"),
    saveConfig: (cfg) => call("saveConfig", cfg as unknown as Record<string, unknown>),
    loadConfig: () => call("loadConfig"),
    healthCheck: (cfg) => call("healthCheck", (cfg ?? {}) as unknown as Record<string, unknown>),
    listModels: () => call("listModels"),
    refreshModels: (cfg) => call("refreshModels", (cfg ?? {}) as unknown as Record<string, unknown>),
    writeCodexConfig: (cfg) =>
      call("writeCodexConfig", (cfg ?? {}) as unknown as Record<string, unknown>),
    syncCodexConfig: (cfg) =>
      call("syncCodexConfig", (cfg ?? {}) as unknown as Record<string, unknown>),
    inspectCodexConfig: () => call("inspectCodexConfig"),
    revertCodexConfig: (cfg) =>
      call("revertCodexConfig", (cfg ?? {}) as unknown as Record<string, unknown>),
    detectCodex: () => call("detectCodex"),
    killCodexByPid: (pid) => call("killCodexByPid", { pid }),
    getAppLogs: () => call("getAppLogs"),
    saveSettings: (settings) => call("saveSettings", settings),
    toggleAutoStart: () => call("toggleAutoStart"),
    setAutoStart: (enabled) => call("setAutoStart", { enabled }),
    getFailoverStatus: () => call("getFailoverStatus"),
    dismissFailoverAlert: () => call("dismissFailoverAlert"),
    failoverToLocal: (profileName) =>
      call("failoverToLocal", profileName ? { profileName } : {}),
    captureSessionCheckpoint: (trigger) =>
      call("captureSessionCheckpoint", trigger ? { trigger } : {}),
    listSessionCheckpoints: () => call("listSessionCheckpoints"),
    listCodexSessions: () => call("listCodexSessions"),
    probeCodexApi: (cfg) => call("probeCodexApi", (cfg ?? {}) as unknown as Record<string, unknown>),
  };
}

export function installCodexRpcClient(): void {
  if (!window.codexRPC) {
    window.codexRPC = createCodexRpcClient();
  }
  window.codexIPC = window.codexRPC;
}