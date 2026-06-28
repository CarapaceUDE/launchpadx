import { Moon, Sun } from "lucide-react";
import { useEffect, useState } from "react";

function getInitialTheme(): "light" | "dark" {
  if (typeof window === "undefined") return "light";
  const saved = localStorage.getItem("theme");
  if (saved === "dark" || saved === "light") return saved;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export function ThemeToggle({ variant = "sidebar" }: { variant?: "sidebar" | "card" }) {
  const [theme, setTheme] = useState<"light" | "dark">("light");

  useEffect(() => {
    const t = getInitialTheme();
    setTheme(t);
    document.documentElement.classList.toggle("dark", t === "dark");
  }, []);

  const toggle = () => {
    const next = theme === "dark" ? "light" : "dark";
    setTheme(next);
    document.documentElement.classList.toggle("dark", next === "dark");
    localStorage.setItem("theme", next);
  };

  const isDark = theme === "dark";
  const isCard = variant === "card";

  return (
    <button
      type="button"
      data-testid="theme-toggle"
      onClick={toggle}
      aria-label="Toggle dark mode"
      title={isDark ? "Switch to light mode" : "Switch to dark mode"}
      className={[
        "flex w-full items-center justify-between rounded-lg px-3.5 py-2.5 text-sm transition-colors",
        isCard
          ? "border border-border bg-card text-foreground hover:bg-muted/70"
          : "border border-white/10 bg-white/5 text-white/85 hover:bg-white/10",
      ].join(" ")}
    >
      <span className="flex items-center gap-2">
        {isDark ? <Moon className="h-3.5 w-3.5" /> : <Sun className="h-3.5 w-3.5" />}
        {isDark ? "Dark Mode" : "Light Mode"}
      </span>
      <span
        className={[
          "relative h-4 w-7 rounded-full transition-colors",
          isDark ? "bg-success" : isCard ? "bg-muted" : "bg-white/25",
        ].join(" ")}
      >
        <span
          className={[
            "absolute top-0.5 h-3 w-3 rounded-full bg-white transition-all",
            isDark ? "left-3.5" : "left-0.5",
          ].join(" ")}
        />
      </span>
    </button>
  );
}