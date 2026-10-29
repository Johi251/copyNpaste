//! Magazin-Zustand: der Stack aller kopierten Elemente.
//!
//! Das Magazin ist eine VecDeque — `push_back` = neu kopiert, `pop_back` =
//! Reverse-Paste entnimmt die Spitze (zuletzt kopiert). Bei Überkapazität wird
//! das älteste *ungepinnte* Element (vorne) verworfen — Gepinnte bleiben.
//! Der komplette Magazin-Stand wird bei jeder Änderung persistent gespeichert
//! und beim Start wiederhergestellt (Magazin überlebt Neustarts).

use std::collections::VecDeque;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Mutex};

use base64::Engine as _;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

/// Tauri-Event-Name für Magazin-Änderungen (Frontend lauscht darauf).
pub const EVENT_CHANGED: &str = "magazine-changed";

// ---------------------------------------------------------------------------
// Datenmodell
// ---------------------------------------------------------------------------

/// Der Inhalt eines Magazin-Platzes.
#[derive(Clone, Debug)]
pub enum Payload {
    Text(String),
    /// PNG-kodiert, damit große Screenshots wenig RAM belegen.
    Image {
        png: Vec<u8>,
        width: u32,
        height: u32,
    },
    Files(Vec<String>),
}

/// Übermittlungsstatus einer erkannten URL an die Smart-Bridge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BridgeState {
    /// Aus älteren Sitzungen ist keine Zustellbestätigung vorhanden.
    Unknown,
    /// URL erkannt, Versand läuft.
    Pending,
    /// Webhook erfolgreich zugestellt.
    Sent,
    /// Endpunkt nicht erreichbar — Magazin unberührt, kein Blockieren.
    EndpointOffline,
    /// Bridge in der Konfiguration deaktiviert.
    Disabled,
}

/// Eine erkannte URL samt Bridge-Status.
#[derive(Clone, Debug)]
pub struct UrlInfo {
    pub url: String,
    pub is_github: bool,
    pub is_local: bool,
    pub bridge: BridgeState,
}

/// Ein Magazin-Platz.
#[derive(Clone, Debug)]
pub struct ClipItem {
    pub id: Uuid,
    pub payload: Payload,
    pub captured_at: DateTime<Local>,
    pub fingerprint: u64,
    pub url: Option<UrlInfo>,
    /// Base64-Data-URL der Miniaturansicht (nur Bilder, einmalig berechnet).
    pub thumb: Option<String>,
    /// Gepinnte Elemente werden vom Auto-Rauswurf und von Alt+V übersprungen.
    pub pinned: bool,
}

impl ClipItem {
    pub fn new(payload: Payload) -> Self {
        let fingerprint = fingerprint_payload(&payload);
        let url = match &payload {
            Payload::Text(t) => crate::bridge::extract_url(t).map(|u| UrlInfo {
                url: u.url,
                is_github: u.is_github,
                is_local: u.is_local,
                bridge: BridgeState::Pending,
            }),
            _ => None,
        };
        Self {
            id: Uuid::new_v4(),
            payload,
            captured_at: Local::now(),
            fingerprint,
            url,
            thumb: None,
            pinned: false,
        }
    }

    /// Erzeugt die Miniaturansicht für Bild-Elemente (einmalig, beim Erfassen).
    pub fn build_thumb(&mut self) {
        if let Payload::Image { png, .. } = &self.payload {
            use clipboard_rs::common::{RustImage, RustImageData};
            self.thumb = RustImageData::from_bytes(png)
                .ok()
                .and_then(|img| img.thumbnail(280, 280).ok())
                .and_then(|t| t.to_png().ok())
                .map(|b| {
                    format!(
                        "data:image/png;base64,{}",
                        base64::engine::general_purpose::STANDARD.encode(b.get_bytes())
                    )
                });
        }
    }
}

