import type { ReactNode } from 'react';

type StatusType = 'success' | 'error' | 'info';

interface StatusLineProps {
    message: string;
    type?: StatusType;
}

const getStatusType = (msg: string, type?: StatusType): StatusType => {
    if (type) return type;
    if (!msg) return 'info';
    if (msg.includes('fail') || msg.includes('error') || msg.includes('Failed')) return 'error';
    return 'success';
};

export default function StatusLine({ message, type }: StatusLineProps) {
    if (!message) return null;
    const t = getStatusType(message, type);
    return <div className={`status-bar ${t}`}>{message}</div>;
}
