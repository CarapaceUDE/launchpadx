export interface LauncherConfig {
    autoStart?: boolean;
    openaiBaseUrl?: string;
    ollamaIp?: string;
    ollamaPort?: number;
    ollamaScheme?: string;
    apiKey?: string;
    persistCodexConfig?: boolean;
    codexModel?: string;
    codexProviderId?: string;
    codexProviderName?: string;
    codexApiPort?: number;
    codexApiScheme?: string;
    codexArgs?: string[];
    discoverOllamaModels?: boolean;
    codexConfigPath?: string;
    codexCommand?: string;
    workingDirectory?: string;
    codexApiKeyMode?: "envKey" | "experimentalBearerToken" | "none";
}

export interface HealthState {
    running: boolean;
    apiReady: boolean;
    endpointReady?: boolean;
    pid?: number | null;
    method?: string | null;
    error?: string;
}

export interface ModelInfo {
    name: string;
    size: number;
    digest: string;
    modified: string;
    fetchedFrom?: string;
}

export interface ModelCache {
    models: ModelInfo[];
    fetchedFrom: string;
}

export type LaunchStatus = 'idle' | 'launching' | 'running' | 'stopping' | 'stopped' | 'error';

export interface CodexProcessInfo {
    running: boolean;
    pid: number | null;
    method: string | null;
    restartRequired: boolean;
}
