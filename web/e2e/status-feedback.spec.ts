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

test.describe("Status feedback", () => {
  test.beforeEach(async () => {
    fs.writeFileSync(codexFixture, seedToml, "utf8");
  });

  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("page-launcher")).toBeVisible();
    await expect(page.getByTestId("status-strip")).not.toHaveAttribute("data-operation", "initializing", {
      timeout: 30_000,
    });
  });

  test("local api switch shows immediate busy feedback", async ({ page }) => {
    await expect(page.getByTestId("provider-mode-codex")).toHaveAttribute("aria-selected", "true");

    const localBtn = page.getByTestId("provider-activate-local");
    await expect(localBtn).toBeEnabled();

    await localBtn.click();

    await expect(page.getByTestId("provider-mode-local")).toHaveAttribute("aria-busy", "true");
    await expect(page.getByTestId("status-strip")).toContainText(/Switching to Local API|Applying Local API/i);
  });

  test("refresh models shows immediate status feedback", async ({ page }) => {
    await page.getByTestId("refresh-models").click();

    await expect(page.getByTestId("refresh-models")).toHaveAttribute("aria-busy", "true");
    await expect(page.getByTestId("status-strip")).toHaveAttribute("data-operation", "refreshing_models");
    await expect(page.getByTestId("status-strip")).toContainText(/Refreshing models/i);
  });

  test("launch click shows immediate starting feedback", async ({ page }) => {
    await page.route("**/rpc", async (route) => {
      const body = route.request().postDataJSON() as { method?: string };
      if (body.method === "launch") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({
            ok: true,
            data: { ok: true, message: "Launching Codex via mock" },
            error: null,
          }),
        });
        return;
      }
      if (body.method === "healthCheck") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({
            ok: true,
            data: { running: false, apiReady: false, endpointReady: false },
            error: null,
          }),
        });
        return;
      }
      await route.continue();
    });

    const launchBtn = page.getByTestId("launch-toggle");
    if (!(await launchBtn.isEnabled())) {
      test.skip(true, "Launch disabled without models in this environment");
    }

    await launchBtn.click();

    await expect(launchBtn).toHaveAttribute("aria-busy", "true");
    await expect(launchBtn).toContainText(/Starting/i);
    await expect(page.getByTestId("status-pill")).toHaveAttribute("data-state", "starting");
    await expect(page.getByTestId("status-strip")).toContainText(/Starting Codex|Waiting for Codex/i);
  });

  test("sidebar reflects refresh operation", async ({ page }) => {
    await page.getByTestId("sidebar-refresh-models").click();
    await expect(page.getByTestId("sidebar-refresh-models")).toContainText("Refreshing");
  });

  test("detects when codex stops externally", async ({ page }) => {
    let running = true;

    await page.route("**/rpc", async (route) => {
      const body = route.request().postDataJSON() as { method?: string };
      if (body.method === "healthCheck") {
        await route.fulfill({
          status: 200,
          contentType: "application/json",
          body: JSON.stringify({
            ok: true,
            data: { running, apiReady: running, endpointReady: true },
            error: null,
          }),
        });
        return;
      }
      await route.continue();
    });

    await page.goto("/");
    await expect(page.getByTestId("status-strip")).not.toHaveAttribute("data-operation", "initializing", {
      timeout: 30_000,
    });
    await expect(page.getByTestId("launch-toggle")).toContainText("Stop Codex", { timeout: 15_000 });

    running = false;
    await expect(page.getByTestId("launch-toggle")).toContainText("Start Codex", { timeout: 15_000 });
    await expect(page.getByTestId("status-pill")).toHaveAttribute("data-state", "stopped");
  });
});