import { Globe, ChevronDown, RefreshCw } from "lucide-react";
import { Card, FormField, TextInput } from "./primitives";

export function EndpointCard({
  ip,
  port,
  scheme,
  onChange,
  onRefresh,
  refreshing,
}: {
  ip: string;
  port: string;
  scheme: "http" | "https";
  onChange: (patch: { ip?: string; port?: string; scheme?: "http" | "https" }) => void;
  onRefresh: () => void;
  refreshing?: boolean;
}) {
  const baseUrl = ip || port ? `${scheme}://${ip || "—"}:${port || "—"}/v1` : "—";

  return (
    <Card icon={<Globe className="h-4 w-4" />} title="Endpoint Configuration">
      <div className="grid grid-cols-2 gap-x-4 gap-y-[14px]">
        <FormField label="IP Address">
          <TextInput
            data-testid="endpoint-ip"
            value={ip}
            onChange={(e) => onChange({ ip: e.target.value })}
            placeholder="127.0.0.1"
          />
        </FormField>
        <FormField label="Port">
          <TextInput
            data-testid="endpoint-port"
            value={port}
            onChange={(e) => onChange({ port: e.target.value })}
            placeholder="11434"
          />
        </FormField>
        <FormField label="Scheme">
          <div className="relative">
            <select
              data-testid="endpoint-scheme"
              value={scheme}
              onChange={(e) => onChange({ scheme: e.target.value as "http" | "https" })}
              className="h-[38px] w-full appearance-none rounded-md border border-input bg-background px-3 pr-9 text-sm text-foreground focus:border-primary focus:outline-none focus:ring-4 focus:ring-primary/15"
            >
              <option value="http">http</option>
              <option value="https">https</option>
            </select>
            <ChevronDown className="pointer-events-none absolute right-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          </div>
        </FormField>
        <FormField label="Base URL" hint="Generated from IP, port, and scheme">
          <TextInput
            data-testid="endpoint-base-url"
            value={baseUrl}
            readOnly
            className="cursor-not-allowed bg-muted/50 font-mono text-[13px] text-muted-foreground"
          />
        </FormField>
      </div>

      <button
        type="button"
        data-testid="refresh-endpoint-models"
        onClick={onRefresh}
        disabled={refreshing}
        className="mt-4 inline-flex h-[38px] w-full items-center justify-center gap-2 rounded-md border border-input bg-background px-4 text-[13px] font-semibold text-foreground transition-colors hover:bg-muted/70 disabled:cursor-not-allowed disabled:opacity-60"
      >
        <RefreshCw className={`h-4 w-4 ${refreshing ? "animate-spin" : ""}`} />
        {refreshing ? "Refreshing models..." : "Refresh Models"}
      </button>
    </Card>
  );
}