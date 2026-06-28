import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type { LauncherConfig, ModelInfo } from "../types";
import { normalizeConfig } from "../lib/endpoint";

export type NavKey = "launcher" | "models" | "settings" | "logs" | "about";

export interface LogEntry {
  level: string;
  message: string;
}

export interface CodexConfigForm {
  codexProviderId: string;
  codexProviderName: string;
  codexConfigPath: string;
  codexCommand: string;
  codexApiPort: string;
  codexApiScheme: string;
  codexArgs: string;
  codexApiKeyMode: string;
}

interface LauncherState {
  isLaunching: boolean;
  running: boolean;
  apiReady: boolean;
  statusMessage: string;
  models: ModelInfo[];
  config: LauncherConfig;
  refreshing: boolean;
}

interface LauncherContextValue extends LauncherState {
  launch: () => Promise<void>;
  stop: () => Promise<void>;
  writeCodexConfig: () => Promise<void>;
  revertCodexConfig: () => Promise<void>;
  refreshModels: () => Promise<void>;
  updateConfig: <K extends keyof LauncherConfig>(key: K, value: LauncherConfig[K]) => void;
  openDirectoryPicker: () => Promise<string | null>;
  getAppLogs: () => Promise<LogEntry[]>;
}

const LauncherContext = createContext<LauncherContextValue | null>(null);

function unwrap<T>(result: { data?: T; error?: string | null }): T {
  if (result.error) {
    throw new Error(result.error);
  }

  const data = result.data;
  if (data && typeof data === "object" && "error" in data) {
    const nested = (data as { error?: unknown }).error;
    if (typeof nested === "string" && nested.length > 0) {
      throw new Error(nested);
    }
  }

  return data as T;
}

