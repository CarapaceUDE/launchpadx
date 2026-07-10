import type { LauncherConfig } from "../types";
import {
  activeProviderMode,
  shouldAutoSyncCodex,
  type CodexProfileState,
  type ProviderMode,
} from "./lpadProfile";
import type { LauncherOperation } from "./operationStatus";
import { APP_NAME, TARGET_APP_NAME } from "./branding";

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
  "failover_switching",
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
  return `Persistent profile sync is disabled. Enable it in Settings → Advanced to apply provider changes.`;
}

export function codexAccountSwitchWarnings(
  profile: CodexProfileState,
  codexRunning: boolean,
): string[] {
  const warnings: string[] = [];

  if (profile.status === "managed" && !profile.restoreAvailable) {
    warnings.push(
      `No restore snapshot was found. ${APP_NAME} will still remove local provider settings and switch back to your cloud account when possible.`,
    );
  }
  if (profile.status === "missing") {
    warnings.push("Profile config file is missing. Switching will start from a clean profile.");
  }
  if (codexRunning) {
    warnings.push(
      `${TARGET_APP_NAME} is running — restart it after switching for the change to take effect.`,
    );
  }

  return warnings;
}

export function localSwitchWarnings(codexRunning: boolean): string[] {
  return codexRunning
    ? [
        `${TARGET_APP_NAME} is running — restart it after switching for the new model provider to apply.`,
      ]
    : [];
}

export function providerSwitchSuccessMessage(base: string, codexRunning: boolean): string {
  if (!codexRunning) return base;
  return `${base} Restart ${TARGET_APP_NAME} for the change to take effect.`;
}