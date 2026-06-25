import type { LauncherConfig, CodexProcessInfo } from "../types";

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
  launch(): Promise<LauncherResponse<{ ok?: boolean; pid?: number; message?: string }>>;
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
  writeCodexConfig(): Promise<LauncherResponse<{ message?: string }>>;
  revertCodexConfig(): Promise<LauncherResponse<{ message?: string }>>;
  detectCodex(): Promise<LauncherResponse<CodexProcessInfo>>;
  killCodexByPid(pid: number): Promise<LauncherResponse<{ message?: string }>>;
  getAppLogs(): Promise<LauncherResponse<{ logs: LogEntry[] }>>;
  saveSettings(settings: Record<string, unknown>): Promise<LauncherResponse<{ message?: string }>>;
  toggleAutoStart(): Promise<LauncherResponse<{ message?: string; enabled: boolean }>>;
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
    launch: () => call("launch"),
    stop: () => call("stop"),
    saveConfig: (cfg) => call("saveConfig", cfg as unknown as Record<string, unknown>),
    loadConfig: () => call("loadConfig"),
    healthCheck: (cfg) => call("healthCheck", (cfg ?? {}) as unknown as Record<string, unknown>),
    listModels: () => call("listModels"),
    refreshModels: (cfg) => call("refreshModels", (cfg ?? {}) as unknown as Record<string, unknown>),
    writeCodexConfig: () => call("writeCodexConfig"),
    revertCodexConfig: () => call("revertCodexConfig"),
    detectCodex: () => call("detectCodex"),
    killCodexByPid: (pid) => call("killCodexByPid", { pid }),
    getAppLogs: () => call("getAppLogs"),
    saveSettings: (settings) => call("saveSettings", settings),
    toggleAutoStart: () => call("toggleAutoStart"),
  };
}

export function installCodexRpcClient(): void {
  if (!window.codexRPC) {
    window.codexRPC = createCodexRpcClient();
  }
  window.codexIPC = window.codexRPC;
}