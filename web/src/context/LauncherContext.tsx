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
import {
  activeProviderMode,
  inspectionToProfile,
  shouldAutoSyncCodex,
  type CodexConfigInspection,
  type CodexProfileState,
  type ProviderMode,
} from "../lib/codexProfile";
import { pickDefaultModel, reconcileModelSelection } from "../lib/modelSelection";
import {
  blocksLaunchToggle,
  healthStatusMessage,
  shouldPreserveOperationStatus,
  type LauncherOperation,
} from "../lib/operationStatus";
import {
  blocksProviderSwitch,
  canStartCodex,
  localActivationRequirements,
  persistCodexDisabledMessage,
  providerSwitchSuccessMessage,
  shouldSyncCodexToDisk,
} from "../lib/providerGuards";
import type { ServerPillState } from "../components/launcher/primitives";

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

export type StatusVariant = "default" | "success" | "error";

interface LauncherState {
  running: boolean;
  apiReady: boolean;
  statusMessage: string;
  statusVariant: StatusVariant;
  operation: LauncherOperation;
  serverState: ServerPillState;
  models: ModelInfo[];
  config: LauncherConfig;
  refreshing: boolean;
  codexProfile: CodexProfileState;
  codexSyncing: boolean;
  writingCodex: boolean;
  revertingCodex: boolean;
}

interface LauncherContextValue extends LauncherState {
  launch: () => Promise<void>;
  stop: () => Promise<void>;
  writeCodexConfig: () => Promise<void>;
  revertCodexConfig: () => Promise<void>;
  switchProvider: (mode: ProviderMode) => Promise<void>;
  refreshModels: () => Promise<void>;
  updateConfig: <K extends keyof LauncherConfig>(key: K, value: LauncherConfig[K]) => void;
  selectModel: (model: string) => Promise<void>;
  setAutoStart: (enabled: boolean) => Promise<void>;
  getAppLogs: () => Promise<LogEntry[]>;
}

const LauncherContext = createContext<LauncherContextValue | null>(null);

const SAVE_DEBOUNCE_MS = 800;
const CODEX_SYNC_DEBOUNCE_MS = 1200;
const HEALTH_POLL_MS = 4000;

const CODEX_SYNC_KEYS: Array<keyof LauncherConfig> = [
  "codexModel",
  "ollamaIp",
  "ollamaPort",
  "ollamaScheme",
  "openaiBaseUrl",
  "apiKey",
  "codexApiKeyMode",
  "codexProviderId",
  "codexProviderName",
  "codexConfigPath",
  "persistCodexConfig",
];

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

function applyInspection(
  inspection: CodexConfigInspection | undefined,
  fallback: CodexProfileState,
): CodexProfileState {
  return inspection ? inspectionToProfile(inspection) : fallback;
}

