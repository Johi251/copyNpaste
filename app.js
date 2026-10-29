(() => {
  "use strict";
  const CAPACITY = 25;
  const STORE_KEY = "copynpaste-web-magazine";
  const urlRe = /^https?:\/\/[^\s]+$/i;

  const $ = (id) => document.getElementById(id);
  let items = [];   // { id, kind: "text"|"image", text, dataUrl?, at }
  let filter = "";

  let db;
  let pending = Promise.resolve();
  function error(e) { $("error").textContent = String(e); }
  function save() {
    return new Promise((resolve, reject) => {
      const tx = db.transaction("state", "readwrite");
      tx.objectStore("state").put(items, "magazine");
      tx.oncomplete = resolve;
      tx.onerror = () => reject(tx.error);
      tx.onabort = () => reject(tx.error ?? new Error("Speichern abgebrochen"));
    });
  }
  function update(change) {
    pending = pending.catch(() => {}).then(async () => {
      const previous = items.slice();
      change();
      try { await save(); render(); $("error").textContent = ""; }
      catch (e) { items = previous; render(); throw e; }
    });
    return pending;
  }
  async function load() {
    db = await new Promise((resolve, reject) => {
      const request = indexedDB.open("copynpaste", 1);
      request.onupgradeneeded = () => request.result.createObjectStore("state");
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
    const stored = await new Promise((resolve, reject) => {
      const request = db.transaction("state").objectStore("state").get("magazine");
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
    const values = stored ?? JSON.parse(localStorage.getItem(STORE_KEY) ?? "[]");
    if (!Array.isArray(values)) throw new Error("Ungültiger gespeicherter Magazin-Inhalt");
    items = values.filter(i => i && typeof i.id === "string" && typeof i.text === "string" &&
      (i.kind === "text" || (i.kind === "image" && typeof i.dataUrl === "string" && i.dataUrl.startsWith("data:image/")))).slice(0, CAPACITY);
    await save();
  }

  function fmtTime(ts) {
    const d = new Date(ts), now = new Date();
    const hm = `${String(d.getHours()).padStart(2,"0")}:${String(d.getMinutes()).padStart(2,"0")}`;
    return d.toDateString() === now.toDateString() ? hm : `${String(d.getDate()).padStart(2,"0")}.${String(d.getMonth()+1).padStart(2,"0")}. ${hm}`;
  }

  function push(item) {
    return update(() => {
      if (items[0] && items[0].kind === item.kind && items[0].text === item.text && items[0].dataUrl === item.dataUrl) return;
      items.unshift(item);
      items = items.slice(0, CAPACITY);
    });
  }

  // ── Aufnahme ──
  function harvestText(text) {
    text = text ?? "";
    if (!text.trim() || text.length > 200000) return;
    return push({ id: crypto.randomUUID(), kind: "text", text, at: Date.now() });
  }
  async function harvestImage(file) {
    if (!file.type.startsWith("image/") || file.size > 20 * 1024 * 1024) throw new Error("Bild muss kleiner als 20 MiB sein.");
    const url = await new Promise((res, reject) => { const r = new FileReader(); r.onload = () => res(r.result); r.onerror = () => reject(r.error); r.readAsDataURL(file); });
    return push({ id: crypto.randomUUID(), kind: "image", text: file.name || file.type, dataUrl: url, at: Date.now() });
  }

  document.addEventListener("paste", (e) => {
    if (e.target instanceof Element && e.target.closest("input,textarea,[contenteditable]")) return;
    const cd = e.clipboardData; if (!cd) return;
    const file = [...(cd.files ?? [])].find((f) => f.type.startsWith("image/"));
    if (file) { void harvestImage(file).catch(error); e.preventDefault(); return; }
    const text = cd.getData("text/plain");
    if (text) { void Promise.resolve(harvestText(text)).catch(error); e.preventDefault(); }
  });

  $("btn-read").addEventListener("click", async () => {
    try { await harvestText(await navigator.clipboard.readText()); } catch (e) { error(e); }
  });

  // ── Zurückkopieren ──
  async function copyBack(item) {
    try {
      if (item.kind === "text") {
        await navigator.clipboard.writeText(item.text);
      } else {
        const blob = await (await fetch(item.dataUrl)).blob();
        await navigator.clipboard.write([new ClipboardItem({ [blob.type]: blob })]);
      }
      await update(() => { items = items.filter(i => i.id !== item.id); });
    } catch (e) { error(e); }
  }
  // ── Rendering ──
  function render() {
    const visible = filter ? items.filter((i) => (i.text ?? "").toLowerCase().includes(filter)) : items;
    const texts = visible.filter((i) => i.kind === "text");
    const media = visible.filter((i) => i.kind !== "text");
    for (const [el, list] of [[$("list-text"), texts], [$("list-media"), media]]) {
      el.innerHTML = "";
      if (list.length === 0) {
        const d = document.createElement("div");
        d.className = "empty";
        d.innerHTML = filter ? "<b>Keine Treffer</b>Suchbegriff anpassen" : "<b>Noch leer</b>Strg + V auf dieser Seite";
        el.append(d);
        continue;
      }
      for (const item of list) {
        const card = document.createElement("div");
        card.className = "card"; card.dataset.id = item.id;
        card.tabIndex = 0; card.setAttribute("role", "button");
        card.setAttribute("aria-label", "Eintrag entnehmen: " + item.text.slice(0, 80));
        card.addEventListener("keydown", e => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); void copyBack(item); } });
        const isUrl = item.kind === "text" && urlRe.test(item.text);
        const meta = document.createElement("div");
        meta.className = "meta";
        meta.innerHTML = `<span>${fmtTime(item.at)}</span>${isUrl ? '<span class="tag">URL</span>' : ""}<span>${item.kind === "text" ? item.text.length + " Zeichen" : "Bild"}</span>`;
        const body = document.createElement("div");
        body.className = "body" + (isUrl ? " url" : "");
        if (item.kind === "image") {
          body.className = "";
          const img = document.createElement("img");
          img.src = item.dataUrl; img.alt = item.text;
          body.append(img);
        } else {
          body.textContent = item.text;
        }
        const check = document.createElement("span");
        check.className = "check"; check.textContent = "✓";
        card.append(meta, body, check);
        card.addEventListener("click", () => void copyBack(item));
        el.append(card);
      }
    }
    $("n-text").textContent = texts.length;
    $("n-media").textContent = media.length;
    $("count").textContent = `${items.length} / ${CAPACITY}`;
    $("fill").style.width = `${Math.min(100, items.length / CAPACITY * 100)}%`;
  }

  $("search").addEventListener("input", (e) => { filter = e.target.value.trim().toLowerCase(); render(); });
  $("search").addEventListener("keydown", (e) => {
    if (e.key !== "Enter") return;
    const visible = filter ? items.filter((i) => (i.text ?? "").toLowerCase().includes(filter)) : items;
    if (visible[0]) void copyBack(visible[0]);
  });
  $("btn-clear").addEventListener("click", () => { void update(() => { items = []; }).catch(error); });
  $("btn-pop").addEventListener("click", () => { if (items[0]) void copyBack(items[0]); });
  document.addEventListener("keydown", e => {
    if (e.altKey && e.key.toLowerCase() === "v") { e.preventDefault(); if (items[0]) void copyBack(items[0]); }
  });

  const zone = $("paste-zone");
  ["dragover", "dragenter"].forEach((t) => zone.addEventListener(t, (e) => { e.preventDefault(); zone.classList.add("drag"); }));
  ["dragleave", "drop"].forEach((t) => zone.addEventListener(t, () => zone.classList.remove("drag")));
  zone.addEventListener("drop", (e) => {
    e.preventDefault();
    const file = [...(e.dataTransfer?.files ?? [])].find((f) => f.type.startsWith("image/"));
    if (file) void harvestImage(file).catch(error);
  });

  load().then(render).catch(error);
})();
