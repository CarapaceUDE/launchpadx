import type { LauncherConfig } from "../types";

export type CodexProfileStatus = "unknown" | "missing" | "external" | "managed";

/** Which provider Codex is configured to use. */
export type ProviderMode = "codex" | "local";

export interface CodexProfileState {
  status: CodexProfileStatus;
  model?: string;
  modelProvider?: string;
  baseUrl?: string;
  restoreAvailable: boolean;
  configPath?: string;
}

export interface CodexConfigInspection {
  configPath?: string;
  exists?: boolean;
  model?: string | null;
  modelProvider?: string | null;
  managedByLauncher?: boolean;
  launcherBaseUrl?: string | null;
  restoreStateAvailable?: boolean;
}

export function inspectionToProfile(inspection: CodexConfigInspection): CodexProfileState {
  if (!inspection.exists) {
    return {
      status: "missing",
      restoreAvailable: Boolean(inspection.restoreStateAvailable),
      configPath: inspection.configPath,
    };
  }

  if (inspection.managedByLauncher) {
    return {
      status: "managed",
      model: inspection.model ?? undefined,
      modelProvider: inspection.modelProvider ?? undefined,
      baseUrl: inspection.launcherBaseUrl ?? undefined,
      restoreAvailable: Boolean(inspection.restoreStateAvailable),
      configPath: inspection.configPath,
    };
  }

  return {
    status: "external",
    model: inspection.model ?? undefined,
    modelProvider: inspection.modelProvider ?? undefined,
    restoreAvailable: Boolean(inspection.restoreStateAvailable),
    configPath: inspection.configPath,
  };
}

export function activeProviderMode(profile: CodexProfileState): ProviderMode {
  return profile.status === "managed" ? "local" : "codex";
}

export function providerModeLabel(mode: ProviderMode): string {
  return mode === "local" ? "Local API" : "Codex Account";
}

export function providerModeDescription(mode: ProviderMode): string {
  return mode === "local"
    ? "Route Codex through your Ollama or OpenAI-compatible endpoint and chosen model."
    : "Use Codex sign-in and your default Codex cloud provider settings.";
}

export function activeProviderSummary(
  mode: ProviderMode,
  profile: CodexProfileState,
  model?: string,
  endpoint?: string,
): string {
  if (mode === "local") {
    const parts = [model, endpoint].filter(Boolean);
    return parts.length > 0
      ? `Codex is using Local API · ${parts.join(" · ")}`
      : "Codex is using Local API";
  }

  if (profile.modelProvider) {
    return `Codex is using ${profile.modelProvider}${profile.model ? ` · ${profile.model}` : ""}`;
  }

  return "Codex is using your account sign-in";
}

export function profileStatusLabel(status: CodexProfileStatus): string {
  switch (status) {
    case "managed":
      return "Local API active";
    case "external":
      return "Codex account active";
    case "missing":
      return "Codex config not found";
    default:
      return "Checking provider...";
  }
}

export function shouldAutoSyncCodex(config: LauncherConfig): boolean {
  return config.persistCodexConfig !== false;
}