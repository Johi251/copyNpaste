//! Reverse-Paste: Das Magazin-Spitzen-Element wird in die OS-Zwischenablage
//! geschrieben und anschließend Strg+V (macOS: Cmd+V) simuliert, sodass der
//! Fokus im Ziel-Feld bleibt und der Inhalt sofort eingefügt wird.
//!
//! Ablauf:
//!   1. Eigenes Fenster sichtbar? → verstecken, kurz warten (Fokus kehr zurück)
//!   2. Element der Spitze entnehmen (pop) — es ist damit aus dem Magazin entfernt
//!   3. Inhalt in die Zwischenablage schreiben (mit Selbst-Reflektions-Schutz)
//!   4. Kurze Settle-Pause, dann Paste-Tastendruck simulieren
//!   5. UI-Update senden — das vorletzte Element ist neue Paste-Spitze

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
            reverse_paste(&app);
        })
        .ok();
}

/// Führt den Reverse-Paste aus (Magazin-Spitze).
pub fn reverse_paste(app: &AppHandle) {
    hide_own_window_if_visible(app);
    match state::pop_last(app) {
        Some(item) => paste_item_internal(app, item),
        None => {
            crate::logger::log(app, "Reverse-Paste: Magazin ist leer — nichts zu pasten");
            state::emit_update(app);
        }
    }
}

/// Fügt ein bestimmtes Element per ID ein (Klick/Enter in der UI).
pub fn paste_by_id(app: &AppHandle, id: &str) -> Result<(), String> {
    hide_own_window_if_visible(app);
    match state::remove_by_id(app, id) {
        Some(item) => {
            paste_item_internal(app, item);
            Ok(())
        }
        None => Err("Element nicht mehr im Magazin.".into()),
    }
}

fn paste_item_internal(app: &AppHandle, item: ClipItem) {
    let kind = match &item.payload {
        Payload::Text(t) => format!("Text ({} Zeichen)", t.chars().count()),
        Payload::Image { width, height, .. } => format!("Bild ({width}x{height})"),
        Payload::Files(f) => format!("Dateien ({})", f.len()),
    };
    crate::logger::log_fmt(app, format!("Reverse-Paste: {kind} wird eingefuegt"));

    // Selbst-Reflektion anmelden, BEVOR geschrieben wird.
    state::announce_self_write(app, item.fingerprint);

    if write_to_clipboard(&item.payload).is_err() {
        crate::logger::log(app, "FEHLER: Clipboard konnte nicht geschrieben werden — Paste abgebrochen");
        return;
    }

    std::thread::sleep(Duration::from_millis(CLIPBOARD_SETTLE_MS));
    send_paste_keystroke();
}

/// Versteckt das eigene Fenster, falls sichtbar, damit der Fokus vor dem
/// simulierten Strg+V zum eigentlichen Zielfenster zurückkehrt.
fn hide_own_window_if_visible(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(WINDOW_LABEL) {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
            std::thread::sleep(Duration::from_millis(HIDE_SETTLE_MS));
        }
    }
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
fn send_paste_keystroke() {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    let Ok(mut enigo) = Enigo::new(&Settings::default()) else {
        crate::logger::log_static("FEHLER: Enigo konnte nicht initialisiert werden");
        return;
    };

    let modifier = if cfg!(target_os = "macos") { Key::Meta } else { Key::Control };
    let _ = enigo.key(modifier, Direction::Press);
    let _ = enigo.key(Key::Unicode('v'), Direction::Click);
    let _ = enigo.key(modifier, Direction::Release);
}
