import { Github, Globe, Heart, MessageCircle, Rocket } from "lucide-react";
import {
  APP_NAME,
  LICENSE_NOTICE,
  LICENSE_URL,
  ORG_DISCORD,
  ORG_GITHUB,
  ORG_PATREON,
  ORG_TAGLINE,
  ORG_WEBSITE,
  TRADEMARK_NOTICE,
} from "../../lib/branding";

export function AboutPanel() {
  return (
    <div className="card-surface p-6">
      <div className="space-y-4 text-center">
        <div className="mb-3 flex items-center justify-center">
          <div className="grid h-12 w-12 place-items-center rounded-xl bg-primary/10 ring-1 ring-primary/20">
            <Rocket className="h-6 w-6 text-primary" />
          </div>
        </div>
        <h3 className="text-[16px] font-semibold text-foreground">{APP_NAME}</h3>
        <p className="text-[12px] text-muted-foreground">{ORG_TAGLINE}</p>
        <p className="text-[12px] text-muted-foreground">Version 0.1.0</p>
      </div>

      <div className="mt-4 space-y-2">
        <a
          href={ORG_GITHUB}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center justify-center gap-2 rounded-lg bg-primary/10 p-3 py-2.5 text-sm text-primary transition-colors hover:bg-primary/15"
        >
          <Github className="h-4 w-4" />
          View on GitHub
        </a>
        <a
          href={ORG_WEBSITE}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center justify-center gap-2 rounded-lg border border-border p-3 py-2.5 text-sm text-muted-foreground transition-colors hover:bg-muted/50"
        >
          <Globe className="h-4 w-4" />
          CarapaceAI
        </a>
        <a
          href={ORG_PATREON}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center justify-center gap-2 rounded-lg border border-border p-3 py-2.5 text-sm text-muted-foreground transition-colors hover:bg-muted/50"
        >
          <Heart className="h-4 w-4" />
          Official Builds (Patreon)
        </a>
        <a
          href={ORG_DISCORD}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center justify-center gap-2 rounded-lg border border-border p-3 py-2.5 text-sm text-muted-foreground transition-colors hover:bg-muted/50"
        >
          <MessageCircle className="h-4 w-4" />
          Join Community
        </a>
      </div>

      <div className="mt-4 border-t border-border pt-4 text-left">
        <h4 className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
          Features
        </h4>
        <ul className="space-y-1.5 text-[12px] text-foreground/70">
          {[
            "Start/stop Codex-compatible local API server",
            "Auto-discovery of available models",
            "Endpoint configuration with auto-generated Base URL",
            "Dark/light theme support",
            "Application log viewer",
          ].map((feature) => (
            <li key={feature} className="flex items-start gap-2">
              <span className="mt-0.5 text-success">•</span>
              {feature}
            </li>
          ))}
        </ul>
      </div>

      <div className="mt-4 border-t border-border pt-3 text-[10px] text-muted-foreground">
        <p>
          <a
            href={LICENSE_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="text-foreground/70 underline-offset-2 hover:text-foreground hover:underline"
          >
            {LICENSE_NOTICE}
          </a>
        </p>
        <p className="mt-1">Built with React, Tailwind CSS, and Rust.</p>
        <p className="mt-2 leading-relaxed">{TRADEMARK_NOTICE}</p>
      </div>
    </div>
  );
}