/// Kompakte, serialisierbare Sicht eines Elements für das Frontend.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemLite {
    pub id: String,
    /// "text" | "image" | "files"
    pub kind: String,
    /// 1 = Spitze des Magazins (nächster Alt+V-Paste), aufsteigend = älter.
    pub position: u32,
    /// Textvorschau bzw. erster Dateiname.
    pub preview: String,
    /// Zusatzzeile: Zeichenzahl, Bildmaße, Dateianzahl …
    pub meta: String,
    pub captured_at: String,
    pub is_url: bool,
    /// None | "pending" | "sent" | "endpoint-offline" | "disabled"
    pub bridge: Option<BridgeState>,
    pub is_github: bool,
    /// Data-URL der Miniaturansicht (Bilder).
    pub thumb: Option<String>,
    /// Vollständiger Text für die Suche; die Darstellung wird per CSS gekürzt.
    pub text_preview: Option<String>,
    /// Gepinnt: bleibt beim Auto-Rauswurf und wird von Alt+V übersprungen.
    pub pinned: bool,
}

// ---------------------------------------------------------------------------
// Persistenz (Magazin überlebt Neustarts)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct PersistedItem {
    id: String,
    kind: String,
    text: Option<String>,
    png_b64: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    files: Option<Vec<String>>,
    captured_at_ms: i64,
    pinned: bool,
    #[serde(default)]
    bridge: Option<BridgeState>,
}

fn persist_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_data_dir()
        .ok()
        .map(|d| d.join("magazine.json"))
}

/// Schreibt den Magazin-Stand weg (bei jeder Änderung).
fn persist(app: &AppHandle, magazine: &VecDeque<ClipItem>) {
    let Some(path) = persist_path(app) else {
        return;
    };
    let items: Vec<PersistedItem> = magazine
        .iter()
        .map(|i| {
            let (kind, text, png_b64, width, height, files) = match &i.payload {
                Payload::Text(t) => ("text", Some(t.clone()), None, None, None, None),
                Payload::Image { png, width, height } => (
                    "image",
                    None,
                    Some(base64::engine::general_purpose::STANDARD.encode(png)),
                    Some(*width),
                    Some(*height),
                    None,
                ),
                Payload::Files(f) => ("files", None, None, None, None, Some(f.clone())),
            };
            PersistedItem {
                id: i.id.to_string(),
                kind: kind.into(),
                text,
                png_b64,
                width,
                height,
                files,
                captured_at_ms: i.captured_at.timestamp_millis(),
                pinned: i.pinned,
                bridge: i.url.as_ref().map(|u| u.bridge),
            }
        })
        .collect();

    let json = match serde_json::to_string(&items) {
        Ok(j) => j,
        Err(e) => {
            crate::logger::log_fmt(
                app,
                format!("Persistenz: Serialisierung fehlgeschlagen ({e})"),
            );
            return;
        }
    };
    if let Err(e) = crate::storage::write_atomic(&path, json.as_bytes()) {
        crate::logger::log_fmt(app, format!("Persistenz: Schreiben fehlgeschlagen ({e})"));
    }
}

