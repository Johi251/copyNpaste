//! Neue Belegungen zuerst reservieren; alte Shortcuts bleiben bei Fehlern erhalten.
use crate::{
    config::{self, Config},
    paste,
};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

pub fn pair(cfg: &Config) -> Result<[Shortcut; 2], String> {
    let paste = cfg
        .paste_shortcut
        .parse::<Shortcut>()
        .map_err(|e| e.to_string())?;
    let toggle = cfg
        .toggle_shortcut
        .parse::<Shortcut>()
        .map_err(|e| e.to_string())?;
    if paste == toggle {
        return Err("Paste- und Toggle-Shortcut dürfen nicht identisch sein.".into());
    }
    Ok([paste, toggle])
}

fn handle(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state != ShortcutState::Released {
        return;
    }
    let Ok([paste_sc, toggle_sc]) = pair(&config::get(app)) else {
        return;
    };
    if *shortcut == paste_sc {
        paste::reverse_paste_async(app.clone());
    } else if *shortcut == toggle_sc {
        crate::toggle_window(app);
    }
}

pub fn prepare(
    app: &AppHandle,
    cfg: &Config,
    old: Option<&Config>,
) -> Result<Vec<Shortcut>, String> {
    let old = old
        .map(pair)
        .transpose()?
        .map(Vec::from)
        .unwrap_or_default();
    let mut added = Vec::new();
    for shortcut in pair(cfg)? {
        if old.contains(&shortcut) {
            continue;
        }
        if let Err(e) = app.global_shortcut().on_shortcut(shortcut, handle) {
            rollback(app, &added);
            return Err(format!("Shortcut konnte nicht registriert werden: {e}"));
        }
        added.push(shortcut);
    }
    Ok(added)
}

pub fn rollback(app: &AppHandle, added: &[Shortcut]) {
    for shortcut in added {
        let _ = app.global_shortcut().unregister(*shortcut);
    }
}

pub fn retire(app: &AppHandle, old: &Config, cfg: &Config) {
    if let (Ok(old), Ok(new)) = (pair(old), pair(cfg)) {
        for shortcut in old {
            if !new.contains(&shortcut) {
                let _ = app.global_shortcut().unregister(shortcut);
            }
        }
    }
}

pub fn register(app: &AppHandle) -> Result<(), String> {
    prepare(app, &config::get(app), None).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gleiche_tasten_trotz_anderer_schreibweise_abgewiesen() {
        let cfg = Config {
            paste_shortcut: "Alt+V".into(),
            toggle_shortcut: "alt+v".into(),
            ..Config::default()
        };
        assert!(pair(&cfg).is_err());
        assert!(pair(&Config::default()).is_ok());
    }
}
