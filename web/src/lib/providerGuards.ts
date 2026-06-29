import type { LauncherConfig } from "../types";
import {
  activeProviderMode,
  shouldAutoSyncCodex,
  type CodexProfileState,
  type ProviderMode,
} from "./codexProfile";
import type { LauncherOperation } from "./operationStatus";

export type GuardResult = { ok: true } | { ok: false; message: string };

const PROVIDER_SWITCH_BLOCKERS: LauncherOperation[] = [
  "initializing",
  "launching",
  "stopping",
  "waiting_for_codex",
  "writing_codex",
  "reverting_codex",
  "syncing_codex",
  "refreshing_models",
  "selecting_model",
];

export function blocksProviderSwitch(operation: LauncherOperation): GuardResult {
  if (PROVIDER_SWITCH_BLOCKERS.includes(operation)) {
    return {
      ok: false,
      message: "Wait for the current operation to finish before switching provider.",
    };
  }
  return { ok: true };
}

export function localActivationRequirements(
  config: LauncherConfig,
  modelCount: number,
): GuardResult {
  if (!config.ollamaIp?.trim()) {
    return { ok: false, message: "Open Local API settings and set your endpoint." };
  }
  if (!config.codexModel?.trim()) {
    if (modelCount === 0) {
      return { ok: false, message: "Refresh models, then pick one in the Local API card." };
    }
    return { ok: false, message: "Select a model in the Local API card." };
  }
  return { ok: true };
}

export function canStartCodex(
  providerMode: ProviderMode,
  config: LauncherConfig,
  modelCount: number,
): { canStart: boolean; reason?: string } {
  if (providerMode === "codex") {
    return { canStart: true };
  }
  const req = localActivationRequirements(config, modelCount);
  return req.ok ? { canStart: true } : { canStart: false, reason: req.message };
}

export function shouldSyncCodexToDisk(
  profile: CodexProfileState,
  config: LauncherConfig,
): boolean {
  return shouldAutoSyncCodex(config) && activeProviderMode(profile) === "local";
}

export function persistCodexDisabledMessage(): string {
  return "Persistent Codex config is disabled. Enable it in Settings → Advanced to apply provider changes.";
}

export function codexAccountSwitchWarnings(
  profile: CodexProfileState,
  codexRunning: boolean,
): string[] {
  const warnings: string[] = [];

  if (profile.status === "managed" && !profile.restoreAvailable) {
    warnings.push(
      "No restore snapshot was found. The launcher will still remove local provider settings and switch Codex back to your account provider when possible.",
    );
  }
  if (profile.status === "missing") {
    warnings.push("Codex config file is missing. Switching will start from a clean profile.");
  }
  if (codexRunning) {
    warnings.push("Codex is running — restart it after switching for the change to take effect.");
  }

  return warnings;
}

export function localSwitchWarnings(codexRunning: boolean): string[] {
  return codexRunning
    ? ["Codex is running — restart it after switching for the new model provider to apply."]
    : [];
}

export function providerSwitchSuccessMessage(base: string, codexRunning: boolean): string {
  if (!codexRunning) return base;
  return `${base} Restart Codex for the change to take effect.`;
}