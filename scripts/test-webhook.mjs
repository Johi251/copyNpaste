/**
 * Test-Empfänger für die Localhost-Smart-Bridge ("Today's Hunt"-Schnittstelle).
 * Startet einen HTTP-Server auf 127.0.0.1:9911 und schreibt jeden POST nach
 * scripts/.webhook-received.json — für den End-to-End-Nachweis des Webhooks.
 *
 * Nutzung: node scripts/test-webhook.mjs [Port]
 */
import { createServer } from "node:http";
import { appendFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const port = Number(process.argv[2] ?? 9911);
const logFile = resolve(dirname(fileURLToPath(import.meta.url)), ".webhook-received.jsonl");

writeFileSync(logFile, "");
const server = createServer((req, res) => {
  const chunks = [];
  req.on("data", (c) => chunks.push(c));
  req.on("end", () => {
    const body = Buffer.concat(chunks).toString("utf8");
    const line = JSON.stringify({ at: new Date().toISOString(), method: req.method, url: req.url, body });
    appendFileSync(logFile, line + "\n");
    console.log(`POST ${req.url} ← ${body}`);
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ ok: true, module: "todays-hunt-stub" }));
  });
});

server.listen(port, "127.0.0.1", () => {
  console.log(`Today's-Hunt-Stub lauscht auf http://127.0.0.1:${port}/api/todays-hunt`);
  console.log(`Mitschnitt: ${logFile}`);
});
