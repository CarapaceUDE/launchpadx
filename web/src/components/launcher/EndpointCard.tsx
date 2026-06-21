import { Globe, ChevronDown } from "lucide-react";
import { Card, FormField, TextInput } from "./primitives";

export function EndpointCard({
  ip,
  port,
  scheme,
  onChange,
}: {
  ip: string;
  port: string;
  scheme: "http" | "https";
  onChange: (patch: { ip?: string; port?: string; scheme?: "http" | "https" }) => void;
}) {
  const baseUrl = ip || port ? `${scheme}://${ip || "—"}:${port || "—"}` : "—";

  return (
    <Card icon={<Globe className="h-4 w-4" />} title="Endpoint Configuration">
      <div className="grid grid-cols-2 gap-x-4 gap-y-[14px]">
        <FormField label="IP Address">
          <TextInput
            value={ip}
            onChange={(e) => onChange({ ip: e.target.value })}
            placeholder="127.0.0.1"
          />
        </FormField>
        <FormField label="Port">
          <TextInput
            value={port}
            onChange={(e) => onChange({ port: e.target.value })}
            placeholder="11434"
          />
        </FormField>
        <FormField label="Scheme">
          <div className="relative">
            <select
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
            value={baseUrl}
            readOnly
            className="cursor-not-allowed bg-muted/50 font-mono text-[13px] text-muted-foreground"
          />
        </FormField>
      </div>
    </Card>
  );
}