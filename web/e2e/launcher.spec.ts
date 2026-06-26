import { expect, test } from "@playwright/test";

test.describe("Codex Launchpad", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("page-launcher")).toBeVisible();
    await expect(page.getByTestId("status-strip")).not.toHaveAttribute("data-operation", "initializing", {
      timeout: 30_000,
    });
  });

  test("loads the launcher shell", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Launcher" })).toBeVisible();
    await expect(page.getByTestId("provider-mode-card")).toBeVisible();
    await expect(page.getByTestId("provider-mode-codex")).toBeVisible();
    await expect(page.getByTestId("provider-mode-local")).toBeVisible();
    await expect(page.getByTestId("model-select")).toBeVisible();
    await expect(page.getByTestId("launch-toggle")).toBeVisible();
  });

  test("switch to codex account requires confirmation", async ({ page }) => {
    await page.getByTestId("provider-activate-local").click();
    await expect(page.getByTestId("provider-mode-status")).toContainText(/Local API/i, {
      timeout: 15_000,
    });

    await page.getByTestId("provider-activate-codex").click();
    await expect(page.getByTestId("provider-confirm-dialog")).toBeVisible();

    await page.getByTestId("provider-confirm-cancel").click();
    await expect(page.getByTestId("provider-confirm-dialog")).toHaveCount(0);
  });

  test("provider settings cog opens settings with provider selected", async ({ page }) => {
    await page.getByTestId("provider-settings-local").click();
    await expect(page.getByTestId("provider-settings-panel")).toBeVisible();
    await expect(page.getByTestId("settings-provider-select")).toHaveValue("local");
    await expect(page.getByTestId("endpoint-ip")).toBeVisible();
  });

  test("sidebar settings defaults to local API with endpoint fields", async ({ page }) => {
    await page.getByTestId("nav-settings").click();
    await expect(page.getByTestId("provider-settings-panel")).toBeVisible();
    await expect(page.getByTestId("settings-provider-select")).toHaveValue("local");
    await expect(page.getByTestId("endpoint-ip")).toBeVisible();
    await expect(page.getByTestId("endpoint-port")).toBeVisible();
    await expect(page.getByTestId("endpoint-base-url")).toBeVisible();
  });

  test("working directory is editable in settings", async ({ page }) => {
    await page.getByTestId("nav-settings").click();
    await page.getByTestId("settings-provider-select").selectOption("codex");

    const input = page.getByTestId("working-directory");
    await input.fill("C:\\e2e\\workspace");
    await expect(input).toHaveValue("C:\\e2e\\workspace");
  });

  test("endpoint fields sync the generated base URL in settings", async ({ page }) => {
    await page.getByTestId("provider-settings-local").click();
    await page.getByTestId("endpoint-ip").fill("10.0.0.5");
    await page.getByTestId("endpoint-port").fill("8080");
    await expect(page.getByTestId("endpoint-base-url")).toHaveValue("http://10.0.0.5:8080/v1");
  });

  test("sidebar navigation reaches About", async ({ page }) => {
    await page.getByTestId("nav-about").click();
    await expect(page.getByRole("heading", { name: "Codex Launchpad" })).toBeVisible();
    await expect(page.getByText("A Carapace LLC community project")).toBeVisible();
    await expect(page.getByRole("link", { name: "Join Community" })).toHaveAttribute(
      "href",
      "https://carapaceai.org/discord",
    );
    await expect(page.getByText("Version 0.1.0")).toBeVisible();
  });

  test("auto-start toggle is clickable in settings", async ({ page }) => {
    await page.getByTestId("nav-settings").click();
    await page.getByTestId("settings-provider-select").selectOption("codex");

    const toggle = page.getByTestId("auto-start-toggle");
    const wasChecked = await toggle.getAttribute("aria-checked");
    await toggle.click();
    await expect(toggle).toHaveAttribute("aria-checked", wasChecked === "true" ? "false" : "true");
  });
});