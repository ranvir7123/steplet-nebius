import { test, expect } from "@playwright/test";

test("the check page verifies a signature in the browser and cannot send anything", async ({ page }) => {
  const requests: string[] = [];
  const consoleLines: string[] = [];
  page.on("request", (r) => requests.push(`${r.method()} ${r.url()}`));
  page.on("console", (m) => consoleLines.push(`${m.type()}: ${m.text()}`));
  await page.goto("/");
  const csp = await page.locator('meta[http-equiv="Content-Security-Policy"]').getAttribute("content");
  expect(csp).toContain("connect-src 'none'");
  await expect(page.getByTestId("valid")).toContainText("valid (SHA256:");
  await expect(page.getByTestId("tampered")).toContainText("invalid: does not verify");
  await expect(page.getByTestId("fetch")).toHaveText("Network request from this page: blocked");
  expect(requests.filter((r) => r.startsWith("POST"))).toEqual([]);
  expect(requests.every((r) => r.includes("127.0.0.1:4179"))).toBe(true);
  console.log("requests:", JSON.stringify(requests));
  console.log("console:", JSON.stringify(consoleLines));
});
