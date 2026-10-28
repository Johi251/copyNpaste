/** copyNpaste — Frontend-Einstieg: Zustand, Events, Rendering, Aktionen. */
import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import "./styles.css";

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { closeSettings, openSettings, saveSettings, testBridge } from "./settings";
import type { AppConfig, ItemLite } from "./types";
import { icons, logoSvg, nextPasteId, renderColumn } from "./ui";

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

const appWindow = getCurrentWindow();

let items: ItemLite[] = [];
let config: AppConfig | null = null;
let filter = "";
/** ID des Elements, das Enter in der Suche (Quick-Paste) einfügen würde. */
let quickPasteId: string | undefined;

const cardActions = {
  onPaste: (id: string) => void invoke("paste_item", { id }).catch(console.error),
  onDelete: (id: string) => void invoke("delete_item", { id }).catch(console.error),
  onPin: (id: string) => void invoke("toggle_pin", { id }).catch(console.error),
};

// ─── Rendering ───────────────────────────────────────────────────────────
function matches(item: ItemLite): boolean {
  if (!filter) return true;
  const needle = filter.toLowerCase();
  return (
    item.preview.toLowerCase().includes(needle)
    || (item.textPreview?.toLowerCase().includes(needle) ?? false)
    || item.meta.toLowerCase().includes(needle)
  );
}

function render(): void {
  const visible = items.filter(matches);
  const texts = visible.filter((i) => i.kind === "text");
  const media = visible.filter((i) => i.kind !== "text");
  const nextId = nextPasteId(visible);

  // Quick-Paste: Enter fügt den obersten sichtbaren ungepinnten Treffer ein
  // (zuerst Text-Spalte, sonst Bild/Datei-Spalte).
  quickPasteId = nextId;

  renderColumn($("list-text"), texts, "text", cardActions, nextId);
  renderColumn($("list-media"), media, "media", cardActions, nextId);

  $("count-text").textContent = String(texts.length);
  $("count-media").textContent = String(media.length);

  // Füllstand
  const cap = config?.capacity ?? 25;
  $("fill-label").textContent = `${items.length} / ${cap}`;
  $("fill-bar").style.width = `${Math.min(100, (items.length / cap) * 100)}%`;

  // Bridge-Status anhand des jüngsten URL-Elements
  const lastUrl = items.find((i) => i.isUrl);
  const dot = $("bridge-dot");
  const label = $("bridge-label");
  dot.classList.remove("b-on", "b-off", "b-disabled");
  if (!config?.bridgeEnabled) {
    dot.classList.add("b-disabled");
    label.textContent = "Bridge aus";
  } else if (lastUrl?.bridge === "sent") {
    dot.classList.add("b-on");
    label.textContent = "Bridge aktiv";
  } else if (lastUrl?.bridge === "endpoint-offline") {
    dot.classList.add("b-off");
    label.textContent = "Bridge: Endpunkt offline";
  } else {
    label.textContent = "Bridge bereit";
  }
}

function applyEffects(): void {
  document.body.classList.toggle("no-effects", config ? !config.effects : false);
}

// ─── Initialisierung ─────────────────────────────────────────────────────
async function bootstrap(): Promise<void> {
  // Statische Icons einsetzen
  $("brand-logo").innerHTML = logoSvg;
  $("btn-settings").innerHTML = icons.settings;
  $("btn-minimize").innerHTML = icons.minimize;
  $("btn-close").innerHTML = icons.close;
  $("search-icon").innerHTML = icons.search;
  $("col-icon-text").innerHTML = icons.text;
  $("col-icon-media").innerHTML = icons.media;

  // Konfiguration + Magazin laden
  try {
    config = await invoke<AppConfig>("get_config");
  } catch (e) {
    console.error("Konfiguration nicht ladbar:", e);
  }
  applyEffects();

  try {
    items = await invoke<ItemLite[]>("magazine_state");
  } catch (e) {
    console.error("Magazin-Zustand nicht ladbar:", e);
  }
  render();

  // Live-Updates aus dem Rust-Backend
  await listen<ItemLite[]>("magazine-changed", (event) => {
    items = event.payload;
    render();
  });

  bindUi();
}

// ─── UI-Aktionen ─────────────────────────────────────────────────────────
function bindUi(): void {
  $("btn-minimize").addEventListener("click", () => void appWindow.minimize());
  // „Schließen" versteckt nur — der Tray-Dienst läuft weiter.
  $("btn-close").addEventListener("click", () => void appWindow.hide());

  $("btn-settings").addEventListener("click", () => {
    if (config) openSettings(config);
  });
  $("btn-settings-close").addEventListener("click", closeSettings);
  $("btn-settings-cancel").addEventListener("click", closeSettings);
  $("btn-test-bridge").addEventListener("click", () => void testBridge());

  $("btn-settings-save").addEventListener("click", async () => {
    const saved = await saveSettings();
    if (saved) {
      config = saved;
      applyEffects();
      render();
      closeSettings();
    }
  });

  $("btn-clear").addEventListener("click", () => {
    void invoke("clear_magazine").catch(console.error);
  });

  const search = $<HTMLInputElement>("search");
  search.addEventListener("input", () => {
    filter = search.value.trim();
    render();
  });

  // Quick-Paste: Enter in der Suche fügt den ersten Treffer ein (Raycast-Flow).
  search.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && quickPasteId) {
      e.preventDefault();
      void invoke("paste_item", { id: quickPasteId }).catch(console.error);
    }
  });

  // Fenster gezeigt/aktiviert → Suche fokussieren (sofort tippen können).
  void appWindow
    .onFocusChanged(({ payload }) => {
      if (payload) {
        search.focus();
        search.select();
      }
    })
    .catch(console.error);

  // Esc: Fenster verstecken · über Overlay: schließen
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      if (!$("settings-overlay").classList.contains("hidden")) closeSettings();
      else void appWindow.hide();
    }
  });

  // ↑/↓: zwischen Karten der jeweiligen Spalte navigieren
  document.addEventListener("keydown", (e) => {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    if (document.activeElement?.classList.contains("cnp-card")) return;
    const first = document.querySelector<HTMLElement>(".cnp-card");
    if (first) {
      e.preventDefault();
      first.focus();
    }
  });
}

void bootstrap();
