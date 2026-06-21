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
  running: boolean;
  apiReady: boolean;
  statusMessage: string;
  models: ModelInfo[];
  config: LauncherConfig;
  refreshing: boolean;
  saving: boolean;
}

interface LauncherContextValue extends LauncherState {
  launch: () => Promise<void>;
  stop: () => Promise<void>;
  saveConfig: () => Promise<void>;
  writeCodexConfig: () => Promise<void>;
  refreshModels: () => Promise<void>;
  updateConfig: <K extends keyof LauncherConfig>(key: K, value: LauncherConfig[K]) => void;
  openDirectoryPicker: () => Promise<string | null>;
  getAppLogs: () => Promise<LogEntry[]>;
}

const LauncherContext = createContext<LauncherContextValue | null>(null);

function unwrap<T>(result: { data?: T }): T {
  return result.data as T;
}

export function LauncherProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<LauncherState>({
    running: false,
    apiReady: false,
    statusMessage: "Initializing...",
    models: [],
    config: {},
    refreshing: false,
    saving: false,
  });

  const configRef = useRef<LauncherConfig>({});
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    configRef.current = state.config;
  }, [state.config]);

  useEffect(() => {
    if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current);
    saveTimeoutRef.current = setTimeout(async () => {
      try {
        await window.codexRPC.saveConfig(configRef.current);
      } catch {
        // auto-save should not disrupt UX
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

  const loadConfig = useCallback(async () => {
    try {
      const result = await window.codexRPC.loadConfig();
      const config = unwrap<LauncherConfig>(result);
      if (config && typeof config === "object" && !("error" in config)) {
        setState((prev) => ({ ...prev, config }));
      }
    } catch {
      setState((prev) => ({ ...prev, statusMessage: "Failed to load config" }));
    }
  }, []);

  const healthCheck = useCallback(async () => {
    try {
      const result = await window.codexRPC.healthCheck();
      const health = unwrap(result);
      const running = health.running ?? false;
      const apiReady = health.apiReady ?? false;
      setState((prev) => ({
        ...prev,
        running,
        apiReady,
        statusMessage: running
          ? apiReady
            ? "Codex API is accepting requests."
            : "Codex is starting up..."
          : "Codex API is stopped",
      }));
    } catch {
      setState((prev) => ({
        ...prev,
        running: false,
        apiReady: false,
        statusMessage: "Codex API is stopped",
      }));
    }
  }, []);

  const refreshModels = useCallback(async () => {
    setState((prev) => ({ ...prev, refreshing: true }));
    try {
      const result = await window.codexRPC.refreshModels();
      const payload = unwrap(result);
      const models = payload.models ?? [];
      const count = Array.isArray(models) ? models.length : 0;
      setState((prev) => ({
        ...prev,
        models,
        statusMessage: count > 0 ? `Found ${count} model(s).` : "No models found.",
      }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((prev) => ({ ...prev, statusMessage: msg }));
    } finally {
      setState((prev) => ({ ...prev, refreshing: false }));
    }
  }, []);

  useEffect(() => {
    void loadConfig();
    void healthCheck();
    void refreshModels();
  }, [loadConfig, healthCheck, refreshModels]);

  const launch = useCallback(async () => {
    setState((prev) => ({ ...prev, statusMessage: "Launching Codex..." }));
    try {
      await window.codexRPC.launch();
      setState((prev) => ({
        ...prev,
        running: true,
        statusMessage: "Codex launched successfully",
      }));
      clearPoll();
      pollRef.current = setInterval(async () => {
        try {
          const result = await window.codexRPC.healthCheck();
          const health = unwrap(result);
          const running = health.running ?? false;
          const apiReady = health.apiReady ?? false;
          setState((prev) => ({
            ...prev,
            running,
            apiReady,
            statusMessage: running
              ? apiReady
                ? "Codex API is accepting requests."
                : "Codex is starting up..."
              : "Codex stopped",
          }));
          if (!running) clearPoll();
        } catch {
          clearPoll();
        }
      }, 3000);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      clearPoll();
      setState((prev) => ({ ...prev, running: false, statusMessage: msg }));
    }
  }, [clearPoll]);

  const stop = useCallback(async () => {
    clearPoll();
    setState((prev) => ({ ...prev, statusMessage: "Stopping Codex..." }));
    try {
      await window.codexRPC.stop();
      setState((prev) => ({
        ...prev,
        running: false,
        apiReady: false,
        statusMessage: "Codex stopped",
      }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((prev) => ({ ...prev, running: false, statusMessage: msg }));
    }
  }, [clearPoll]);

  const saveConfig = useCallback(async () => {
    setState((prev) => ({ ...prev, saving: true }));
    try {
      await window.codexRPC.saveConfig(configRef.current);
      setState((prev) => ({ ...prev, statusMessage: "Configuration saved." }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((prev) => ({ ...prev, statusMessage: msg }));
    } finally {
      setState((prev) => ({ ...prev, saving: false }));
    }
  }, []);

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

  const updateConfig = useCallback(
    <K extends keyof LauncherConfig>(key: K, value: LauncherConfig[K]) => {
      setState((prev) => ({
        ...prev,
        config: { ...prev.config, [key]: value },
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
    saveConfig,
    writeCodexConfig,
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