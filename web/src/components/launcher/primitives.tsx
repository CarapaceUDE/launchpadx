import {
  type ComponentPropsWithoutRef,
  type ReactNode,
  type InputHTMLAttributes,
  forwardRef,
} from "react";

export function Card({
  icon,
  title,
  children,
  className = "",
  ...props
}: {
  icon?: ReactNode;
  title?: ReactNode;
  children: ReactNode;
  className?: string;
} & Omit<ComponentPropsWithoutRef<"section">, "children" | "className">) {
  return (
    <section className={`card-surface p-6 ${className}`} {...props}>
      {title && (
        <header className="mb-5 flex items-center gap-2.5">
          {icon && (
            <span className="grid h-8 w-8 place-items-center rounded-lg bg-primary/10 text-primary">
              {icon}
            </span>
          )}
          <h2 className="text-[15px] font-semibold tracking-tight text-foreground">{title}</h2>
        </header>
      )}
      {children}
    </section>
  );
}

export function FormField({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <label className="block">
      <div className="mb-1.5 text-[12px] font-medium text-foreground/80">{label}</div>
      {children}
      {hint && <p className="mt-1.5 text-[12px] text-muted-foreground">{hint}</p>}
    </label>
  );
}

export const themedNativeSelectClass =
  "themed-native-select appearance-none rounded-md border border-input bg-background text-foreground focus:border-primary focus:outline-none focus:ring-4 focus:ring-primary/15 disabled:cursor-not-allowed disabled:bg-muted/60 disabled:text-muted-foreground";

export const TextInput = forwardRef<HTMLInputElement, InputHTMLAttributes<HTMLInputElement>>(
  function TextInput({ className = "", ...props }, ref) {
    return (
      <input
        ref={ref}
        className={`h-[38px] w-full rounded-md border border-input bg-background px-3 text-sm text-foreground placeholder:text-muted-foreground/70 transition-all focus:border-primary focus:outline-none focus:ring-4 focus:ring-primary/15 disabled:bg-muted disabled:text-muted-foreground ${className}`}
        {...props}
      />
    );
  },
);

export type ServerPillState = "stopped" | "starting" | "running" | "stopping";

export function StatusPill({ state }: { state: ServerPillState }) {
  if (state === "running") {
    return (
      <span
        data-testid="status-pill"
        data-state={state}
        className="inline-flex items-center gap-1.5 rounded-full bg-success/15 px-2.5 py-1 text-[12px] font-semibold text-success"
      >
        <span className="h-1.5 w-1.5 rounded-full bg-success" />
        Running
      </span>
    );
  }

  if (state === "starting") {
    return (
      <span
        data-testid="status-pill"
        data-state={state}
        className="inline-flex items-center gap-1.5 rounded-full bg-primary/15 px-2.5 py-1 text-[12px] font-semibold text-primary"
      >
        <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-primary" />
        Starting
      </span>
    );
  }

  if (state === "stopping") {
    return (
      <span
        data-testid="status-pill"
        data-state={state}
        className="inline-flex items-center gap-1.5 rounded-full bg-warning-bg/20 px-2.5 py-1 text-[12px] font-semibold text-warning-fg"
      >
        <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-warning-fg" />
        Stopping
      </span>
    );
  }

  return (
    <span
      data-testid="status-pill"
      data-state={state}
      className="inline-flex items-center gap-1.5 rounded-full bg-warning-bg/20 px-2.5 py-1 text-[12px] font-semibold text-warning-fg"
    >
      <span className="h-1.5 w-1.5 rounded-full bg-warning-fg" />
      Stopped
    </span>
  );
}

export function ToggleRow({
  label,
  description,
  checked,
  onChange,
  testId,
}: {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (v: boolean) => void;
  testId?: string;
}) {
  return (
    <div className="flex items-start justify-between gap-4 rounded-lg border border-border bg-background/50 px-3.5 py-3">
      <div>
        <div className="text-sm font-medium text-foreground">{label}</div>
        {description && <p className="mt-0.5 text-[12px] text-muted-foreground">{description}</p>}
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        data-testid={testId}
        onClick={() => onChange(!checked)}
        className={[
          "relative mt-0.5 inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full transition-colors",
          checked ? "bg-primary" : "bg-border",
        ].join(" ")}
      >
        <span
          className={[
            "inline-block h-4 w-4 transform rounded-full bg-white shadow transition-transform",
            checked ? "translate-x-[18px]" : "translate-x-0.5",
          ].join(" ")}
        />
      </button>
    </div>
  );
}