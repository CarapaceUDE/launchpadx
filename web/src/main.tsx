import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { installCodexRpcClient } from './lib/lpadRpc';
import './styles.css';

installCodexRpcClient();

ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
        <App />
    </React.StrictMode>
);