export function LauncherProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<LauncherState>({
    running: false,
    apiReady: false,
    statusMessage: "Starting launcher...",
    statusVariant: "default",
    operation: "initializing",
    serverState: "stopped",
    models: [],
    config: {},
    refreshing: false,
    codexProfile: { status: "unknown", restoreAvailable: false },
    codexSyncing: false,
    writingCodex: false,
    revertingCodex: false,
  });

  const configRef = useRef<LauncherConfig>({});
  const configLoadedRef = useRef(false);
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const codexSyncTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const healthPollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const saveGenerationRef = useRef(0);
  const modelsRef = useRef<ModelInfo[]>([]);
  const codexProfileRef = useRef<CodexProfileState>(state.codexProfile);
  const operationRef = useRef<LauncherOperation>("initializing");

  useEffect(() => {
    configRef.current = state.config;
  }, [state.config]);

  useEffect(() => {
    codexProfileRef.current = state.codexProfile;
  }, [state.codexProfile]);

  const setOperation = useCallback((operation: LauncherOperation, message: string) => {
    operationRef.current = operation;
    setState((prev) => ({
      ...prev,
      operation,
      statusMessage: message,
      statusVariant: "default",
      serverState:
        operation === "launching" || operation === "waiting_for_codex"
          ? "starting"
          : operation === "stopping"
            ? "stopping"
            : prev.serverState,
    }));
  }, []);

  const applyHealth = useCallback(
    (
      running: boolean,
      apiReady: boolean,
      endpointReady: boolean,
      force = false,
      quiet = false,
    ) => {
      const op = operationRef.current;
      const providerMode = activeProviderMode(codexProfileRef.current);
      const nextServerState = running ? "running" : "stopped";

      if (!force && shouldPreserveOperationStatus(op)) {
        if (
          (op === "launching" || op === "waiting_for_codex") &&
          running
        ) {
          operationRef.current = "idle";
          setState((prev) => ({
            ...prev,
            operation: "idle",
            running,
            apiReady,
            serverState: "running",
            statusMessage: healthStatusMessage(running, apiReady, endpointReady, providerMode),
            statusVariant: "success",
          }));
        }
        return;
      }

      operationRef.current = "idle";
      setState((prev) => {
        const runningChanged =
          prev.running !== running || prev.serverState !== nextServerState;
        const shouldUpdateMessage = force || !quiet || runningChanged;

        return {
          ...prev,
          operation: "idle",
          running,
          apiReady,
          serverState: nextServerState,
          statusMessage: shouldUpdateMessage
            ? healthStatusMessage(running, apiReady, endpointReady, providerMode)
            : prev.statusMessage,
          statusVariant: shouldUpdateMessage && !quiet ? "default" : prev.statusVariant,
        };
      });
    },
    [],
  );

  const clearPoll = useCallback(() => {
    if (pollRef.current) {
      clearInterval(pollRef.current);
      pollRef.current = null;
    }
  }, []);

  const clearHealthPoll = useCallback(() => {
    if (healthPollRef.current) {
      clearInterval(healthPollRef.current);
      healthPollRef.current = null;
    }
  }, []);

  useEffect(() => () => {
    clearPoll();
    clearHealthPoll();
  }, [clearPoll, clearHealthPoll]);

  const flushConfigSave = useCallback(async () => {
    if (!configLoadedRef.current) return;

    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
      saveTimeoutRef.current = null;
    }

    saveGenerationRef.current += 1;
    await window.codexRPC.saveConfig(configRef.current);
  }, []);

  const inspectCodexProfile = useCallback(async (): Promise<CodexProfileState> => {
    try {
      const result = await window.codexRPC.inspectCodexConfig();
      const inspection = unwrap<CodexConfigInspection>(result);
      const profile = inspectionToProfile(inspection);
      setState((prev) => ({ ...prev, codexProfile: profile }));
      return profile;
    } catch {
      const profile: CodexProfileState = { status: "unknown", restoreAvailable: false };
      setState((prev) => ({ ...prev, codexProfile: profile }));
      return profile;
    }
  }, []);

  const syncCodexProfile = useCallback(
    async (
      snapshot?: LauncherConfig,
      quiet = false,
      statusMessage = "Updating Local API settings...",
      force = false,
    ) => {
      const cfg = normalizeConfig(snapshot ?? configRef.current);
      if (!force && !shouldSyncCodexToDisk(codexProfileRef.current, cfg)) return;
      if (!shouldAutoSyncCodex(cfg)) return;

      if (!quiet) {
        setOperation("syncing_codex", statusMessage);
      }
      setState((prev) => ({ ...prev, codexSyncing: true }));
      try {
        await flushConfigSave();
        const result = await window.codexRPC.syncCodexConfig(cfg);
        const payload = unwrap<{ message?: string; inspection?: CodexConfigInspection }>(result);
        const profile = applyInspection(payload.inspection, {
          status: "managed",
          restoreAvailable: true,
        });
        if (!quiet) {
          operationRef.current = "idle";
        }
        setState((prev) => ({
          ...prev,
          codexProfile: profile,
          codexSyncing: false,
          operation: quiet ? prev.operation : "idle",
          statusMessage: quiet
            ? prev.statusMessage
            : providerSwitchSuccessMessage(
                payload.message ?? "Codex profile synchronized.",
                force && prev.serverState === "running",
              ),
          statusVariant: quiet ? prev.statusVariant : "success",
        }));
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        if (!quiet) {
          operationRef.current = "idle";
        }
        setState((prev) => ({
          ...prev,
          codexSyncing: false,
          operation: quiet ? prev.operation : "idle",
          statusMessage: quiet ? prev.statusMessage : `Codex sync failed: ${msg}`,
          statusVariant: quiet ? prev.statusVariant : "error",
        }));
      }
    },
    [flushConfigSave, setOperation],
  );

  const queueCodexSync = useCallback(
    (quiet = true) => {
      if (!configLoadedRef.current) return;
      if (!shouldSyncCodexToDisk(codexProfileRef.current, configRef.current)) return;

      if (codexSyncTimeoutRef.current) clearTimeout(codexSyncTimeoutRef.current);
      codexSyncTimeoutRef.current = setTimeout(() => {
        void syncCodexProfile(undefined, quiet);
      }, CODEX_SYNC_DEBOUNCE_MS);
    },
    [syncCodexProfile],
  );

  useEffect(() => {
    if (!configLoadedRef.current) return;

    const generation = ++saveGenerationRef.current;
    if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current);
    saveTimeoutRef.current = setTimeout(async () => {
      try {
        await window.codexRPC.saveConfig(configRef.current);
        if (generation !== saveGenerationRef.current) return;
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setState((prev) => ({ ...prev, statusMessage: `Failed to save config: ${msg}` }));
      }
    }, SAVE_DEBOUNCE_MS);

    return () => {
      if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current);
    };
  }, [state.config]);

  const reconcileModelsInConfig = useCallback(
    (models: ModelInfo[], config: LauncherConfig): LauncherConfig => {
      const modelNames = models.map((m) => m.name);
      const reconciled = reconcileModelSelection(modelNames, config.codexModel);
      if (reconciled === config.codexModel || (!reconciled && !config.codexModel)) {
        return config;
      }

      return normalizeConfig({
        ...config,
        codexModel: reconciled,
      });
    },
    [],
  );

  const bootstrapFromCodex = useCallback(
    (config: LauncherConfig, inspection: CodexProfileState, models: ModelInfo[]) => {
      let next = normalizeConfig(config);
      const modelNames = models.map((m) => m.name);
      const before = next.codexModel ?? "";

      if (!next.codexModel?.trim() && inspection.model) {
        const adopted = reconcileModelSelection(modelNames, inspection.model) ?? inspection.model;
        next = normalizeConfig({ ...next, codexModel: adopted });
      }

      if (!next.codexModel?.trim() && modelNames.length > 0) {
        next = normalizeConfig({
          ...next,
          codexModel: pickDefaultModel(modelNames, next.codexModel),
        });
      }

      if ((next.codexModel ?? "") !== before) {
        configRef.current = next;
        setState((prev) => ({ ...prev, config: next }));
        void flushConfigSave();
        queueCodexSync();
      }
    },
    [flushConfigSave, queueCodexSync],
  );

  const loadConfig = useCallback(async () => {
    try {
      const result = await window.codexRPC.loadConfig();
      const config = unwrap<LauncherConfig>(result);
      if (config && typeof config === "object" && !("error" in config)) {
        configLoadedRef.current = true;
        const normalized = normalizeConfig(config);
        configRef.current = normalized;
        setState((prev) => ({ ...prev, config: normalized }));
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((prev) => ({ ...prev, statusMessage: `Failed to load config: ${msg}` }));
    }
  }, []);

  const healthCheck = useCallback(async (quiet = false) => {
    try {
      const result = await window.codexRPC.healthCheck(configRef.current);
      const health = unwrap(result);
      applyHealth(
        health.running ?? false,
        health.apiReady ?? false,
        health.endpointReady ?? false,
        false,
        quiet,
      );
    } catch (e) {
      if (quiet) return;
      const msg = e instanceof Error ? e.message : String(e);
      operationRef.current = "idle";
      setState((prev) => ({
        ...prev,
        operation: "idle",
        running: false,
        apiReady: false,
        serverState: "stopped",
        statusMessage: msg,
        statusVariant: "error",
      }));
    }
  }, [applyHealth]);

  useEffect(() => {
    healthPollRef.current = setInterval(() => {
      if (shouldPreserveOperationStatus(operationRef.current)) return;
      void healthCheck(true);
    }, HEALTH_POLL_MS);

    return () => clearHealthPoll();
  }, [clearHealthPoll, healthCheck]);

  const refreshModels = useCallback(async (): Promise<void> => {
    setOperation("refreshing_models", "Refreshing models from endpoint...");
    setState((prev) => ({ ...prev, refreshing: true }));
    try {
      await flushConfigSave();
      const snapshot = normalizeConfig(configRef.current);
      configRef.current = snapshot;
      const result = await window.codexRPC.refreshModels(snapshot);
      const payload = unwrap(result);
      const models = payload.models ?? [];
      const count = Array.isArray(models) ? models.length : 0;
      const detail = payload.message ?? payload.fetchedFrom ?? payload.endpoint;

      modelsRef.current = models;
      const reconciledConfig = reconcileModelsInConfig(models, configRef.current);
      configRef.current = reconciledConfig;
      operationRef.current = "idle";
      setState((prev) => ({
        ...prev,
        operation: "idle",
        models,
        config: reconciledConfig,
        statusMessage:
          count > 0
            ? (detail ?? `Found ${count} model(s). Select one below to enable Start.`)
            : detail
              ? `No models found at ${detail}. Is Ollama running?`
              : "No models found at this endpoint. Is Ollama running?",
        statusVariant: count > 0 ? "success" : "default",
      }));

      queueCodexSync();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      operationRef.current = "idle";
      setState((prev) => ({
        ...prev,
        operation: "idle",
        statusMessage: `Model refresh failed: ${msg}`,
        statusVariant: "error",
      }));
    } finally {
      setState((prev) => ({ ...prev, refreshing: false }));
    }
  }, [flushConfigSave, queueCodexSync, reconcileModelsInConfig, setOperation]);

  useEffect(() => {
    void (async () => {
      setOperation("initializing", "Loading configuration...");
      await loadConfig();
      setOperation("initializing", "Inspecting Codex profile...");
      const profile = await inspectCodexProfile();
      setOperation("initializing", "Checking Codex status...");
      await healthCheck();
      setOperation("initializing", "Discovering models...");
      await refreshModels();
      bootstrapFromCodex(configRef.current, profile, modelsRef.current);
      if (operationRef.current === "initializing") {
        operationRef.current = "idle";
        setState((prev) => ({ ...prev, operation: "idle" }));
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const launch = useCallback(async () => {
    if (blocksLaunchToggle(operationRef.current)) {
      setState((prev) => ({
        ...prev,
        statusMessage: "Wait for the current operation to finish before starting Codex.",
        statusVariant: "error",
      }));
      return;
    }

    const startCheck = canStartCodex(
      activeProviderMode(codexProfileRef.current),
      configRef.current,
      modelsRef.current.length,
    );
    if (!startCheck.canStart) {
      setState((prev) => ({
        ...prev,
        statusMessage: startCheck.reason ?? "Cannot start Codex with the current settings.",
        statusVariant: "error",
      }));
      return;
    }

    setOperation("launching", "Starting Codex — saving your settings...");
    try {
      await flushConfigSave();
      if (shouldSyncCodexToDisk(codexProfileRef.current, configRef.current)) {
        setOperation("launching", "Starting Codex — syncing Codex profile...");
        await syncCodexProfile(configRef.current, true, undefined, true);
      }
      setOperation("launching", "Starting Codex — launching process...");
      const result = await window.codexRPC.launch(configRef.current);
      const payload = unwrap(result);
      const message = (payload as { message?: string }).message ?? "Codex launch requested.";
      setOperation("waiting_for_codex", `${message} Waiting for Codex to appear...`);
      clearPoll();
      let attempts = 0;
      pollRef.current = setInterval(async () => {
        attempts += 1;
        try {
          const pollResult = await window.codexRPC.healthCheck(configRef.current);
          const health = unwrap(pollResult);
          const running = health.running ?? false;
          const apiReady = health.apiReady ?? false;
          const endpointReady = health.endpointReady ?? false;

          if (running) {
            clearPoll();
            applyHealth(running, apiReady, endpointReady, true);
            return;
          }

          if (attempts >= 30) {
            clearPoll();
            operationRef.current = "idle";
            setState((prev) => ({
              ...prev,
              operation: "idle",
              serverState: "stopped",
              statusMessage:
                "Codex did not start within 60 seconds. Check logs or try again.",
              statusVariant: "error",
            }));
          } else if (attempts % 3 === 0) {
            setState((prev) => ({
              ...prev,
              statusMessage: `Still waiting for Codex to start (${attempts * 2}s)...`,
            }));
          }
        } catch {
          clearPoll();
          operationRef.current = "idle";
          setState((prev) => ({
            ...prev,
            operation: "idle",
            serverState: "stopped",
            statusVariant: "error",
          }));
        }
      }, 2000);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      clearPoll();
      operationRef.current = "idle";
      setState((prev) => ({
        ...prev,
        operation: "idle",
        running: false,
        serverState: "stopped",
        statusMessage: msg,
        statusVariant: "error",
      }));
    }
  }, [applyHealth, clearPoll, flushConfigSave, setOperation, syncCodexProfile]);

  const stop = useCallback(async () => {
    if (blocksLaunchToggle(operationRef.current) && operationRef.current !== "waiting_for_codex") {
      return;
    }
    clearPoll();
    setOperation("stopping", "Stopping Codex...");
    try {
      const result = await window.codexRPC.stop();
      const payload = unwrap(result);
      const message = (payload as { message?: string }).message ?? "Codex stopped.";
      const healthResult = await window.codexRPC.healthCheck(configRef.current);
      const health = unwrap(healthResult);
      applyHealth(
        health.running ?? false,
        health.apiReady ?? false,
        health.endpointReady ?? false,
        true,
      );
      setState((prev) => ({
        ...prev,
        statusMessage: message,
        statusVariant: "success",
      }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      operationRef.current = "idle";
      setState((prev) => ({
        ...prev,
        operation: "idle",
        serverState: "stopped",
        running: false,
        statusMessage: msg,
        statusVariant: "error",
      }));
    }
  }, [applyHealth, clearPoll, setOperation]);

  const writeCodexConfig = useCallback(async (statusMessage = "Applying Local API settings...") => {
    if (!shouldAutoSyncCodex(configRef.current)) {
      setState((prev) => ({
        ...prev,
        statusMessage: persistCodexDisabledMessage(),
        statusVariant: "error",
      }));
      return;
    }
    setOperation("writing_codex", statusMessage);
    setState((prev) => ({ ...prev, writingCodex: true }));
    try {
      await flushConfigSave();
      const result = await window.codexRPC.writeCodexConfig(configRef.current);
      const payload = unwrap<{ message?: string; inspection?: CodexConfigInspection }>(result);
      const profile = applyInspection(payload.inspection, {
        status: "managed",
        restoreAvailable: true,
      });
      operationRef.current = "idle";
      setState((prev) => ({
        ...prev,
        operation: "idle",
        writingCodex: false,
        codexProfile: profile,
        statusMessage: providerSwitchSuccessMessage(
          payload.message ?? "Local API is now active for Codex.",
          prev.serverState === "running",
        ),
        statusVariant: "success",
      }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      operationRef.current = "idle";
      setState((prev) => ({
        ...prev,
        operation: "idle",
        writingCodex: false,
        statusMessage: msg,
        statusVariant: "error",
      }));
    }
  }, [flushConfigSave, setOperation]);

  const revertCodexConfig = useCallback(async (statusMessage = "Switching to Codex Account...") => {
    setOperation("reverting_codex", statusMessage);
    setState((prev) => ({ ...prev, revertingCodex: true }));
    try {
      await flushConfigSave();
      const result = await window.codexRPC.revertCodexConfig(configRef.current);
      const payload = unwrap<{ message?: string; inspection?: CodexConfigInspection }>(result);
      const profile = applyInspection(payload.inspection, {
        status: "external",
        restoreAvailable: Boolean(payload.inspection?.restoreStateAvailable),
      });
      operationRef.current = "idle";
      setState((prev) => ({
        ...prev,
        operation: "idle",
        revertingCodex: false,
        codexProfile: profile,
        statusMessage: providerSwitchSuccessMessage(
          payload.message ?? "Codex Account is now active.",
          prev.serverState === "running",
        ),
        statusVariant: "success",
      }));
      await inspectCodexProfile();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      operationRef.current = "idle";
      setState((prev) => ({
        ...prev,
        operation: "idle",
        revertingCodex: false,
        statusMessage: msg,
        statusVariant: "error",
      }));
    }
  }, [flushConfigSave, inspectCodexProfile, setOperation]);

  const switchProvider = useCallback(
    async (mode: ProviderMode) => {
      const block = blocksProviderSwitch(operationRef.current);
      if (!block.ok) {
        setState((prev) => ({
          ...prev,
          statusMessage: block.message,
          statusVariant: "error",
        }));
        return;
      }

      const resolved = activeProviderMode(codexProfileRef.current);

      if (mode === "local") {
        const req = localActivationRequirements(
          configRef.current,
          modelsRef.current.length,
        );
        if (!req.ok) {
          setState((prev) => ({
            ...prev,
            statusMessage: req.message,
            statusVariant: "error",
          }));
          return;
        }

        if (!shouldAutoSyncCodex(configRef.current)) {
          setState((prev) => ({
            ...prev,
            statusMessage: persistCodexDisabledMessage(),
            statusVariant: "error",
          }));
          return;
        }

        if (resolved === "local") {
          await syncCodexProfile(undefined, false, "Applying Local API settings...", true);
        } else {
          await writeCodexConfig("Switching to Local API...");
        }
        return;
      }

      if (resolved === "codex") return;
      await revertCodexConfig();
    },
    [revertCodexConfig, syncCodexProfile, writeCodexConfig],
  );

  const updateConfig = useCallback(
    <K extends keyof LauncherConfig>(key: K, value: LauncherConfig[K]) => {
      setState((prev) => {
        const next = normalizeConfig({ ...prev.config, [key]: value });
        configRef.current = next;
        return { ...prev, config: next };
      });
      if (CODEX_SYNC_KEYS.includes(key)) {
        queueCodexSync();
      }
    },
    [queueCodexSync],
  );

  const selectModel = useCallback(
    async (model: string) => {
      const trimmed = model.trim();
      if (!trimmed) return;

      setOperation("selecting_model", `Selected model: ${trimmed}`);
      const next = normalizeConfig({ ...configRef.current, codexModel: trimmed });
      configRef.current = next;
      setState((prev) => ({ ...prev, config: next }));

      try {
        await flushConfigSave();
        const onLocal = activeProviderMode(codexProfileRef.current) === "local";
        if (shouldSyncCodexToDisk(codexProfileRef.current, next)) {
          await syncCodexProfile(next, true, undefined, true);
        }
        operationRef.current = "idle";
        setState((prev) => ({
          ...prev,
          operation: "idle",
          statusMessage: onLocal
            ? `Model set to ${trimmed}.`
            : `Model saved as ${trimmed}. Switch to Local API to apply.`,
          statusVariant: "success",
        }));
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        operationRef.current = "idle";
        setState((prev) => ({
          ...prev,
          operation: "idle",
          statusMessage: msg,
          statusVariant: "error",
        }));
      }
    },
    [flushConfigSave, setOperation, syncCodexProfile],
  );

  const setAutoStart = useCallback(async (enabled: boolean) => {
    const next = normalizeConfig({ ...configRef.current, autoStart: enabled });
    configRef.current = next;
    setState((prev) => ({ ...prev, config: next }));

    try {
      const result = await window.codexRPC.setAutoStart(enabled);
      const payload = unwrap<{ message?: string; enabled?: boolean }>(result);
      setState((prev) => ({
        ...prev,
        config: normalizeConfig({ ...prev.config, autoStart: payload.enabled ?? enabled }),
        statusMessage: payload.message ?? (enabled ? "Auto-start enabled." : "Auto-start disabled."),
        statusVariant: "success",
      }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((prev) => ({
        ...prev,
        statusMessage: `Auto-start update failed: ${msg}`,
        statusVariant: "error",
      }));
    }
  }, []);

  useEffect(() => {
    const flushOnExit = () => {
      void window.codexRPC.saveConfig(configRef.current);
    };
    window.addEventListener("beforeunload", flushOnExit);
    return () => window.removeEventListener("beforeunload", flushOnExit);
  }, []);

  useEffect(() => {
    const refreshOnFocus = () => {
      if (document.visibilityState === "visible" && operationRef.current === "idle") {
        void inspectCodexProfile();
        void healthCheck(true);
      }
    };
    window.addEventListener("focus", refreshOnFocus);
    document.addEventListener("visibilitychange", refreshOnFocus);
    return () => {
      window.removeEventListener("focus", refreshOnFocus);
      document.removeEventListener("visibilitychange", refreshOnFocus);
    };
  }, [healthCheck, inspectCodexProfile]);

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
    switchProvider,
    refreshModels,
    updateConfig,
    selectModel,
    setAutoStart,
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
    codexProviderId: get("codexProviderId", "codex-launchpad"),
    codexProviderName: get("codexProviderName", "Codex Launchpad"),
    codexConfigPath: get("codexConfigPath", ""),
    codexCommand: get("codexCommand", ""),
    codexApiPort: String(config.codexApiPort ?? 4000),
    codexApiScheme: get("codexApiScheme", "http"),
    codexArgs: Array.isArray(config.codexArgs) ? config.codexArgs.join(",") : "",
    codexApiKeyMode: get("codexApiKeyMode", "experimentalBearerToken"),
  };
}