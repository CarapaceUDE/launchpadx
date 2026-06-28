import { expect, test } from "@playwright/test";
import http from "http";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const output = path.resolve(__dirname, "../../assets/readme-screenshot.png");
const mockPort = 11499;

let mockServer: http.Server | undefined;

test.describe.configure({ mode: "serial" });

test.beforeAll(async () => {
  mockServer = http.createServer((req, res) => {
    if (req.url?.includes("/api/tags")) {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(
        JSON.stringify({
          models: [
            { name: "llama3.2", model: "llama3.2:latest" },
            { name: "qwen2.5-coder", model: "qwen2.5-coder:latest" },
            { name: "deepseek-r1", model: "deepseek-r1:latest" },
          ],
        }),
      );
      return;
    }

    res.writeHead(200, { "Content-Type": "application/json" });
    res.end("{}");
  });

  await new Promise<void>((resolve, reject) => {
    mockServer!.listen(mockPort, "127.0.0.1", () => resolve());
    mockServer!.on("error", reject);
  });
});

test.afterAll(async () => {
  await new Promise<void>((resolve, reject) => {
    if (!mockServer) {
      resolve();
      return;
    }
    mockServer.close((err) => (err ? reject(err) : resolve()));
  });
});

test("capture readme screenshot", async ({ page }) => {
  await page.setViewportSize({ width: 1360, height: 840 });
  await page.goto("/");
  await expect(page.getByTestId("page-launcher")).toBeVisible();
  await expect(page.getByTestId("status-strip")).toHaveAttribute("data-operation", "idle", {
    timeout: 30_000,
  });
  await expect(page.getByText(/Model refresh failed/i)).toHaveCount(0);

  await page.getByTestId("provider-activate-local").click();
  const confirm = page.getByTestId("provider-confirm-dialog");
  if (await confirm.isVisible().catch(() => false)) {
    await page.getByTestId("provider-confirm-switch").click();
  }
  await expect(page.getByTestId("status-strip")).toHaveAttribute("data-operation", "idle", {
    timeout: 30_000,
  });
  await expect(page.getByTestId("provider-mode-status")).toContainText(/Local API/i);

  await page.evaluate(() => localStorage.setItem("theme", "dark"));
  await page.reload();
  await expect(page.getByTestId("page-launcher")).toBeVisible();
  await expect(page.getByTestId("status-strip")).toHaveAttribute("data-operation", "idle", {
    timeout: 30_000,
  });
  await expect(page.getByTestId("provider-mode-card")).toBeVisible();

  await page.screenshot({
    path: output,
    fullPage: false,
    animations: "disabled",
  });
});