/// Lädt den Magazin-Stand beim Start zurück.
fn restore(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use chrono::TimeZone as _;
    let Some(path) = persist_path(app) else {
        return Err("Kein Datenverzeichnis".into());
    };
    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    let items: Vec<PersistedItem> = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            crate::logger::log_fmt(
                app,
                format!("Persistenz: Datei unlesbar ({e}) — Start gestoppt, Datei bleibt erhalten"),
            );
            return Err(e.into());
        }
    };

    let state = app.state::<AppState>();
    let mut mag = state.magazine.lock().unwrap();
    let mut restored = 0usize;
    let mut dropped_images = 0usize;
    for p in items {
        let payload = match (p.kind.as_str(), p.text, p.png_b64, p.files) {
            ("text", Some(t), _, _) => Some(Payload::Text(t)),
            ("image", _, Some(b64), _) => {
                match base64::engine::general_purpose::STANDARD.decode(b64) {
                    Ok(png) => Some(Payload::Image {
                        png,
                        width: p.width.unwrap_or(0),
                        height: p.height.unwrap_or(0),
                    }),
                    Err(_) => None,
                }
            }
            ("image", _, None, _) => {
                dropped_images += 1;
                None
            }
            ("files", _, _, Some(f)) if !f.is_empty() => Some(Payload::Files(f)),
            _ => None,
        };
        let Some(payload) = payload else { continue };
        let captured_at = chrono::Local
            .timestamp_millis_opt(p.captured_at_ms)
            .single()
            .unwrap_or_else(Local::now);
        let mut item = ClipItem::new(payload);
        item.id = p.id.parse().unwrap_or_else(|_| Uuid::new_v4());
        item.captured_at = captured_at;
        item.pinned = p.pinned;
        // Zustellbestätigung nur übernehmen, wenn sie tatsächlich gespeichert wurde.
        if let Some(u) = item.url.as_mut() {
            u.bridge = match p.bridge {
                Some(BridgeState::Pending) | None => BridgeState::Unknown,
                Some(status) => status,
            };
        }
        item.build_thumb();
        mag.push_back(item);
        restored += 1;
    }
    drop(mag);
    if restored > 0 {
        crate::logger::log_fmt(
            app,
            format!("Persistenz: {restored} Elemente aus vorheriger Sitzung wiederhergestellt{dropped_images_pre}", 
                dropped_images_pre = if dropped_images > 0 { format!(" ({dropped_images} zu große Bilder ausgelassen)") } else { String::new() }),
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Anwendungszustand
// ---------------------------------------------------------------------------

pub struct AppState {
    pub magazine: Mutex<VecDeque<ClipItem>>,
    pub paste_lock: Mutex<()>,
    persist_lock: Mutex<()>,
    persist_tx: mpsc::SyncSender<()>,
    /// Fingerprint des letzten selbst geschriebenen Clipboard-Inhalts.
    pub self_write: AtomicU64,
    /// Verbraucht genau das nächste Clipboard-Ereignis, wenn es der eigenen
    /// Schreiboperation entspricht (verhindert Selbst-Reflektion beim Pasten).
    pub ignore_next: AtomicBool,
}

impl AppState {
    fn new(persist_tx: mpsc::SyncSender<()>) -> Self {
        Self {
            magazine: Mutex::new(VecDeque::new()),
            paste_lock: Mutex::new(()),
            persist_lock: Mutex::new(()),
            persist_tx,
            self_write: AtomicU64::new(0),
            ignore_next: AtomicBool::new(false),
        }
    }
}

pub fn init(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::sync_channel(1);
    app.manage(AppState::new(tx));
    restore(&app)?;
    std::thread::Builder::new()
        .name("cnp-persist".into())
        .spawn(move || {
            while rx.recv().is_ok() {
                persist_now(&app);
            }
        })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Magazin-Operationen
// ---------------------------------------------------------------------------

/// Fingerprint über Art + Inhalt — identifiziert Clipboard-Inhalte zuverlässig.
pub fn fingerprint_payload(payload: &Payload) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    match payload {
        Payload::Text(t) => {
            "text".hash(&mut h);
            t.hash(&mut h);
        }
        Payload::Image { png, .. } => {
            "image".hash(&mut h);
            png.hash(&mut h);
        }
        Payload::Files(f) => {
            "files".hash(&mut h);
            f.hash(&mut h);
        }
    }
    h.finish()
}

/// Nimmt ein neu erfasstes Element ins Magazin auf.
/// Eigene Schreiboperationen (Reverse-Paste) und identische Folge-Kopien werden verworfen.
pub fn ingest(app: &AppHandle, mut item: ClipItem) {
    let state = app.state::<AppState>();
    let fp = item.fingerprint;

    // Selbst-Reflektion: das Ereignis stammt von unserem eigenen Paste-Schreiben.
    if state.ignore_next.swap(false, Ordering::SeqCst)
        && state.self_write.load(Ordering::SeqCst) == fp
    {
        return;
    }
    item.build_thumb();

    let capacity = crate::config::get(app).capacity.max(1);
    {
        let mut mag = state.magazine.lock().unwrap();
        if is_duplicate(&mag, fp) {
            return;
        }
        mag.push_back(item.clone());
        trim_capacity(&mut mag, capacity);
    }

    let kind = match &item.payload {
        Payload::Text(t) => format!("Text ({} Zeichen)", t.chars().count()),
        Payload::Image { width, height, .. } => format!("Bild ({}x{})", width, height),
        Payload::Files(f) => format!("Dateien ({})", f.len()),
    };
    crate::logger::log_fmt(
        app,
        format!(
            "Erfasst: {} — Magazin belegt {}/{}",
            kind,
            {
                let mag = state.magazine.lock().unwrap();
                mag.len()
            },
            capacity
        ),
    );

    emit_update(app);

    // Smart-Bridge: URL erkannt → asynchronen Webhook anstoßen (blockiert nichts).
    if let Some(url_info) = item.url.as_ref() {
        crate::bridge::dispatch_async(
            app.clone(),
            item.id,
            url_info.url.clone(),
            url_info.is_github,
            url_info.is_local,
        );
    }
}

/// Wählt das jüngste kompatible ungepinnte Element, ohne es zu entfernen.
pub fn select_for_paste(mag: &VecDeque<ClipItem>, text_only: bool) -> Option<ClipItem> {
    mag.iter()
        .rev()
        .find(|i| !i.pinned && (!text_only || matches!(i.payload, Payload::Text(_))))
        .cloned()
}

fn is_duplicate(mag: &VecDeque<ClipItem>, fingerprint: u64) -> bool {
    mag.back()
        .is_some_and(|i| !i.pinned && i.fingerprint == fingerprint)
}

/// Gemeinsamer Schutz für Aufnahme und Kapazitätsänderung.
pub fn trim_capacity(mag: &mut VecDeque<ClipItem>, capacity: usize) {
    while mag.len() > capacity {
        let Some(pos) = mag.iter().position(|i| !i.pinned) else {
            break;
        };
        mag.remove(pos);
    }
}

/// Entfernt ein Element per ID und liefert es zurück.
pub fn remove_by_id(app: &AppHandle, id: &str) -> Option<ClipItem> {
    let state = app.state::<AppState>();
    let uuid = Uuid::parse_str(id).ok()?;
    let removed = {
        let mut mag = state.magazine.lock().unwrap();
        mag.iter()
            .position(|i| i.id == uuid)
            .and_then(|pos| mag.remove(pos))
    };
    if removed.is_some() {
        emit_update(app);
    }
    removed
}

/// Leert das Magazin.
pub fn clear(app: &AppHandle) {
    let state = app.state::<AppState>();
    state.magazine.lock().unwrap().clear();
    crate::logger::log(app, "Magazin geleert");
    emit_update(app);
}

/// Pin umschalten: gepinnte Elemente bleiben beim Auto-Rauswurf und werden
/// von Alt+V übersprungen (nur gezielt per Klick/Enter einfügbar).
pub fn toggle_pin(app: &AppHandle, id: &str) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let uuid = Uuid::parse_str(id).map_err(|_| "Ungültige ID.".to_string())?;
    let pinned = {
        let mut mag = state.magazine.lock().unwrap();
        mag.iter_mut()
            .find(|i| i.id == uuid)
            .map(|i| {
                i.pinned = !i.pinned;
                i.pinned
            })
            .ok_or_else(|| "Element nicht gefunden.".to_string())?
    };
    crate::logger::log_fmt(
        app,
        format!(
            "Pin {} ({})",
            if pinned { "gesetzt" } else { "entfernt" },
            id
        ),
    );
    emit_update(app);
    Ok(pinned)
}

/// Aktualisiert den Bridge-Status eines Elements (aus dem Webhook-Thread).
pub fn set_bridge_state(app: &AppHandle, id: Uuid, bridge: BridgeState) {
    let state = app.state::<AppState>();
    {
        let mut mag = state.magazine.lock().unwrap();
        if let Some(item) = mag.iter_mut().find(|i| i.id == id) {
            if let Some(u) = item.url.as_mut() {
                u.bridge = bridge;
            }
        }
    }
    emit_update(app);
}

/// Vermerkt eine selbst geschriebene Clipboard-Operation, damit der Watcher
/// genau dieses eine Ereignis verwirft.
pub fn announce_self_write(app: &AppHandle, fp: u64) {
    let state = app.state::<AppState>();
    state.self_write.store(fp, Ordering::SeqCst);
    state.ignore_next.store(true, Ordering::SeqCst);
}

/// Sendet den kompletten Magazin-Zustand an das Frontend, aktualisiert den
/// Tray-Tooltip und persistiert den Stand.
pub fn emit_update(app: &AppHandle) {
    let lites = snapshot(app);

    // Tray-Tooltip: Elementzahl + Vorschau der Paste-Spitze.
    if let Some(tray) = app.tray_by_id(crate::tray::TRAY_ID) {
        let tip = match lites.iter().find(|i| !i.pinned) {
            Some(top) => format!(
                "copyNpaste — {} Elemente · Als nächstes einfügen: {}",
                lites.len(),
                truncate(&top.preview, 40)
            ),
            None if !lites.is_empty() => {
                format!("copyNpaste — {} Elemente (alle gepinnt)", lites.len())
            }
            None => "copyNpaste — Magazin leer".to_string(),
        };
        let _ = tray.set_tooltip(Some(tip));
    }

    if let Err(e) = app.emit(EVENT_CHANGED, lites) {
        crate::logger::log_fmt(app, format!("Fehler beim Event-Versand: {e}"));
    }

    let _ = app.state::<AppState>().persist_tx.try_send(());
}

/// Der Worker und das Beenden speichern stets den neuesten Stand unter derselben Sperre.
pub fn persist_now(app: &AppHandle) {
    let state = app.state::<AppState>();
    let _write = state.persist_lock.lock().unwrap();
    let mag = state.magazine.lock().unwrap().clone();
    persist(app, &mag);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}…")
    }
}

