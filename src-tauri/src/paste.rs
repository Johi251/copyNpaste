//! Reverse-Paste: Das Magazin-Spitzen-Element wird in die OS-Zwischenablage
//! geschrieben und anschließend Strg+V (macOS: Cmd+V) simuliert, sodass der
//! Fokus im Ziel-Feld bleibt und der Inhalt sofort eingefügt wird.
//!
use std::time::Duration;

use clipboard_rs::common::{RustImage, RustImageData};
use clipboard_rs::{Clipboard, ClipboardContext};
use tauri::{AppHandle, Manager};

use crate::state::{self, ClipItem, Payload};
use crate::WINDOW_LABEL;

/// Settle-Zeit nach dem Clipboard-Schreiben, bevor Strg+V gesendet wird.
const CLIPBOARD_SETTLE_MS: u64 = 70;
/// Wartezeit nach dem Verstecken des eigenen Fensters (Fokus-Wechsel zurück).
const HIDE_SETTLE_MS: u64 = 140;

/// Reverse-Paste aus dem globalen Shortcut — immer in eigenem Thread,
/// damit der Shortcut-Handler nie blockiert.
pub fn reverse_paste_async(app: AppHandle) {
    std::thread::Builder::new()
        .name("cnp-reverse-paste".into())
        .spawn(move || {
            if let Err(e) = reverse_paste(&app) {
                crate::logger::log_fmt(&app, format!("Paste abgebrochen: {e}"));
                use tauri::Emitter;
                let _ = app.emit("operation-error", e);
            }
        })
        .ok();
}

/// Führt den Reverse-Paste aus (Magazin-Spitze).
pub fn reverse_paste(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<state::AppState>();
    let _paste = state
        .paste_lock
        .lock()
        .map_err(|_| "Paste-Sperre beschädigt")?;
    hide_own_window_if_visible(app)?;
    let target = crate::target::inspect();
    if target.password {
        return Err("Kein automatisches Einfügen in Passwortfelder.".into());
    }
    let item = state::select_for_paste(&state.magazine.lock().unwrap(), target.text_only)
        .ok_or("Kein passendes ungepinntes Element im Magazin.")?;
    paste_item_internal(app, &item)
}

pub fn paste_by_id(app: &AppHandle, id: &str) -> Result<(), String> {
    let state = app.state::<state::AppState>();
    let _paste = state
        .paste_lock
        .lock()
        .map_err(|_| "Paste-Sperre beschädigt")?;
    hide_own_window_if_visible(app)?;
    let target = crate::target::inspect();
    let item = state
        .magazine
        .lock()
        .unwrap()
        .iter()
        .find(|i| i.id.to_string() == id)
        .cloned()
        .ok_or("Element nicht mehr im Magazin.")?;
    if target.password || (target.text_only && !matches!(item.payload, Payload::Text(_))) {
        return Err(
            "Dieses Element passt nicht in das ausgewählte Textfeld; es bleibt im Magazin.".into(),
        );
    }
    paste_item_internal(app, &item)
}

fn paste_item_internal(app: &AppHandle, item: &ClipItem) -> Result<(), String> {
    state::announce_self_write(app, item.fingerprint);
    let result = write_to_clipboard(&item.payload).and_then(|_| {
        std::thread::sleep(Duration::from_millis(CLIPBOARD_SETTLE_MS));
        send_paste_keystroke()
    });
    let state = app.state::<state::AppState>();
    if result.is_err() {
        state
            .ignore_next
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
    let outcome = finish_paste(&mut state.magazine.lock().unwrap(), item.id, result);
    state::emit_update(app);
    outcome
}

/// Nur ein erfolgreich versendeter Paste entfernt den Eintrag.
fn finish_paste(
    magazine: &mut std::collections::VecDeque<ClipItem>,
    id: uuid::Uuid,
    result: Result<(), String>,
) -> Result<(), String> {
    result?;
    if let Some(pos) = magazine.iter().position(|i| i.id == id) {
        magazine.remove(pos);
    }
    Ok(())
}

/// Versteckt das eigene Fenster, falls sichtbar, damit der Fokus vor dem
/// simulierten Strg+V zum eigentlichen Zielfenster zurückkehrt.
fn hide_own_window_if_visible(app: &AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(WINDOW_LABEL) {
        if win.is_visible().unwrap_or(false) {
            win.hide().map_err(|e| e.to_string())?;
            if !crate::target::restore() {
                return Err(
                    "Zielfenster konnte nicht aktiviert werden; Element bleibt erhalten.".into(),
                );
            }
            std::thread::sleep(Duration::from_millis(HIDE_SETTLE_MS));
        }
    }
    Ok(())
}

/// Schreibt das Element in die OS-Zwischenablage (eigener Kontext pro Aufruf).
fn write_to_clipboard(payload: &Payload) -> Result<(), String> {
    let ctx = ClipboardContext::new().map_err(|e| format!("{e:?}"))?;
    match payload {
        Payload::Text(t) => ctx.set_text(t.clone()).map_err(|e| format!("{e:?}"))?,
        Payload::Image { png, .. } => {
            let img = RustImageData::from_bytes(png).map_err(|e| format!("{e:?}"))?;
            ctx.set_image(img).map_err(|e| format!("{e:?}"))?
        }
        Payload::Files(f) => ctx.set_files(f.clone()).map_err(|e| format!("{e:?}"))?,
    }
    Ok(())
}

/// Simuliert Strg+V (Windows/Linux) bzw. Cmd+V (macOS) im fokussierten Fenster.
fn send_paste_keystroke() -> Result<(), String> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Tastatureingabe nicht verfügbar: {e}"))?;
    let modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };
    enigo
        .key(modifier, Direction::Press)
        .map_err(|e| e.to_string())?;
    let pasted = enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| e.to_string());
    let released = enigo
        .key(modifier, Direction::Release)
        .map_err(|e| e.to_string());
    pasted.and(released)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fehler_behaelt_inhalt_erfolg_entnimmt_genau_ein_element() {
        let a = ClipItem::new(Payload::Text("  code\n".into()));
        let b = ClipItem::new(Payload::Text("other".into()));
        let mut mag = std::collections::VecDeque::from([a.clone(), b.clone()]);
        assert!(finish_paste(&mut mag, a.id, Err("clipboard busy".into())).is_err());
        assert_eq!(mag.len(), 2);
        assert!(finish_paste(&mut mag, a.id, Ok(())).is_ok());
        assert_eq!(mag.front().unwrap().id, b.id);
    }
}
