/**
 * copyNpaste Browser-Add-on — Popup-Logik.
 * Beim Öffnen wird die aktuelle Zwischenablage gelesen und (sofern neu)
 * oben ins Magazin gestapelt. Klick auf ein Element kopiert es zurück.
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
  items = magazine;
}

async function saveStore() {
  await chrome.storage.local.set({ magazine: items.slice(0, CAPACITY) });
  countEl.textContent = `${items.length} / ${CAPACITY}`;
}

/** Liest die Zwischenablage und stapelt sie (sofern der Inhalt neu ist). */
async function harvestClipboard() {
  try {
    const text = (await navigator.clipboard.readText() ?? "").trim();
    if (!text || text.length > 100_000) return;
    if (items[0]?.text === text) return; // unverändert → kein neuer Platz
    items.unshift({ id: crypto.randomUUID(), text, at: Date.now() });
    items = items.slice(0, CAPACITY);
    await saveStore();
    render();
  } catch {
    // Lese-Rechte verweigert oder leer — still ignorieren, UI bleibt nutzbar.
  }
}

/** Kopiert ein Element zurück in die Zwischenablage. */
async function copyBack(item) {
  try {
    await navigator.clipboard.writeText(item.text);
    flashCopied(item.id);
  } catch {
    /* ignoriert */
  }
}

function flashCopied(id) {
  for (const el of listEl.querySelectorAll(".item")) {
    el.classList.toggle("copied", el.dataset.id === id);
  }
  setTimeout(() => {
    for (const el of listEl.querySelectorAll(".item.copied")) el.classList.remove("copied");
  }, 900);
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
  items = [];
  await saveStore();
  render();
});

// ─── Start ────────────────────────────────────────────────────────────────
(async () => {
  await loadStore();
  countEl.textContent = `${items.length} / ${CAPACITY}`;
  render();
  searchEl.focus();
  await harvestClipboard();
})();
