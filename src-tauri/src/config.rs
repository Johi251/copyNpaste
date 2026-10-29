//! Persistente Konfiguration (JSON) im App-Config-Verzeichnis.
//!
//! Alles ist zur Laufzeit änderbar: Magazin-Kapazität, beide Shortcuts,
//! Bridge-Schalter und Bridge-Endpunkt. Shortcuts werden nach dem Speichern
//! neu registriert.

use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::bridge::DEFAULT_BRIDGE_URL;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    /// Magazin-Kapazität (Mindestens 10, per Anforderung; konfigurierbar bis 100).
    pub capacity: usize,
    /// Shortcut für Reverse-Paste (Standard: Alt+V).
    pub paste_shortcut: String,
    /// Shortcut für Magazin zeigen/verstecken (Standard: Alt+Shift+V).
    pub toggle_shortcut: String,
    /// Smart-Bridge ein/aus.
    pub bridge_enabled: bool,
    /// Webhook-Endpunkt (Today's Hunt des Bookmarks Dynamic Managers).
    pub bridge_url: String,
    /// Durchscheinende Fenster-Effekte (backdrop-blur).
    pub effects: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            capacity: 25,
            paste_shortcut: "alt+v".into(),
            toggle_shortcut: "alt+shift+v".into(),
            bridge_enabled: true,
            bridge_url: DEFAULT_BRIDGE_URL.into(),
            effects: true,
        }
    }
}

/// Verwalteter Zustand: aktuell aktive Konfiguration.
pub struct ConfigState(pub RwLock<Config>);

pub fn init(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path(&app)?;
    let cfg: Config = match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|e| {
            eprintln!("copyNpaste: Konfiguration beschädigt ({e}), Standard wird verwendet");
            Config::default()
        }),
        Err(_) => {
            let def = Config::default();
            let _ = fs::write(&path, serde_json::to_string_pretty(&def)?);
            def
        }
    };
    app.manage(ConfigState(RwLock::new(cfg)));
    Ok(())
}

fn config_path(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    fs::create_dir_all(&dir)?;
    Ok(dir.join("config.json"))
}

/// Aktuelle Konfiguration (Kopie).
pub fn get(app: &AppHandle) -> Config {
    let st = app.state::<ConfigState>();
    let guard = st.0.read().unwrap();
    guard.clone()
}

/// Speichert, validiert und aktiviert eine neue Konfiguration.
pub fn save(app: &AppHandle, cfg: Config) -> Result<(), String> {
    // Validierung
    if !(10..=100).contains(&cfg.capacity) {
        return Err("Kapazität muss zwischen 10 und 100 liegen.".into());
    }
    if cfg.paste_shortcut.trim().is_empty() || cfg.toggle_shortcut.trim().is_empty() {
        return Err("Beide Shortcuts müssen gesetzt sein.".into());
    }
    crate::shortcuts::pair(&cfg)?;
    if cfg.bridge_enabled {
        crate::bridge::validate_endpoint(&cfg.bridge_url)?;
    }
    let st = app.state::<ConfigState>();
    let mut current = st.0.write().unwrap();
    let old = current.clone();
    let added = crate::shortcuts::prepare(app, &cfg, Some(&old))?;
    let saved = config_path(app)
        .map_err(|e| e.to_string())
        .and_then(|path| {
            let json = serde_json::to_vec_pretty(&cfg).map_err(|e| e.to_string())?;
            crate::storage::write_atomic(&path, &json).map_err(|e| e.to_string())
        });
    if let Err(e) = saved {
        crate::shortcuts::rollback(app, &added);
        return Err(format!("Konfiguration nicht gespeichert: {e}"));
    }
    *current = cfg.clone();
    drop(current);
    crate::shortcuts::retire(app, &old, &cfg);
    {
        let state = app.state::<crate::state::AppState>();
        crate::state::trim_capacity(&mut state.magazine.lock().unwrap(), cfg.capacity);
    }
    crate::logger::log(app, "Konfiguration gespeichert");
    crate::state::emit_update(app);
    Ok(())
}
