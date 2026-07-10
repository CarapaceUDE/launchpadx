import { TARGET_APP_NAME } from "./branding";

export type LauncherOperation =
  | "idle"
  | "initializing"
  | "saving"
  | "launching"
  | "stopping"
  | "waiting_for_codex"
  | "refreshing_models"
  | "syncing_codex"
  | "writing_codex"
  | "reverting_codex"
  | "selecting_model"
  | "failover_switching";

export function isBusyOperation(op: LauncherOperation): boolean {
  return op !== "idle";
}

export function operationLabel(op: LauncherOperation): string {
  switch (op) {
    case "initializing":
      return "Initializing";
    case "saving":
      return "Saving";
    case "launching":
      return "Starting";
    case "waiting_for_codex":
      return "Starting";
    case "stopping":
      return "Stopping";
    case "refreshing_models":
      return "Refreshing";
    case "syncing_codex":
      return "Syncing";
    case "writing_codex":
      return "Switching";
    case "reverting_codex":
      return "Switching";
    case "selecting_model":
      return "Updating model";
    case "failover_switching":
      return "Failover";
    default:
      return "";
  }
}

export function healthStatusMessage(
  running: boolean,
  apiReady: boolean,
  endpointReady: boolean,
  providerMode: "codex" | "local" = "local",
): string {
  if (running) {
    return apiReady
      ? `${TARGET_APP_NAME} is running.`
      : `${TARGET_APP_NAME} started — API is warming up...`;
  }
  if (providerMode === "codex") {
    return `${TARGET_APP_NAME} is stopped.`;
  }
  if (endpointReady) {
    return `${TARGET_APP_NAME} is stopped. Local API endpoint is reachable.`;
  }
  return `${TARGET_APP_NAME} is stopped. Local API endpoint is not reachable — check IP/port.`;
}

export function blocksLaunchToggle(op: LauncherOperation): boolean {
  return (
    op === "initializing" ||
    op === "launching" ||
    op === "stopping" ||
    op === "waiting_for_codex" ||
    op === "writing_codex" ||
    op === "reverting_codex" ||
    op === "failover_switching"
  );
}

export function shouldPreserveOperationStatus(op: LauncherOperation): boolean {
  return (
    op === "launching" ||
    op === "waiting_for_codex" ||
    op === "stopping" ||
    op === "writing_codex" ||
    op === "reverting_codex" ||
    op === "selecting_model" ||
    op === "initializing" ||
    op === "failover_switching"
  );
}