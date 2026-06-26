import type { ReactNode } from 'react';
import type { LaunchStatus, ModelInfo, CodexProcessInfo } from '../types';
import Sidebar from './Sidebar';

interface LayoutProps {
      children: ReactNode;
      darkMode: boolean;
      onToggleDarkMode: () => void;
      statusMessage: string;
      launchStatus: LaunchStatus;
      models: ModelInfo[];
      restartRequired?: boolean;
      codexInfo?: CodexProcessInfo;
}

const getStatusType = (msg: string): 'success' | 'error' | 'info' => {
      if (!msg) return 'info';
      if (msg.includes('fail') || msg.includes('error') || msg.includes('Failed')) return 'error';
      return 'success';
};

export default function Layout({ children, darkMode, onToggleDarkMode, statusMessage, launchStatus, models, restartRequired, codexInfo }: LayoutProps) {
      return (
                      <>
                          <header className="header">
                              <h1>Codex Launcher</h1>
                              <div className="header-controls">
                                  <label className="theme-toggle" aria-label="Toggle dark mode">
                                      <input
                                      type="checkbox"
                                      checked={darkMode}
                                      onChange={onToggleDarkMode}
                                       />
                                       <span className="theme-toggle-track" />
                                  </label>
                                  <span className="theme-label">{darkMode ? 'Dark' : 'Light'}</span>
                              </div>
                          </header>
                          <div className="layout-container">
                              <Sidebar
                             launchStatus={launchStatus}
                             models={models}
                             statusMessage={statusMessage}
                             activeNav="launcher"
                             onNavClick={() => {}}
                             onRefreshModels={() => {}}
                             onToggleDarkMode={onToggleDarkMode}
                             darkMode={darkMode}
                              />
                              <main className="main-content">{children}</main>
                          </div>
                          <footer className={'status-bar ' + getStatusType(statusMessage)}>
                              {statusMessage}
                          </footer>
                      </>
                  );
}