/// Erzeugt die Frontend-Sicht (neueste zuerst, Position 1 = Paste-Spitze).
pub fn snapshot(app: &AppHandle) -> Vec<ItemLite> {
    let state = app.state::<AppState>();
    let mag = state.magazine.lock().unwrap();
    mag.iter()
        .rev()
        .enumerate()
        .map(|(idx, item)| {
            let position = (idx + 1) as u32;
            let (preview, meta, text_preview) = match &item.payload {
                Payload::Text(t) => {
                    let first_line = t
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(80)
                        .collect::<String>();
                    let n = t.chars().count();
                    (first_line, format!("{n} Zeichen"), Some(t.clone()))
                }
                Payload::Image { width, height, png } => (
                    format!("{width}×{height}"),
                    format!("{width}×{height} · {}", kib(png.len())),
                    None,
                ),
                Payload::Files(f) => {
                    let base = |p: &String| p.rsplit(['\\', '/']).next().unwrap_or(p).to_string();
                    let name = f.first().map(&base).unwrap_or_default();
                    let joined = f.iter().map(&base).collect::<Vec<_>>().join("\n");
                    (name, format!("{} Datei(en)", f.len()), Some(joined))
                }
            };
            ItemLite {
                id: item.id.to_string(),
                kind: match item.payload {
                    Payload::Text(_) => "text",
                    Payload::Image { .. } => "image",
                    Payload::Files(_) => "files",
                }
                .to_string(),
                position,
                preview,
                meta,
                captured_at: item.captured_at.format("%d.%m. %H:%M:%S").to_string(),
                is_url: item.url.is_some(),
                bridge: item.url.as_ref().map(|u| u.bridge),
                is_github: item.url.as_ref().map(|u| u.is_github).unwrap_or(false),
                thumb: item.thumb.clone(),
                text_preview,
                pinned: item.pinned,
            }
        })
        .collect()
}

