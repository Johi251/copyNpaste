//! Registrierung der globalen Shortcuts (Reverse-Paste + Fenster-Toggle).
//! Shortcuts sind konfigurierbar; `register` kann jederzeit erneut aufgerufen
//! werden (hebt alte Registrierungen zuerst auf).

use tauri::AppHandle;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::{config, logger, paste};

/// Registriert beide Shortcuts aus der aktuellen Konfiguration neu.
pub fn register(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    let cfg = config::get(app);

    let app_for_paste = app.clone();
    let paste_sc = cfg.paste_shortcut.clone();
    gs.on_shortcut(paste_sc.as_str(), move |_app, _shortcut, event| {
        use tauri_plugin_global_shortcut::ShortcutState;
        if event.state == ShortcutState::Pressed {
            paste::reverse_paste_async(app_for_paste.clone());
        }
    })?;

    let toggle_sc = cfg.toggle_shortcut.clone();
    gs.on_shortcut(toggle_sc.as_str(), move |app, _shortcut, event| {
        use tauri_plugin_global_shortcut::ShortcutState;
        if event.state == ShortcutState::Pressed {
            crate::toggle_window(app);
        }
    })?;

    logger::log_fmt(
        app,
        format!("Shortcuts registriert: {} (Reverse-Paste), {} (Magazin-Toggle)", paste_sc, toggle_sc),
    );
    Ok(())
}

/// Prüft einen Shortcut-String, ohne ihn zu registrieren.
pub fn validate(spec: &str) -> Result<(), tauri_plugin_global_shortcut::Error> {
    use tauri_plugin_global_shortcut::Shortcut;
    let _parsed: Shortcut = spec.parse()?;
    Ok(())
}
