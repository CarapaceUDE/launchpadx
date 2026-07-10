import { Download, Github, Globe, MessageCircle } from "lucide-react";
import { AppIcon } from "./AppIcon";
import {
  APP_NAME,
  LICENSE_NOTICE,
  LICENSE_URL,
  ORG_DISCORD,
  ORG_GITHUB,
  ORG_TAGLINE,
  ORG_WEBSITE,
  ISSUES_URL,
  RELEASES_URL,
  TRADEMARK_NOTICE,
} from "../../lib/branding";

export function AboutPanel() {
  return (
    <div className="card-surface p-6">
      <div className="space-y-4 text-center">
        <div className="mb-3 flex items-center justify-center">
          <div className="h-12 w-12 overflow-hidden rounded-xl ring-1 ring-primary/20">
            <AppIcon size={48} className="h-full w-full" />
          </div>
        </div>
        <h3 className="text-[16px] font-semibold text-foreground">{APP_NAME}</h3>
        <p className="text-[12px] text-muted-foreground">{ORG_TAGLINE}</p>
        <p className="text-[12px] text-muted-foreground">Version 0.2.3</p>
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
          href={RELEASES_URL}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center justify-center gap-2 rounded-lg border border-border p-3 py-2.5 text-sm text-muted-foreground transition-colors hover:bg-muted/50"
        >
          <Download className="h-4 w-4" />
          Download Releases
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
          Open Source Releases
        </h4>
        <div className="space-y-2 text-[12px] leading-relaxed text-foreground/70">
          <p>
            Pre-compiled Windows, macOS, and Linux archives are available on{" "}
            <a
              href={RELEASES_URL}
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary underline-offset-2 hover:underline"
            >
              GitHub Releases
            </a>{" "}
            and are built automatically from each tagged source version.
          </p>
          <p>
            Found a bug? Please file a concise report with reproduction steps on{" "}
            <a
              href={ISSUES_URL}
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary underline-offset-2 hover:underline"
            >
              GitHub Issues
            </a>. For setup help, join our{" "}
            <a
              href={ORG_DISCORD}
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary underline-offset-2 hover:underline"
            >
              community Discord
            </a>{" "}
            and we will help you get set up.
          </p>
        </div>
      </div>

      <div className="mt-4 border-t border-border pt-3 text-[10px] text-muted-foreground">
        <p>
          <a
            href={LICENSE_URL}
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
