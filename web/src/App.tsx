import { useState, useCallback, useEffect, useRef, useLayoutEffect } from 'react';
import type { LauncherConfig, LaunchStatus, ModelInfo, CodexProcessInfo } from './types';
import Layout from './components/Layout';
import LaunchPanel from './components/LaunchPanel';
import ConfigForm from './components/ConfigForm';

interface HealthInfo {
    running: boolean;
    apiReady: boolean;
    endpoint: string;
    error?: string | null;
}

interface AppConfigState {
    loaded: boolean;
    config: LauncherConfig | null;
    error: string | null;
}

function App() {
    const [darkMode, setDarkMode] = useState(() => {
        const stored = localStorage.getItem('darkMode');
        if (stored !== null) return stored === 'true';
        return window.matchMedia('(prefers-color-scheme: dark)').matches;
     });
    const [configState, setConfigState] = useState<AppConfigState>({
        loaded: false,
        config: null,
        error: null,
       });
    const [launchStatus, setLaunchStatus] = useState<LaunchStatus>('idle');
    const [statusMessage, setStatusMessage] = useState('');
    const [models, setModels] = useState<ModelInfo[]>([]);
    const [modelsError, setModelsError] = useState<string | null>(null);
    const [healthInfo, setHealthInfo] = useState<HealthInfo | null>(null);
    const [healthError, setHealthError] = useState<string | null>(null);
    const [loading, setLoading] = useState(true);
    const [lastChecked, setLastChecked] = useState<string | null>(null);
    const [codexInfo, setCodexInfo] = useState<CodexProcessInfo | null>(null);
    const [restartRequired, setRestartRequired] = useState(false);
    const launchStatusRef = useRef<LaunchStatus>('idle');
    const configRef = useRef<LauncherConfig | null>(null);
    const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

      // Apply dark mode immediately on mount, before paint (useLayoutEffect)
    useLayoutEffect(() => {
        document.documentElement.classList.toggle('dark', darkMode);
        localStorage.setItem('darkMode', String(darkMode));
      }, [darkMode]);

      // Listen for system theme changes
    useEffect(() => {
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        const handler = (e: MediaQueryListEvent) => {
            setDarkMode(e.matches);
         };
        mediaQuery.addEventListener('change', handler);
        return () => mediaQuery.removeEventListener('change', handler);
      }, []);

       // Auto-save config whenever it changes (debounced by 500ms)
    useEffect(() => {
        if (!configState.loaded || !configState.config) return;
        configRef.current = configState.config;

        if (saveTimeoutRef.current) {
            clearTimeout(saveTimeoutRef.current);
          }

        saveTimeoutRef.current = setTimeout(async () => {
            try {
                await window.codexRPC.saveConfig(configRef.current!);
               } catch {
                   // Silently fail - auto-save shouldn't disrupt UX
               }
           }, 500);

        return () => {
            if (saveTimeoutRef.current) {
                clearTimeout(saveTimeoutRef.current);
               }
           };
       }, [configState.config]);

    useEffect(() => {
        let cancelled = false;
        const detect = async () => {
                // Small delay to ensure RPC server is fully ready
                // Wait for config to load before making RPC calls
            if (!configState.loaded) return;
            await new Promise(r => setTimeout(r, 1000));
            if (cancelled) return;
            for (let attempt = 0; attempt < 5; attempt++) {
                if (cancelled) break;
                try {
                    const result = await window.codexRPC.detectCodex();
                    if (cancelled) return;
                    const info: CodexProcessInfo = result.data || result;
                    setCodexInfo(info);
                    setRestartRequired(info.restartRequired || false);
                    if (info.running) {
                        setLaunchStatus('running');
                        launchStatusRef.current = 'running';
                        setStatusMessage('Codex detected via ' + (info.method || 'unknown'));
                       } else {
                        setLaunchStatus('stopped');
                        launchStatusRef.current = 'stopped';
                       }
                    return;
                   } catch {
                    await new Promise(r => setTimeout(r, 500));
                   }
               }
            if (!cancelled) {
                setLaunchStatus('stopped');
                launchStatusRef.current = 'stopped';
               }
           };
        detect();
        return () => { cancelled = true; };
       }, [configState.loaded]);

        // Load config on mount; after it loads we kick off health check + model load
    useEffect(() => {
        const configPromise = window.codexRPC.loadConfig();
        configPromise.then((result) => {
            setLoading(false);
            setConfigState({ loaded: true, config: result.data || result, error: null });
           }).catch((e: unknown) => {
            setLoading(false);
            const msg = e instanceof Error ? e.message : String(e);
            setConfigState({ loaded: true, config: null, error: msg });
           });
       }, []);

    const handleLaunch = async () => {
        try {
            const result = await window.codexRPC.launch();
            setLaunchStatus('running');
            launchStatusRef.current = 'running';
            setStatusMessage(result.data?.message || 'Codex launched');
           } catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            setLaunchStatus('stopped');
            launchStatusRef.current = 'stopped';
            setStatusMessage('Failed to launch: ' + msg);
           }
       };

    const handleStop = async () => {
        try {
            const result = await window.codexRPC.stop();
            setLaunchStatus('stopped');
            launchStatusRef.current = 'stopped';
            setStatusMessage(result.data?.message || 'Codex stopped');
           } catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            setStatusMessage('Failed to stop: ' + msg);
           }
       };

    const handleHealthCheck = async () => {
        try {
            const result = await window.codexRPC.healthCheck();
            const health = result.data || result;
            setHealthInfo(health);
            setHealthError(null);
            setLastChecked(new Date().toLocaleTimeString());
           } catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            setHealthError(msg);
            setStatusMessage('Health check failed: ' + msg);
           }
       };

    const handleWriteCodexConfig = async () => {
        try {
            const result = await window.codexRPC.writeCodexConfig();
            setStatusMessage(result.data?.message || 'Config written');
           } catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            setStatusMessage('Failed to write config: ' + msg);
           }
       };

    const handleRevertCodexConfig = async () => {
        try {
            const result = await window.codexRPC.revertCodexConfig();
            setStatusMessage(result.data?.message || 'Config reverted');
           } catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            setStatusMessage('Failed to revert config: ' + msg);
           }
       };

    const handleKillCodexByPid = async () => {
        if (!codexInfo?.pid) return;
        try {
            const result = await window.codexRPC.killCodexByPid(codexInfo.pid);
            setCodexInfo(prev => prev ? { ...prev, running: false } : null);
            setRestartRequired(false);
            setStatusMessage(result.data?.message || 'Codex process killed');
           } catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            setStatusMessage('Failed to kill: ' + msg);
           }
       };

    const handleToggleAutoStart = async () => {
        try {
            const result = await window.codexRPC.toggleAutoStart();
            setStatusMessage(result.data?.message || 'Auto-start toggled');
           } catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            setStatusMessage('Failed to toggle auto-start: ' + msg);
           }
       };

    const handleRefreshModels = async () => {
        try {
            const result = await window.codexRPC.refreshModels();
            const modelsList = result.data?.models || [];
            setModels(modelsList);
            setModelsError(null);
            setStatusMessage('Model cache refreshed');
           } catch (e) {
            const msg = e instanceof Error ? e.message : String(e);
            setModelsError(msg);
            setStatusMessage(msg);
           }
       };

    const updateConfig = <K extends keyof LauncherConfig>(key: K, value: LauncherConfig[K]) => {
        setConfigState(prev => ({
               ...prev,
            config: prev.config ? { ...prev.config, [key]: value } : null,
           }));
       };

    const getConfig = () => configState.config || {};
    const config = getConfig();
    const isLoading = loading || !configState.loaded;

    if (isLoading) {
        return (
               <div style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                height: '100vh',
                background: 'var(--bg-primary)',
                color: 'var(--text-secondary)',
               }}>
                   {loading ? 'Loading...' : 'Failed to load config'}
                   {!loading && configState.error && (
                       <div style={{ marginTop: 12, color: 'var(--warning-text)', fontSize: 12 }}>
                           {configState.error}
                       </div>
                   )}
               </div>
           );
       }

    return (
               <Layout
                darkMode={darkMode}
                onToggleDarkMode={() => setDarkMode((p) => !p)}
                statusMessage={statusMessage}
                launchStatus={launchStatus}
                models={models}
                restartRequired={restartRequired}
                codexInfo={codexInfo || undefined}
               >
                   {restartRequired && (
                       <div style={{
                        background: 'var(--warning-bg)',
                        color: 'var(--warning-text)',
                        padding: '10px 16px',
                        margin: '0 16px 8px',
                        borderRadius: 'var(--radius)',
                        fontSize: 12,
                        display: 'flex',
                        alignItems: 'center',
                        gap: 12,
                       }}>
                           <span>{'\u26A0'} Restart required</span>
                           <span>Codex is currently running. Changes will take effect after you stop and restart the server.</span>
                           {codexInfo?.pid && (
                               <button
                                onClick={handleKillCodexByPid}
                                style={{
                                    background: 'var(--warning-text)',
                                    color: 'var(--bg-primary)',
                                    border: 'none',
                                    padding: '4px 12px',
                                    borderRadius: 'var(--radius)',
                                    cursor: 'pointer',
                                    fontWeight: 'bold',
                                    fontSize: 11,
                                }}
                               >
                                Kill Process ({codexInfo.pid})
                               </button>
                           )}
                       </div>
                   )}
                   <LaunchPanel
                    launchStatus={launchStatus}
                    onLaunch={handleLaunch}
                    onStop={handleStop}
                    onRefreshModels={handleRefreshModels}
                    models={models}
                    healthError={healthError}
                    modelsError={modelsError}
                   />
                   <ConfigForm
                    config={config}
                    onUpdate={updateConfig}
                    onWriteCodexConfig={handleWriteCodexConfig}
                    onRevertCodexConfig={handleRevertCodexConfig}
                    onRefreshModels={handleRefreshModels}
                    codexInfo={codexInfo || undefined}
                    onKillCodex={handleKillCodexByPid}
                    onToggleAutoStart={handleToggleAutoStart}
                    models={models}
                   />
               </Layout>
           );
}


export default App;