interface StatusPillProps {
    status: "running" | "stopped" | "launching";
}

export default function StatusPill({ status }: StatusPillProps) {
    const label = status === "running" ? "Running" : status === "launching" ? "Starting..." : "Stopped";
    return <span className={`status-pill ${status}`}>{label}</span>;
}
