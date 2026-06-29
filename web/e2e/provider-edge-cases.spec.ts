import { expect, test } from "@playwright/test";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const codexFixture = path.join(__dirname, "fixtures", "codex-config.toml");
const backupDir = path.join(__dirname, "fixtures", "backups", "codex-launchpad");
const seedToml = `model = "gpt-test"
model_provider = "openai"

[model_providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com/v1"
`;

test.describe.configure({ mode: "serial" });

test.describe("Provider edge cases", () => {
  test.beforeEach(async () => {
    fs.writeFileSync(codexFixture, seedToml, "utf8");
    if (fs.existsSync(backupDir)) {
      fs.rmSync(backupDir, { recursive: true, force: true });
    }
  });

  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("page-launcher")).toBeVisible();
    await expect(page.getByTestId("status-strip")).not.toHaveAttribute("data-operation", "initializing", {
      timeout: 30_000,
    });
  });

  test("blocks provider switch while refreshing models", async ({ page }) => {
    await page.getByTestId("refresh-models").click();
    await expect(page.getByTestId("refresh-models")).toHaveAttribute("aria-busy", "true");

    await expect(page.getByTestId("provider-activate-local")).toBeDisabled();
    await expect(page.getByTestId("provider-activate-codex")).toBeDisabled();
    await expect(page.getByTestId("provider-mode-card")).toContainText(
      /Wait for the current operation to finish/i,
    );
  });

  test("allows start on codex account without a local model", async ({ page }) => {
    await expect(page.getByTestId("provider-mode-codex")).toHaveAttribute("aria-selected", "true");
    await expect(page.getByTestId("launch-toggle")).toBeEnabled();
  });

  test("shows restore warning when switching back without a snapshot", async ({ page }) => {
    await page.getByTestId("provider-activate-local").click();
    await expect(page.getByTestId("provider-mode-status")).toContainText(/Local API/i, {
      timeout: 15_000,
    });

    if (fs.existsSync(backupDir)) {
      fs.rmSync(backupDir, { recursive: true, force: true });
    }
    await page.reload();
    await expect(page.getByTestId("status-strip")).not.toHaveAttribute("data-operation", "initializing", {
      timeout: 30_000,
    });

    await page.getByTestId("provider-activate-codex").click();
    await expect(page.getByTestId("provider-confirm-dialog")).toBeVisible();
    await expect(page.getByTestId("provider-confirm-dialog")).toContainText(
      /restore snapshot|switch Codex back to your account provider/i,
    );
  });
});