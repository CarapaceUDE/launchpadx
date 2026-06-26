import { defineConfig } from "@playwright/test";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const isReadmeScreenshot = process.argv.some((arg) => arg.includes("readme-screenshot"));
const configFixture = isReadmeScreenshot
  ? path.join(__dirname, "e2e", "fixtures", "readme-config.json")
  : path.join(__dirname, "e2e", "fixtures", "config.json");
const port = process.env.CODEX_E2E_PORT ?? "4173";

export default defineConfig({
  testDir: "./e2e",
  // Shared codex-config.toml fixture — avoid parallel workers racing on disk state.
  workers: 1,
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
  use: {
    baseURL: `http://127.0.0.1:${port}`,
    trace: "on-first-retry",
  },
  webServer: {
    command: `cargo run -- --serve-only --port ${port} --config "${configFixture}"`,
    cwd: repoRoot,
    url: `http://127.0.0.1:${port}`,
    timeout: 120_000,
    reuseExistingServer: !process.env.CI,
  },
});