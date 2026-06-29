export interface FailoverSettings {
    enabled?: boolean;
    autoSwitch?: boolean;
    monitorIntervalSecs?: number;
    rateLimitPatterns?: string[];
    fallbackChain?: string[];
}

export interface ProfileOverlay {
    openaiBaseUrl?: string;
    ollamaIp?: string;
    ollamaPort?: number;
    ollamaScheme?: string;
    apiKey?: string;
    codexModel?: string;
    codexProviderId?: string;
    codexProviderName?: string;
    codexApiKeyMode?: "envKey" | "experimentalBearerToken" | "none";
    workingDirectory?: string;
}

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
    failover?: FailoverSettings;
    profiles?: Record<string, ProfileOverlay>;
}

export interface FailoverAlert {
    detectedAt: string;
    matchedPattern: string;
    source: string;
    sessionId?: string | null;
    snippet: string;
    dismissed: boolean;
}

export interface SessionCheckpoint {
    id: string;
    capturedAt: string;
    threadId?: string | null;
    sessionId?: string | null;
    workingDirectory?: string | null;
    providerMode: "codexAccount" | "localApi";
    model?: string | null;
    activeGoal?: string | null;
    lastUserMessage?: string | null;
    lastAssistantSummary?: string | null;
    gitBranch?: string | null;
    trigger: string;
    resumePrompt: string;
}

export interface FailoverStatus {
    enabled: boolean;
    autoSwitch: boolean;
    monitoring: boolean;
    lastPollAt?: string | null;
    lastError?: string | null;
    activeAlert?: FailoverAlert | null;
    recentAlerts: FailoverAlert[];
    lastCheckpoint?: SessionCheckpoint | null;
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
