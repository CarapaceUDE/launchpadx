import { expect, test } from "@playwright/test";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const codexFixture = path.join(__dirname, "fixtures", "codex-config.toml");
const seedToml = `model = "gpt-test"
model_provider = "openai"

[model_providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com/v1"
`;

test.describe.configure({ mode: "serial" });

test.describe("Codex provider mode", () => {
  test.beforeEach(async () => {
    fs.writeFileSync(codexFixture, seedToml, "utf8");
    const backupDir = path.join(__dirname, "fixtures", "backups", "launchpadx");
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

  test("shows provider status on load", async ({ page }) => {
    await expect(page.getByTestId("provider-mode-status")).toBeVisible();
  });

  test("switches to local api and writes launcher settings", async ({ page }) => {
    await page.getByTestId("provider-activate-local").click();
    await expect(page.getByTestId("provider-mode-status")).toContainText(/Local API/i, {
      timeout: 15_000,
    });

    const written = fs.readFileSync(codexFixture, "utf8");
    expect(written).toContain('model_provider = "launchpadx"');
    expect(written).toContain('model = "llama3.2"');
    expect(written).toContain("experimental_bearer_token");
  });

  test("launch preserves codex account config after switching back from local api", async ({ page }) => {
    await page.getByTestId("provider-activate-local").click();
    await expect(page.getByTestId("provider-mode-status")).toContainText(/Local API/i, {
      timeout: 15_000,
    });

    await page.getByTestId("provider-activate-codex").click();
    await page.getByTestId("provider-confirm-switch").click();
    await expect(page.getByTestId("provider-mode-codex")).toHaveAttribute("aria-selected", "true", {
      timeout: 15_000,
    });

    await page.getByTestId("launch-toggle").click();

    await expect
      .poll(() => fs.readFileSync(codexFixture, "utf8"), { timeout: 15_000 })
      .toContain('model_provider = "openai"');
    await expect
      .poll(() => fs.readFileSync(codexFixture, "utf8"))
      .toContain('model = "gpt-test"');
    await expect
      .poll(() => fs.readFileSync(codexFixture, "utf8"))
      .not.toContain("launchpadx");
  });

  test("switches back to codex account after local api", async ({ page }) => {
    await page.getByTestId("provider-activate-local").click();
    await expect(page.getByTestId("provider-mode-status")).toContainText(/Local API/i, {
      timeout: 15_000,
    });

    await page.getByTestId("provider-activate-codex").click();
    await page.getByTestId("provider-confirm-switch").click();

    await expect(page.getByTestId("provider-mode-status")).toContainText(/cloud account|account sign-in|openai/i, {
      timeout: 15_000,
    });

    const restored = fs.readFileSync(codexFixture, "utf8");
    expect(restored).toContain('model_provider = "openai"');
    expect(restored).toContain('model = "gpt-test"');
    expect(restored).not.toContain("launchpadx");
  });

  test("model selection stays valid in the dropdown", async ({ page }) => {
    const trigger = page.getByTestId("model-select");
    await expect(trigger).toBeVisible();

    const isEnabled = await trigger.isEnabled();
    if (!isEnabled) {
      await expect(trigger).toContainText(/No models|Select/i);
      return;
    }

    await trigger.click();
    const listbox = page.getByTestId("model-select-listbox");
    const options = await listbox.getByRole("option").allTextContents();
    const selectable = options.filter((o) => o && !o.includes("Select"));
    expect(selectable.length).toBeGreaterThan(0);
    await listbox.getByRole("option", { name: selectable[0] }).click();
    await expect(trigger).toContainText(selectable[0]);
  });
});