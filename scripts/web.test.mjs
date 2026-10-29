import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";

test("Add-on-Shortcut verwendet Offscreen-Dokument und bewahrt Einrückungen", async () => {
  let command, stored;
  const chrome = {
    runtime: { onMessage: { addListener() {} }, getContexts: async () => [], sendMessage: async () => ({ text: "  code\n" }) },
    offscreen: { createDocument: async () => {} },
    commands: { onCommand: { addListener(fn) { command = fn; } } },
    storage: { local: { get: async () => ({}), set: async value => { stored = value; } } },
    action: { setBadgeText: async () => {}, setTitle: async () => {} },
  };
  runInNewContext(readFileSync("web-extension/background.js", "utf8"), { chrome, navigator: {}, crypto: { randomUUID: () => "test" } });
  await command("push-clipboard");
  assert.equal(stored.magazine[0].text, "  code\n");
});

test("Add-on meldet Clipboard-Fehler, ohne Magazin zu ändern", async () => {
  let command, changed = false, badge;
  const chrome = {
    runtime: { onMessage: { addListener() {} }, getContexts: async () => [{ id: 1 }], sendMessage: async () => ({ error: "denied" }) },
    commands: { onCommand: { addListener(fn) { command = fn; } } },
    storage: { local: { set: async () => { changed = true; } } },
    action: { setBadgeText: async value => { badge = value.text; }, setTitle: async () => {} },
  };
  runInNewContext(readFileSync("web-extension/background.js", "utf8"), { chrome, navigator: {} });
  await command("push-clipboard");
  assert.equal(changed, false);
  assert.equal(badge, "!");
});

test("Web-App: Originaltext und Entnahme nur nach Clipboard-Erfolg", async () => {
  const elements = new Map();
  const node = () => ({ textContent: "", innerHTML: "", value: "", style: {}, dataset: {}, children: [],
    classList: { add() {}, remove() {} }, setAttribute() {}, addEventListener() {}, append(...children) { this.children.push(...children); } });
  let fail = true;
  let source = readFileSync("web-app/app.js", "utf8");
  source = source.replace("load().then(render).catch(error);", "globalThis.api = { harvestText, copyBack, getItems: () => items, setDb: value => { db = value; } };");
  const context = { document: { getElementById(id) { if (!elements.has(id)) elements.set(id, node()); return elements.get(id); },
    createElement: node, addEventListener() {} }, navigator: { clipboard: { writeText: async () => { if (fail) throw Error("denied"); } } },
    crypto: { randomUUID: () => "test" }, Date, Promise, Element: class {}, localStorage: {}, indexedDB: {} };
  runInNewContext(source, context);
  context.api.setDb({ transaction() { const tx = { objectStore: () => ({ put() { queueMicrotask(() => tx.oncomplete()); } }) }; return tx; } });
  await context.api.harvestText("  code\n");
  assert.equal(context.api.getItems()[0].text, "  code\n");
  await context.api.copyBack(context.api.getItems()[0]);
  assert.equal(context.api.getItems().length, 1);
  fail = false;
  await context.api.copyBack(context.api.getItems()[0]);
  assert.equal(context.api.getItems().length, 0);
});
