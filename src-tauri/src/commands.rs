//! Tauri-IPC-Befehle — die Brücke zwischen Frontend und Rust-Backend.

use tauri::AppHandle;

use crate::{bridge, config, paste, state};

/// Aktueller Magazin-Zustand (neueste zuerst).
#[tauri::command]
pub fn magazine_state(app: AppHandle) -> Vec<state::ItemLite> {
    state::snapshot(&app)
}

/// Fügt ein bestimmtes Element per ID ein (Klick/Enter in der UI).
#[tauri::command]
pub fn paste_item(app: AppHandle, id: String) -> Result<(), String> {
    paste::paste_by_id(&app, &id)
}

/// Entfernt ein Element, ohne es einzufügen.
#[tauri::command]
pub fn delete_item(app: AppHandle, id: String) -> Result<(), String> {
    state::remove_by_id(&app, &id)
        .map(|_| ())
        .ok_or_else(|| "Element nicht gefunden.".to_string())
}

/// Pin umschalten: gepinnte Elemente bleiben beim Auto-Rauswurf und werden
/// von Alt+V übersprungen (nur gezielt per Klick/Enter einfügbar).
#[tauri::command]
pub fn toggle_pin(app: AppHandle, id: String) -> Result<bool, String> {
    state::toggle_pin(&app, &id)
}

/// Leert das gesamte Magazin.
#[tauri::command]
pub fn clear_magazine(app: AppHandle) {
    state::clear(&app);
}

/// Aktuelle Konfiguration.
#[tauri::command]
pub fn get_config(app: AppHandle) -> config::Config {
    config::get(&app)
}

/// Speichert + aktiviert eine neue Konfiguration (validiert Shortcuts).
#[tauri::command]
pub fn save_config(app: AppHandle, config: config::Config) -> Result<(), String> {
    config::save(&app, config)
}

/// Testet den Bridge-Endpunkt aus dem Einstellungen-Dialog.
#[tauri::command]
pub fn test_bridge(app: AppHandle) -> Result<String, String> {
    bridge::probe(&app)
}