fn kib(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    }
}

// ---------------------------------------------------------------------------
// Unit-Tests: Magazin-Logik und Regex (Belegpflicht aus AGENTS.md)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprints_unterscheiden_arten() {
        let a = fingerprint_payload(&Payload::Text("hallo".into()));
        let b = fingerprint_payload(&Payload::Text("hallo".into()));
        let c = fingerprint_payload(&Payload::Text("welt".into()));
        let d = fingerprint_payload(&Payload::Files(vec!["C:/x.txt".into()]));
        assert_eq!(a, b, "identischer Inhalt → gleicher Fingerprint");
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn vecdeque_verhaelt_sicht_wie_magazin() {
        // Simulation der Kern-Stack-Logik ohne AppHandle:
        // 3x kopieren → 3x Reverse-Paste liefert umgekehrt und leert das Magazin.
        let mut mag: VecDeque<&str> = VecDeque::new();
        for x in ["eins", "zwei", "drei"] {
            mag.push_back(x);
            while mag.len() > 10 {
                mag.pop_front();
            }
        }
        assert_eq!(mag.pop_back(), Some("drei"));
        assert_eq!(mag.pop_back(), Some("zwei"));
        assert_eq!(mag.pop_back(), Some("eins"));
        assert_eq!(mag.pop_back(), None);
    }

    #[test]
    fn kapazitaet_verwirft_aeltestes() {
        let mut mag: VecDeque<u32> = VecDeque::new();
        for i in 0..12 {
            mag.push_back(i);
            while mag.len() > 10 {
                mag.pop_front();
            }
        }
        assert_eq!(mag.len(), 10);
        assert_eq!(*mag.front().unwrap(), 2, "die zwei ältesten sind gefallen");
        assert_eq!(*mag.back().unwrap(), 11);
    }

    #[test]
    fn echte_pins_bleiben_beim_verkleinern() {
        let mut mag = VecDeque::new();
        for n in 0..25 {
            let mut item = ClipItem::new(Payload::Text(n.to_string()));
            item.pinned = n == 0;
            mag.push_back(item);
        }
        trim_capacity(&mut mag, 10);
        assert_eq!(mag.len(), 10);
        assert!(mag.front().unwrap().pinned);
        assert!(!select_for_paste(&mag, false).unwrap().pinned);
    }

    #[test]
    fn verbrauchter_oder_gepinnter_inhalt_kann_erneut_erfasst_werden() {
        let item = ClipItem::new(Payload::Text("same".into()));
        let mut mag = VecDeque::from([item.clone()]);
        assert!(is_duplicate(&mag, item.fingerprint));
        mag.pop_back();
        assert!(!is_duplicate(&mag, item.fingerprint));
        let mut pinned = item.clone();
        pinned.pinned = true;
        mag.push_back(pinned);
        assert!(!is_duplicate(&mag, item.fingerprint));
    }

    #[test]
    fn textziel_ueberspringt_bilder_ohne_entnahme() {
        let mut mag = VecDeque::from([ClipItem::new(Payload::Text("  code\n".into()))]);
        mag.push_back(ClipItem::new(Payload::Files(vec!["file.txt".into()])));
        assert!(matches!(
            select_for_paste(&mag, true).unwrap().payload,
            Payload::Text(_)
        ));
        assert!(matches!(
            select_for_paste(&mag, false).unwrap().payload,
            Payload::Files(_)
        ));
        assert_eq!(mag.len(), 2);
    }

    #[test]
    fn persistierung_roundtrip_ohne_bilder() {
        // Die Persistenzstruktur muss verlustfrei serialisierbar sein.
        let items = vec![PersistedItem {
            id: "0b6f6c79-7c9d-4a54-a111-0b25e26a5b8f".into(),
            kind: "text".into(),
            text: Some("https://github.com/example/repo".into()),
            png_b64: None,
            width: None,
            height: None,
            files: None,
            captured_at_ms: 1_799_000_000_000,
            pinned: true,
            bridge: Some(BridgeState::EndpointOffline),
        }];
        let json = serde_json::to_string(&items).expect("serialisierbar");
        let back: Vec<PersistedItem> = serde_json::from_str(&json).expect("deserialisierbar");
        assert_eq!(back.len(), 1);
        assert_eq!(
            back[0].text.as_deref(),
            Some("https://github.com/example/repo")
        );
        assert!(back[0].pinned);
        assert_eq!(back[0].bridge, Some(BridgeState::EndpointOffline));
    }
}
