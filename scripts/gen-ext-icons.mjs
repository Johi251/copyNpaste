/**
 * Rendert das Add-on-Icon-Set (16/32/48/128 px) aus assets/logo.svg
 * nach web-extension/icons/. Nutzung: node scripts/gen-ext-icons.mjs
 */
import { Resvg } from "@resvg/resvg-js";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const svg = readFileSync(resolve(root, "assets", "logo.svg"), "utf8");
const outDir = resolve(root, "web-extension", "icons");

mkdirSync(outDir, { recursive: true });
for (const size of [16, 32, 48, 128]) {
  const png = new Resvg(svg, {
    fitTo: { mode: "width", value: size },
    background: "rgba(0,0,0,0)",
  })
    .render()
    .asPng();
  const out = resolve(outDir, `icon-${size}.png`);
  writeFileSync(out, png);
  console.log(`OK: ${out} (${png.length} Bytes)`);
}
