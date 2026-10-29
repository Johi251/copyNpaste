/**
 * copyNpaste Browser-Add-on — Popup-Logik.
 * Alle Änderungen laufen über den gemeinsamen Hintergrund-Schreibpfad.
 */

const CAPACITY = 25;
const listEl = document.getElementById("list");
const searchEl = document.getElementById("search");
const countEl = document.getElementById("count");

const urlRe = /^https?:\/\/[^\s]+$/i;

let items = []; // { id, text, at, pinned } — neueste zuerst
let filter = "";

async function loadStore() {
  const { magazine = [] } = await chrome.storage.local.get("magazine");
  if (!Array.isArray(magazine) || magazine.some(i => !i || typeof i.id !== "string" || typeof i.text !== "string")) {
    throw new Error("Gespeichertes Magazin ist beschädigt; der Bestand bleibt erhalten.");
  }
  items = magazine;
}

async function changeStore(message) {
  const result = await chrome.runtime.sendMessage({ target: "magazine", ...message });
  if (result?.error) throw new Error(result.error);
  if (!result?.ok) throw new Error("Magazin nicht erreichbar");
  await loadStore();
  countEl.textContent = `${items.length} / ${CAPACITY}`;
  render();
}

/** Liest die Zwischenablage und stapelt sie (sofern der Inhalt neu ist). */
async function harvestClipboard() {
  try {
    const text = (await navigator.clipboard.readText() ?? "");
    if (!text.trim() || text.length > 100_000) return;
    if (items[0]?.text === text) return; // unverändert → kein neuer Platz
    await changeStore({ type: "capture", text });
  } catch {
    document.getElementById("hint").textContent = "Zwischenablage konnte nicht gelesen werden. Bitte Rechte und Browserfokus prüfen.";
  }
}

/** Kopiert ein Element zurück in die Zwischenablage. */
async function copyBack(item) {
  try {
    await navigator.clipboard.writeText(item.text);
    await changeStore({ type: "consume", id: item.id });
  } catch {
    document.getElementById("hint").textContent = "Kopieren fehlgeschlagen; der Eintrag bleibt erhalten.";
  }
}

function fmtTime(ts) {
  const d = new Date(ts);
  const today = new Date();
  const sameDay = d.toDateString() === today.toDateString();
  const hm = `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
  return sameDay ? hm : `${String(d.getDate()).padStart(2, "0")}.${String(d.getMonth() + 1).padStart(2, "0")}. ${hm}`;
}

function render() {
  const visible = filter
    ? items.filter((i) => i.text.toLowerCase().includes(filter))
    : items;
  listEl.innerHTML = "";
  if (visible.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty";
    empty.innerHTML = `<div class="big">${filter ? "Keine Treffer" : "Magazin ist leer"}</div>
      <div>${filter ? "Suchbegriff anpassen" : "Etwas kopieren (Strg+C), dann Add-on öffnen"}</div>`;
    listEl.append(empty);
    return;
  }
  for (const item of visible) {
    const isUrl = urlRe.test(item.text);
    const el = document.createElement("div");
    el.className = "item";
    el.dataset.id = item.id;

    const meta = document.createElement("div");
    meta.className = "meta";
    meta.innerHTML = `<span>${fmtTime(item.at)}</span>${isUrl ? '<span class="tag">URL</span>' : ""}<span>${item.text.length} Zeichen</span>`;

    const body = document.createElement("div");
    body.className = `body${isUrl ? " url" : ""}`;
    body.textContent = item.text;

    const check = document.createElement("span");
    check.className = "check";
    check.textContent = "✓";

    el.append(meta, body, check);
    el.addEventListener("click", () => void copyBack(item));
    listEl.append(el);
  }
}

// ─── Ereignisse ───────────────────────────────────────────────────────────
searchEl.addEventListener("input", () => {
  filter = searchEl.value.trim().toLowerCase();
  render();
});
searchEl.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    const visible = filter ? items.filter((i) => i.text.toLowerCase().includes(filter)) : items;
    if (visible[0]) void copyBack(visible[0]);
  }
});
document.getElementById("clear").addEventListener("click", async () => {
  try { await changeStore({ type: "clear" }); }
  catch (e) { document.getElementById("hint").textContent = String(e); }
});

chrome.storage.onChanged.addListener((changes, area) => {
  if (area === "local" && changes.magazine) void loadStore().then(() => {
    countEl.textContent = `${items.length} / ${CAPACITY}`;
    render();
  }).catch(e => { document.getElementById("hint").textContent = String(e); });
});
document.getElementById("read").addEventListener("click", () => void harvestClipboard());

// ─── Start ────────────────────────────────────────────────────────────────
(async () => {
  await loadStore();
  countEl.textContent = `${items.length} / ${CAPACITY}`;
  render();
  searchEl.focus();

})().catch(e => { document.getElementById("hint").textContent = String(e); });
