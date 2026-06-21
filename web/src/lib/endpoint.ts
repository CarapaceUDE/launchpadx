import type { LauncherConfig } from "../types";

export function buildOpenAiBaseUrl(ip: string, port: number, scheme: string): string {
  const trimmed = ip.trim();
  if (!trimmed) return "";

  const host =
    trimmed.includes(":") && !trimmed.startsWith("[") ? `[${trimmed}]` : trimmed;
  return `${scheme}://${host}:${port}/v1`;
}

export function normalizeConfig(config: LauncherConfig): LauncherConfig {
  const ip = config.ollamaIp?.trim() ?? "";
  const port = config.ollamaPort ?? 11434;
  const scheme = config.ollamaScheme ?? "http";
  const derivedBaseUrl = ip ? buildOpenAiBaseUrl(ip, port, scheme) : undefined;

  return {
    ...config,
    ollamaPort: port,
    ollamaScheme: scheme,
    openaiBaseUrl: derivedBaseUrl ?? config.openaiBaseUrl,
  };
}