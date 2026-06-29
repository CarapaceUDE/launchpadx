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

export type ConnectionAlertKind =
    | "endpointDown"
    | "endpointRestored"
    | "codexApiDown"
    | "codexApiRestored"
    | "sessionConnectionError";

export type AlertSeverity = "error" | "warn" | "info";

export interface EndpointHealth {
    checkedAt: string;
    endpointUrl?: string | null;
    reachable: boolean;
    statusCode?: number | null;
    latencyMs?: number | null;
    error?: string | null;
    modelCount?: number | null;
}

export interface ConnectionAlert {
    detectedAt: string;
    kind: ConnectionAlertKind;
    severity: AlertSeverity;
    title: string;
    message: string;
    endpointHealth?: EndpointHealth | null;
    dismissed: boolean;
}

export interface FailoverStatus {
    watching: boolean;
    autoSwitch: boolean;
    lastPollAt?: string | null;
    lastError?: string | null;
    activeAlert?: FailoverAlert | null;
    recentAlerts: FailoverAlert[];
    lastCheckpoint?: SessionCheckpoint | null;
    discoveryLogHint?: string;
    activeConnectionAlert?: ConnectionAlert | null;
    recentConnectionAlerts?: ConnectionAlert[];
    endpointHealth?: EndpointHealth | null;
    codexApiReady?: boolean;
    endpointReachable?: boolean;
    connectionLogHint?: string;
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

export interface RateLimitWindow {
    usedPercent?: number | null;
    windowDurationMins?: number | null;
    resetsAt?: number | null;
}

export interface RateLimitCredits {
    hasCredits?: boolean | null;
    unlimited?: boolean | null;
    balance?: string | null;
}

export interface CodexRateLimits {
    limitId?: string | null;
    limitName?: string | null;
    primary?: RateLimitWindow | null;
    secondary?: RateLimitWindow | null;
    credits?: RateLimitCredits | null;
    planType?: string | null;
    rateLimitReachedType?: string | null;
}

export interface RateLimitResetCredits {
    availableCount?: number | null;
}

export interface CodexThreadSummary {
    id: string;
    name?: string | null;
    status?: string | null;
    path?: string | null;
    createdAt?: string | null;
    model?: string | null;
}

export interface CodexThreadListStatus {
    ok: boolean;
    fetchedAt: string;
    source: string;
    codexCli?: string | null;
    error?: string | null;
    threads: CodexThreadSummary[];
}

export interface SessionPreviewContent {
    role: string;
    done: boolean;
    content: string;
}

export interface CodexSessionDetail {
    sessionId: string;
    createdAt?: string | null;
    preview?: SessionPreviewContent | null;
    previewError?: string | null;
}

export interface CodexSessionListDetail {
    sessions: CodexSessionDetail[];
    error?: string | null;
}

export interface DiscoveryLogEntry {
    stream: string;
    source: string;
    at: string;
    event: string;
    details: Record<string, unknown>;
}

export interface CodexRateLimitsStatus {
    ok: boolean;
    fetchedAt: string;
    source: string;
    codexCli?: string | null;
    error?: string | null;
    requiresAuth?: boolean | null;
    planType?: string | null;
    rateLimits?: CodexRateLimits | null;
    rateLimitResetCredits?: RateLimitResetCredits | null;
}
