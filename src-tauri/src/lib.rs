//! copyNpaste — globales Clipboard-Magazin mit Reverse-Paste.
//!
//! Architektur:
//! - `state`     : Magazin-Stack (VecDeque), ClipItem, Fingerprint gegen Selbst-Reflektion
//! - `monitor`   : OS-Clipboard-Watcher (clipboard-rs) als Hintergrund-Thread
//! - `paste`     : Reverse-Paste (Alt+V): Clipboard setzen + Ctrl+V/Cmd+V simulieren (enigo)
//! - `bridge`    : URL-Regex-Filter + asynchroner Localhost-Webhook (Today's Hunt)
//! - `config`    : persistente Konfiguration (JSON im App-Config-Verzeichnis)
//! - `shortcuts` : Registrierung der globalen Shortcuts (Reverse-Paste + Fenster-Toggle)
//! - `commands`  : Tauri-IPC-Befehle für das Frontend
//! - `tray`      : System-Tray mit Menü (Hintergrund-Dienst-Charakter)
//! - `logger`    : minimalistische Datei-Protokollierung für Nachweise

pub mod bridge;
pub mod commands;
pub mod config;
pub mod logger;
pub mod monitor;
pub mod paste;
pub mod shortcuts;
pub mod state;
pub mod tray;

use tauri::Manager;

/// Fester Name des Magazin-Fensters.
pub const WINDOW_LABEL: &str = "magazine";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::magazine_state,
            commands::paste_item,
            commands::delete_item,
            commands::toggle_pin,
            commands::clear_magazine,
            commands::get_config,
            commands::save_config,
            commands::test_bridge
        ])
        .setup(|app| {
            logger::init(app.handle().clone())?;
            state::init(app.handle().clone())?;
            config::init(app.handle().clone())?;

            logger::log(app.handle(), "copyNpaste v1.1.0 startet");

            let cfg = config::get(app.handle());
            logger::log_fmt(
                app.handle(),
                format!(
                    "Konfiguration geladen: Kapazitaet={}, Paste={}, Toggle={}, Bridge={} ({})",
                    cfg.capacity,
                    cfg.paste_shortcut,
                    cfg.toggle_shortcut,
                    cfg.bridge_enabled,
                    cfg.bridge_url
                ),
            );

            shortcuts::register(app.handle())?;
            monitor::start(app.handle().clone())?;
            tray::build(app.handle())?;

            // Fenster startet unsichtbar: Hintergrund-Dienst im Tray.
            Ok(())
        })
        .on_window_event(|window, event| {
            // Schließen (X) versteckt das Fenster nur — der Tray-Dienst läuft weiter.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == WINDOW_LABEL {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("copyNpaste: Fehler beim Starten der Anwendung");
}

/// Zeigt/versteckt das Magazin-Fenster (Tray-Klick oder Toggle-Shortcut).
pub fn toggle_window(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window(WINDOW_LABEL) {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
            logger::log(app, "Magazin-Fenster versteckt (Toggle)");
        } else {
            let _ = win.show();
            let _ = win.set_focus();
            logger::log(app, "Magazin-Fenster angezeigt (Toggle)");
        }
    }
}
