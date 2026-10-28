/**
 * Rendert assets/logo.svg → src-tauri/icons/icon-1024.png (App-Icon-Quelle).
 * Nutzung: npm run icon   (danach: npx tauri icon src-tauri/icons/icon-1024.png)
 */
import { Resvg } from "@resvg/resvg-js";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const svgPath = resolve(root, "assets", "logo.svg");
const outPath = resolve(root, "src-tauri", "icons", "icon-1024.png");

const svg = readFileSync(svgPath, "utf8");
const resvg = new Resvg(svg, {
  fitTo: { mode: "width", value: 1024 },
  background: "rgba(0,0,0,0)",
});
const png = resvg.render().asPng();

mkdirSync(dirname(outPath), { recursive: true });
writeFileSync(outPath, png);
console.log(`OK: ${outPath} (${png.length} Bytes)`);
