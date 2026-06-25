import { expect, test } from "@playwright/test";

test.describe("Codex Local Launcher", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("page-launcher")).toBeVisible();
  });

  test("loads the launcher shell", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Launcher" })).toBeVisible();
    await expect(page.getByTestId("launch-toggle")).toBeVisible();
    await expect(page.getByTestId("revert-codex-profile")).toBeVisible();
    await expect(page.getByTestId("write-codex-config")).toBeVisible();
  });

  test("revert dialog requires confirmation", async ({ page }) => {
    await page.getByTestId("revert-codex-profile").click();
    await expect(page.getByTestId("revert-confirm-dialog")).toBeVisible();

    await page.getByTestId("revert-cancel").click();
    await expect(page.getByTestId("revert-confirm-dialog")).toHaveCount(0);
  });

  test("working directory is editable in the UI", async ({ page }) => {
    const input = page.getByTestId("working-directory");
    await input.fill("C:\\e2e\\workspace");
    await expect(input).toHaveValue("C:\\e2e\\workspace");
  });

  test("endpoint fields sync the generated base URL", async ({ page }) => {
    await page.getByTestId("endpoint-ip").fill("10.0.0.5");
    await page.getByTestId("endpoint-port").fill("8080");
    await expect(page.getByTestId("endpoint-base-url")).toHaveValue("http://10.0.0.5:8080/v1");
  });

  test("sidebar navigation reaches About", async ({ page }) => {
    await page.getByTestId("nav-about").click();
    await expect(page.getByRole("heading", { name: "Codex Local Launcher" })).toBeVisible();
    await expect(page.getByText("Version 0.1.0")).toBeVisible();
  });

  test("auto-start toggle is clickable", async ({ page }) => {
    const toggle = page.getByTestId("auto-start-toggle");
    await expect(toggle).toHaveAttribute("aria-checked", "false");
    await toggle.click();
    await expect(toggle).toHaveAttribute("aria-checked", "true");
  });
});