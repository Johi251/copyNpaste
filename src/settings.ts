/** Einstellungen-Dialog: laden, validieren, speichern, Bridge-Test. */
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "./types";

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

/** Öffnet den Dialog mit der aktuellen Konfiguration. */
export function openSettings(cfg: AppConfig): void {
  $<HTMLInputElement>("set-capacity").value = String(cfg.capacity);
  $<HTMLInputElement>("set-paste-sc").value = cfg.pasteShortcut;
  $<HTMLInputElement>("set-toggle-sc").value = cfg.toggleShortcut;
  $<HTMLInputElement>("set-bridge-url").value = cfg.bridgeUrl;
  $<HTMLInputElement>("set-bridge-on").checked = cfg.bridgeEnabled;
  $<HTMLInputElement>("set-effects").checked = cfg.effects;
  hideError();
  $("bridge-test-result").textContent = "";
  $("settings-overlay").classList.remove("hidden");
  $("settings-overlay").classList.add("flex");
}

export function closeSettings(): void {
  $("settings-overlay").classList.add("hidden");
  $("settings-overlay").classList.remove("flex");
}

function showError(msg: string): void {
  const el = $("settings-error");
  el.textContent = msg;
  el.classList.remove("hidden");
}

function hideError(): void {
  $("settings-error").classList.add("hidden");
}

/** Sammelt die Formularwerte und speichert sie über das Rust-Backend. */
export async function saveSettings(): Promise<AppConfig | null> {
  hideError();
  const cfg: AppConfig = {
    capacity: Number($<HTMLInputElement>("set-capacity").value),
    pasteShortcut: $<HTMLInputElement>("set-paste-sc").value.trim().toLowerCase(),
    toggleShortcut: $<HTMLInputElement>("set-toggle-sc").value.trim().toLowerCase(),
    bridgeEnabled: $<HTMLInputElement>("set-bridge-on").checked,
    bridgeUrl: $<HTMLInputElement>("set-bridge-url").value.trim(),
    effects: $<HTMLInputElement>("set-effects").checked,
  };
  try {
    await invoke("save_config", { config: cfg });
    return cfg;
  } catch (e) {
    showError(String(e));
    return null;
  }
}

/** Testet den Bridge-Endpunkt synchron (mit Timeout im Backend). */
export async function testBridge(): Promise<void> {
  const out = $("bridge-test-result");
  out.textContent = "Teste …";
  out.className = "mt-2 text-[11px] text-neutral-500";
  try {
    const msg = await invoke<string>("test_bridge", { endpoint: $<HTMLInputElement>("set-bridge-url").value.trim() });
    out.textContent = `✓ ${msg}`;
    out.className = "mt-2 text-[11px] text-teal-400";
  } catch (e) {
    out.textContent = `✗ ${String(e)}`;
    out.className = "mt-2 text-[11px] text-amber-500";
  }
}