export function LauncherProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<LauncherState>({
    isLaunching: false,
    running: false,
    apiReady: false,
    statusMessage: "Initializing...",
    models: [],
    config: {},
    refreshing: false,
  });

  const configRef = useRef<LauncherConfig>({});
  const configLoadedRef = useRef(false);
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    configRef.current = state.config;
  }, [state.config]);

  useEffect(() => {
    if (!configLoadedRef.current) return;

    if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current);
    saveTimeoutRef.current = setTimeout(async () => {
      try {
        await window.codexRPC.saveConfig(configRef.current);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setState((prev) => ({ ...prev, statusMessage: `Failed to save config: ${msg}` }));
      }
    }, 1000);

    return () => {
      if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current);
    };
  }, [state.config]);

  const clearPoll = useCallback(() => {
    if (pollRef.current) {
      clearInterval(pollRef.current);
      pollRef.current = null;
    }
  }, []);

  useEffect(() => () => clearPoll(), [clearPoll]);

  const flushConfigSave = useCallback(async () => {
    if (!configLoadedRef.current) return;

    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
      saveTimeoutRef.current = null;
    }

    await window.codexRPC.saveConfig(configRef.current);
  }, []);

  const loadConfig = useCallback(async () => {
    try {
      const result = await window.codexRPC.loadConfig();
      const config = unwrap<LauncherConfig>(result);
      if (config && typeof config === "object" && !("error" in config)) {
        configLoadedRef.current = true;
        setState((prev) => ({ ...prev, config: normalizeConfig(config) }));
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((prev) => ({ ...prev, statusMessage: `Failed to load config: ${msg}` }));
    }
  }, []);

  const formatStatusMessage = (
    running: boolean,
    apiReady: boolean,
    endpointReady: boolean,
  ) => {
    if (running) {
      return apiReady ? "Codex is running." : "Codex is running (API starting up)...";
    }
    if (endpointReady) {
      return "Codex is stopped. Ollama endpoint is reachable.";
    }
    return "Codex is stopped. Ollama endpoint is not reachable â€” check IP/port.";
  };

  const healthCheck = useCallback(async () => {
    try {
      const result = await window.codexRPC.healthCheck(configRef.current);
      const health = unwrap(result);
      const running = health.running ?? false;
      const apiReady = health.apiReady ?? false;
      const endpointReady = health.endpointReady ?? false;
      setState((prev) => ({
        ...prev,
        running,
        apiReady,
        statusMessage: formatStatusMessage(running, apiReady, endpointReady),
      }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((prev) => ({
        ...prev,
        running: false,
        apiReady: false,
        statusMessage: msg,
      }));
    }
  }, []);

  const refreshModels = useCallback(async () => {
    setState((prev) => ({ ...prev, refreshing: true, statusMessage: "Refreshing models..." }));
    try {
      const snapshot = normalizeConfig(configRef.current);
      configRef.current = snapshot;
      await window.codexRPC.saveConfig(snapshot);
      const result = await window.codexRPC.refreshModels(snapshot);
      const payload = unwrap(result);
      const models = payload.models ?? [];
      const count = Array.isArray(models) ? models.length : 0;
      const detail = payload.message ?? payload.fetchedFrom ?? payload.endpoint;
      setState((prev) => ({
        ...prev,
        models,
        statusMessage:
          count > 0
            ? detail ?? `Found ${count} model(s). Select one below to enable Start.`
            : detail
              ? `No models found at ${detail}. Is Ollama running?`
              : "No models found at this endpoint. Is Ollama running?",
      }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((prev) => ({ ...prev, statusMessage: `Model refresh failed: ${msg}` }));
    } finally {
      setState((prev) => ({ ...prev, refreshing: false }));
    }
  }, []);

  useEffect(() => {
    void (async () => {
      await loadConfig();
      // skip health check — server is expected to be down after stop
      await refreshModels();
    })();
  }, [loadConfig, healthCheck, refreshModels]);

  const launch = useCallback(async () => {
    setState((prev) => ({ ...prev, statusMessage: "Launching Codex..." }));
    setState((prev) => ({ ...prev, isLaunching: true }));
    try {
      await flushConfigSave();
      const result = await window.codexRPC.launch();
      const payload = unwrap(result);
      const message = (payload as { message?: string }).message ?? "Codex launch requested.";
      setState((prev) => ({ ...prev, statusMessage: message }));
      clearPoll();
      pollRef.current = setInterval(async () => {
        try {
          const pollResult = await window.codexRPC.healthCheck(configRef.current);
          const health = unwrap(pollResult);
          const running = health.running ?? false;
          const apiReady = health.apiReady ?? false;
          const endpointReady = health.endpointReady ?? false;
          setState((prev) => ({
            ...prev,
            running,
            apiReady,
            statusMessage: formatStatusMessage(running, apiReady, endpointReady),
          }));
          if (running) clearPoll();
        } catch {
          clearPoll();
        }
      }, 2000);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      clearPoll();
      setState((prev) => ({ ...prev, running: false, statusMessage: msg }));
    }
  }, [clearPoll, flushConfigSave]);

  const stop = useCallback(async () => {
    clearPoll();
    setState((prev) => ({ ...prev, statusMessage: "Stopping Codex..." }));
    try {
      const result = await window.codexRPC.stop();
      const payload = unwrap(result);
      const message = (payload as { message?: string }).message ?? "Codex stopped.";
      // skip health check — server is expected to be down after stop
      setState((prev) => ({ ...prev, statusMessage: message }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      // skip health check — server is expected to be down after stop
      setState((prev) => ({ ...prev, statusMessage: msg }));
    }
  }, [clearPoll]);

  const writeCodexConfig = useCallback(async () => {
    try {
      const result = await window.codexRPC.writeCodexConfig();
      const payload = unwrap(result);
      const message =
        (payload as { message?: string }).message ?? "Operation complete";
      setState((prev) => ({ ...prev, statusMessage: message }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((prev) => ({ ...prev, statusMessage: msg }));
    }
  }, []);

  const revertCodexConfig = useCallback(async () => {
    try {
      const result = await window.codexRPC.revertCodexConfig();
      const payload = unwrap(result);
      const message =
        (payload as { message?: string }).message ?? "Codex profile restored.";
      setState((prev) => ({ ...prev, statusMessage: message }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((prev) => ({ ...prev, statusMessage: msg }));
    }
  }, []);

  const updateConfig = useCallback(
    <K extends keyof LauncherConfig>(key: K, value: LauncherConfig[K]) => {
      setState((prev) => ({
        ...prev,
        config: normalizeConfig({ ...prev.config, [key]: value }),
      }));
    },
    [],
  );

  const openDirectoryPicker = useCallback(async () => {
    try {
      const result = await window.codexRPC.openDirectoryPicker();
      const payload = unwrap(result);
      return payload.path ?? null;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((prev) => ({ ...prev, statusMessage: msg }));
      return null;
    }
  }, []);

  const getAppLogs = useCallback(async () => {
    try {
      const result = await window.codexRPC.getAppLogs();
      const payload = unwrap(result);
      return payload.logs ?? [];
    } catch {
      return [];
    }
  }, []);

  const value: LauncherContextValue = {
    ...state,
    launch,
    stop,
    writeCodexConfig,
    revertCodexConfig,
    refreshModels,
    updateConfig,
    openDirectoryPicker,
    getAppLogs,
  };

  return <LauncherContext.Provider value={value}>{children}</LauncherContext.Provider>;
}

export function useLauncher() {
  const ctx = useContext(LauncherContext);
  if (!ctx) throw new Error("useLauncher must be used within LauncherProvider");
  return ctx;
}

export function configToCodexForm(config: LauncherConfig): CodexConfigForm {
  const get = <K extends keyof LauncherConfig>(key: K, fallback: string) =>
    (config[key] as string | undefined) ?? fallback;

  return {
    codexProviderId: get("codexProviderId", "codex-local-launcher"),
    codexProviderName: get("codexProviderName", "Codex Local Launcher"),
    codexConfigPath: get("codexConfigPath", ""),
    codexCommand: get("codexCommand", ""),
    codexApiPort: String(config.codexApiPort ?? 4000),
    codexApiScheme: get("codexApiScheme", "http"),
    codexArgs: Array.isArray(config.codexArgs) ? config.codexArgs.join(",") : "",
    codexApiKeyMode: get("codexApiKeyMode", "experimentalBearerToken"),
  };
}