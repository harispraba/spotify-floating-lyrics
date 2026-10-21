// Renders the README images (docs/*.png) from the built frontend in headless Edge or Chrome.
// The frontend runs for real; mock.js stands in for Tauri with made-up lyrics and a frozen clock.
// Usage, from the repo root: npm run build, then cd docs/render && npm ci && npm run render
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import puppeteer from "puppeteer-core";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, "../..");
const DIST = path.join(ROOT, "dist");
const OUT = path.join(ROOT, "docs");
const BROWSER =
  process.env.BROWSER_PATH ?? "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe";

const SCENES = [
  { scene: "hero", file: "overlay.png", size: { width: 1120, height: 520 } },
  { scene: "toolbar", file: "toolbar.png", size: { width: 880, height: 300 }, hover: true },
];

if (!fs.existsSync(path.join(DIST, "index.html"))) {
  console.error("dist/ is missing. Run `npm run build` in the repo root first.");
  process.exit(1);
}
if (!fs.existsSync(BROWSER)) {
  console.error(`No browser at ${BROWSER}. Set BROWSER_PATH to Edge or Chrome.`);
  process.exit(1);
}

const TYPES = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".woff2": "font/woff2" };
const server = http.createServer((req, res) => {
  const url = new URL(req.url, "http://localhost").pathname;
  const file = url.startsWith("/harness/")
    ? path.join(HERE, url)
    : path.join(DIST, url === "/" ? "index.html" : url);
  fs.readFile(file, (err, data) => {
    if (err) return res.writeHead(404).end();
    res.writeHead(200, { "content-type": TYPES[path.extname(file)] ?? "application/octet-stream" }).end(data);
  });
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const base = `http://127.0.0.1:${server.address().port}`;

const profile = fs.mkdtempSync(path.join(os.tmpdir(), "render-"));
const browser = await puppeteer.launch({
  executablePath: BROWSER,
  headless: true,
  args: ["--hide-scrollbars", "--force-color-profile=srgb"],
  userDataDir: profile,
});
const mock = fs.readFileSync(path.join(HERE, "mock.js"), "utf8");

async function shot({ scene, file, size, hover = false }) {
  const page = await browser.newPage();
  page.on("pageerror", (e) => console.error(`[${scene}] ${e.message}`));
  await page.setViewport({ ...size, deviceScaleFactor: 2 });
  await page.evaluateOnNewDocument(mock);
  await page.goto(`${base}/harness/${scene}.html`, { waitUntil: "networkidle0" });
  const frame = page.frames().find((f) => f !== page.mainFrame());
  await frame.waitForFunction(
    () => (document.getElementById("current-text")?.textContent ?? "").length > 0 && document.fonts.status === "loaded",
    { timeout: 10000 },
  );
  if (hover) {
    const box = await (await page.$("iframe")).boundingBox();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2 + 20);
  }
  // Let the line's rise animation and the hover transition finish.
  await new Promise((resolve) => setTimeout(resolve, 900));
  const out = path.join(OUT, file);
  await page.screenshot({ path: out });
  console.log(`${path.relative(ROOT, out)}: ${(fs.statSync(out).size / 1024).toFixed(0)} KiB`);
  await page.close();
}

try {
  for (const scene of SCENES) await shot(scene);
} finally {
  await browser.close();
  server.close();
  fs.rmSync(profile, { recursive: true, force: true });
}
