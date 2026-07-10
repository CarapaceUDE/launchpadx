import type { LauncherConfig } from "../types";

export function buildOpenAiBaseUrl(ip: string, port: number, scheme: string): string {
  const trimmed = ip.trim();
  if (!trimmed) return "";

  const host =
    trimmed.includes(":") && !trimmed.startsWith("[") ? `[${trimmed}]` : trimmed;
  return `${scheme}://${host}:${port}/v1`;
}

type LegacyLauncherConfig = LauncherConfig & {
  lpadModel?: string;
  lpadPersistConfig?: boolean;
  lpadProviderId?: string;
  lpadProviderName?: string;
  lpadApiKeyMode?: LauncherConfig["codexApiKeyMode"];
  lpadApiPort?: number;
  lpadApiScheme?: string;
  lpadArgs?: string[];
};

export function normalizeConfig(config: LauncherConfig): LauncherConfig {
  const raw = config as LegacyLauncherConfig;
  const ip = config.ollamaIp?.trim() ?? "";
  const port = config.ollamaPort ?? 11434;
  const scheme = config.ollamaScheme ?? "http";
  const derivedBaseUrl = ip ? buildOpenAiBaseUrl(ip, port, scheme) : undefined;

  return {
    ...config,
    persistCodexConfig: config.persistCodexConfig ?? raw.lpadPersistConfig,
    codexModel: config.codexModel ?? raw.lpadModel,
    codexProviderId: config.codexProviderId ?? raw.lpadProviderId,
    codexProviderName: config.codexProviderName ?? raw.lpadProviderName,
    codexApiKeyMode: config.codexApiKeyMode ?? raw.lpadApiKeyMode,
    codexApiPort: config.codexApiPort ?? raw.lpadApiPort,
    codexApiScheme: config.codexApiScheme ?? raw.lpadApiScheme,
    codexArgs: config.codexArgs ?? raw.lpadArgs,
    ollamaPort: port,
    ollamaScheme: scheme,
    openaiBaseUrl: derivedBaseUrl ?? config.openaiBaseUrl,
  };
}