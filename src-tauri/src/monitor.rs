//! Der OS-Clipboard-Watcher: läuft als eigener Thread und meldet jede
//! Änderung der Zwischenablage — egal ob per Strg+C, Cmd+C, Rechtsklick oder
//! Drag&Drop. Das ist die robustere Umsetzung des "globalen Abfangens von
//! Kopier-Befehlen": nicht die Taste wird abgefangen, sondern das Ergebnis
//! jeder Kopieraktion landet zuverlässig im Magazin.

use clipboard_rs::common::RustImage;
use clipboard_rs::{Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext};
use tauri::AppHandle;

use crate::state::{self, ClipItem, Payload};

/// Startet den Watcher-Thread. Kehrt sofort zurück.
pub fn start(app: AppHandle) -> Result<(), String> {
    struct ChangeHandler {
        app: AppHandle,
        ctx: ClipboardContext,
    }

    impl ClipboardHandler for ChangeHandler {
        fn on_clipboard_change(&mut self) {
            if is_privacy_excluded(&self.ctx) {
                crate::logger::log(&self.app, "Datenschutz: Kopie uebergangen (Passwort-Manager-Marker)");
                return;
            }
            match capture(&self.ctx) {
                Some(item) => state::ingest(&self.app, item),
                None => {
                    // Leere clipboard content types interessieren nicht.
                }
            }
        }
    }

    let handler = ChangeHandler {
        app: app.clone(),
        ctx: ClipboardContext::new().map_err(|e| format!("Clipboard-Kontext: {e:?}"))?,
    };
    let mut watcher = ClipboardWatcherContext::new().map_err(|e| format!("Clipboard-Watcher: {e:?}"))?;
    watcher.add_handler(handler);

    std::thread::Builder::new()
        .name("cnp-clipboard-watch".into())
        .spawn(move || watcher.start_watch())
        .map_err(|e| format!("Watcher-Thread: {e:?}"))?;

    crate::logger::log(&app, "Clipboard-Watcher gestartet (lauert auf jede Aenderung der Zwischenablage)");
    Ok(())
}

/// Liest den aktuellen Clipboard-Inhalt. Priorität: Dateien > Bild > Text
/// (Kopieraktionen setzen mehrere Formate; die höchstwertige gewinnt).
/// Datenschutz: Inhalte mit Passwort-Manager-Ausschluss-Markern werden
/// absichtlich ÜBERGANGEN (nie erfasst, nie gespeichert, nie über die Bridge gesendet).
fn capture(ctx: &ClipboardContext) -> Option<ClipItem> {
    // 1) Datenschutz-Marker prüfen (Windows-Konvention, gesetzt von
    //    Bitwarden, KeePass, 1Password & Co.):
    if is_privacy_excluded(ctx) {
        return None;
    }

    // 2) Dateiliste (z. B. Strg+C im Explorer)
    if let Ok(files) = ctx.get_files() {
        if !files.is_empty() {
            return Some(ClipItem::new(Payload::Files(files)));
        }
    }

    // 3) Bild (z. B. Screenshot, Bild aus Browser/Grafikprogramm)
    if let Ok(img) = ctx.get_image() {
        if !img.is_empty() {
            let (width, height) = img.get_size();
            if let Ok(png) = img.to_png() {
                let bytes = png.get_bytes().to_vec();
                if !bytes.is_empty() && bytes.len() <= MAX_IMAGE_BYTES {
                    return Some(ClipItem::new(Payload::Image { png: bytes, width, height }));
                }
            }
        }
    }

    // 4) Text (reiner Text, ggf. mit Formatierung — nur der Plain-Text zählt)
    if let Ok(text) = ctx.get_text() {
        let t = text.trim_end_matches('\0').trim().to_string();
        if !t.is_empty() && t.len() <= MAX_TEXT_BYTES {
            return Some(ClipItem::new(Payload::Text(t)));
        }
    }

    None
}

/// Schutzgrenzen: pro Element maximal diese Datenmengen erfassen.
const MAX_TEXT_BYTES: usize = 5 * 1024 * 1024;
const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;

/// Erkennt Windows-Clipboard-Marker, mit denen Passwort-Manager sensible
/// Kopien von der Verarbeitung durch Clipboard-Tools ausschließen lassen.
fn is_privacy_excluded(ctx: &ClipboardContext) -> bool {
    let Ok(formats) = ctx.available_formats() else {
        return false;
    };
    let has = |name: &str| {
        formats
            .iter()
            .any(|f| f.eq_ignore_ascii_case(name))
    };

    // Expliziter Ausschluss-Marker (Windows 10+)
    if has("ExcludeClipboardContentFromMonitorProcessing") {
        return true;
    }
    // Historischer Marker (ältere Tools)
    if has("Clipboard Viewer Ignore") {
        return true;
    }
    // Windows-Clipboard-History-Marker: Wert 0 (=DWORD 0) heißt „nicht aufnehmen“
    if has("CanIncludeInClipboardHistory") {
        if let Ok(buf) = ctx.get_buffer("CanIncludeInClipboardHistory") {
            if buf.len() >= 4 && buf[..4] == [0, 0, 0, 0] {
                return true;
            }
        }
    }
    false
}
