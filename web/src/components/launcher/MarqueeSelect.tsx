import { useEffect, useId, useRef, useState } from "react";
import { ChevronDown, Loader2 } from "lucide-react";

export function MarqueeSelect({
  value,
  options,
  placeholder,
  disabled,
  busy,
  onChange,
  testId,
  emptyLabel = "No models",
}: {
  value: string;
  options: { value: string; label: string }[];
  placeholder?: string;
  disabled?: boolean;
  busy?: boolean;
  onChange: (value: string) => void;
  testId?: string;
  emptyLabel?: string;
}) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const listId = useId();
  const empty = options.length === 0;
  const inactive = disabled || empty || busy;
  const display =
    options.find((o) => o.value === value)?.label ??
    (empty ? emptyLabel : placeholder ?? "Select...");
  const marquee = display.length > 22;

  useEffect(() => {
    if (!open) return;

    const onPointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen(false);
    };

    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  const toggle = () => {
    if (inactive) return;
    setOpen((prev) => !prev);
  };

  const selectValue = (next: string) => {
    onChange(next);
    setOpen(false);
  };

  return (
    <div ref={rootRef} className="relative min-w-0 flex-1">
      <button
        type="button"
        data-testid={testId}
        role="combobox"
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-controls={listId}
        disabled={inactive}
        aria-busy={busy}
        onClick={toggle}
        className={[
          "flex h-7 w-full items-center rounded-md border border-input bg-background pl-2 pr-7 text-left text-[11px] text-foreground transition-colors",
          "hover:bg-muted/40 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/15",
          "disabled:cursor-not-allowed disabled:bg-muted/50 disabled:text-muted-foreground",
          open ? "border-primary ring-2 ring-primary/15" : "",
        ].join(" ")}
      >
        <span
          className={[
            "min-w-0 flex-1",
            marquee ? "overflow-hidden whitespace-nowrap" : "truncate",
          ].join(" ")}
        >
          <span className={marquee ? "marquee-select-label inline-block pr-6" : ""}>{display}</span>
        </span>
      </button>

      {busy ? (
        <Loader2 className="pointer-events-none absolute right-1.5 top-1/2 h-3 w-3 -translate-y-1/2 animate-spin text-primary" />
      ) : (
        <ChevronDown
          className={[
            "pointer-events-none absolute right-1.5 top-1/2 h-3 w-3 -translate-y-1/2 text-muted-foreground transition-transform",
            open ? "rotate-180" : "",
          ].join(" ")}
        />
      )}

      {open && !empty ? (
        <ul
          id={listId}
          role="listbox"
          data-testid={testId ? `${testId}-listbox` : undefined}
          className="themed-scrollbar absolute left-0 right-0 top-[calc(100%+4px)] z-50 max-h-48 overflow-y-auto rounded-md border border-border bg-popover py-1 shadow-lg"
        >
          {options.map((option) => {
            const selected = option.value === value;
            return (
              <li
                key={option.value}
                role="option"
                aria-selected={selected}
                title={option.label}
                onClick={() => selectValue(option.value)}
                className={[
                  "cursor-pointer px-2.5 py-1.5 text-[11px] leading-snug text-popover-foreground transition-colors",
                  "hover:bg-accent hover:text-accent-foreground",
                  selected ? "bg-primary/10 font-medium text-foreground" : "",
                ].join(" ")}
              >
                {option.label}
              </li>
            );
          })}
        </ul>
      ) : null}
    </div>
  );
}