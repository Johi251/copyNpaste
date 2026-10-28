//! Die Localhost-Smart-Bridge: erkennt URLs im Clipboard-Strom und meldet sie
//! asynchron per Webhook an das "Today's Hunt"-Modul des Bookmarks Dynamic
//! Managers. Der Versand blockiert den Clipboard-Ablauf zu 0 % — er läuft in
//! einem eigenen Thread mit hartem Timeout.

use std::sync::OnceLock;
use std::time::Duration;

use regex::Regex;
use serde_json::json;
use tauri::AppHandle;
use uuid::Uuid;

use crate::state::{self, BridgeState};

/// Standard-Endpunkt des Bookmarks Dynamic Managers (Today's Hunt).
pub const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:9911/api/todays-hunt";

/// Harter Timeout für den Webhook — danach gilt der Endpunkt als offline.
const WEBHOOK_TIMEOUT: Duration = Duration::from_millis(2500);

/// Eine vom Regex-Filter erkannte URL.
pub struct UrlMatch {
    pub url: String,
    pub is_github: bool,
    /// localhost / 127.0.0.1 — echte URL, aber i. d. R. kein Lesezeichen-Kandidat.
    pub is_local: bool,
}

fn url_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Vollständige Übereinstimmung:Schema://Host[(:Port)][/Pfad][?Query][#Fragment]
        // Host: „localhost" oder eine Domain mit mindestens einem Punkt.
        Regex::new(r"^https?://(?P<host>localhost|127\.0\.0\.1|(?:[A-Za-z0-9](?:[A-Za-z0-9-]*[A-Za-z0-9])?\.)+[A-Za-z0-9](?:[A-Za-z0-9-]*[A-Za-z0-9])?)(?::[0-9]{1,5})?(?:/[^\s?#]*)?(?:\?[^\s#]*)?(?:#.*)?$")
            .expect("statisches URL-Regex ist gültig")
    })
}

/// Prüft, ob ein kopiertes Element *genau* eine echte URL ist
/// (kein Satz mit eingebettetem Link, keine Leerzeichen, max. 2048 Zeichen).
pub fn extract_url(text: &str) -> Option<UrlMatch> {
    let t = text.trim();
    if t.is_empty() || t.len() > 2048 {
        return None;
    }
    if t.chars().any(char::is_whitespace) {
        return None;
    }
    let re = url_regex();
    let caps = re.captures(t)?;
    let host = caps.name("host")?.as_str().to_ascii_lowercase();
    Some(UrlMatch {
        url: t.to_string(),
        is_github: host == "github.com" || host.ends_with(".github.com") || host == "github.io" || host.ends_with(".github.io"),
        is_local: host == "localhost" || host == "127.0.0.1",
    })
}

/// Stößt den Webhook-Versand an — kehrt sofort zurück (fire-and-forget).
pub fn dispatch_async(app: AppHandle, item_id: Uuid, url: String, is_github: bool, is_local: bool) {
    let cfg = crate::config::get(&app);
    if !cfg.bridge_enabled {
        state::set_bridge_state(&app, item_id, BridgeState::Disabled);
        return;
    }
    let endpoint = cfg.bridge_url.clone();
    std::thread::Builder::new()
        .name("cnp-bridge-webhook".into())
        .spawn(move || send_webhook(&app, item_id, &endpoint, &url, is_github, is_local))
        .ok();
}

/// Blockierender Versand (läuft im eigenen Thread): POST {url, source, …}.
fn send_webhook(app: &AppHandle, item_id: Uuid, endpoint: &str, url: &str, is_github: bool, is_local: bool) {
    let hint = if is_local {
        "local-address"
    } else if is_github {
        "software-repository"
    } else {
        "general-url"
    };
    let body = json!({
        "url": url,
        "source": "copyNpaste",
        "capturedAt": chrono::Local::now().to_rfc3339(),
        "isGithub": is_github,
        "hint": hint,
    });

    crate::logger::log_fmt(app, format!("Bridge: POST {endpoint} <- {url}"));

    let result = ureq::post(endpoint)
        .timeout(WEBHOOK_TIMEOUT)
        .send_json(body);

    match result {
        Ok(resp) => {
            let status = resp.status();
            if (200..300).contains(&status) {
                crate::logger::log_fmt(app, format!("Bridge: zugestellt (HTTP {status}) — {url}"));
                state::set_bridge_state(app, item_id, BridgeState::Sent);
            } else {
                crate::logger::log_fmt(app, format!("Bridge: Endpunkt antwortete HTTP {status} — {url}"));
                state::set_bridge_state(app, item_id, BridgeState::EndpointOffline);
            }
        }
        Err(e) => {
            // Erwartbar, solange der Bookmarks Manager nicht läuft — kein Fehler im Clipboard-Fluss.
            crate::logger::log_fmt(app, format!("Bridge: Endpunkt nicht erreichbar ({e}) — {url}"));
            state::set_bridge_state(app, item_id, BridgeState::EndpointOffline);
        }
    }
}

/// Sondierender Test aus dem Einstellungen-Dialog (synchron, mit eigenem Timeout).
pub fn probe(app: &AppHandle) -> Result<String, String> {
    let cfg = crate::config::get(app);
    let endpoint = cfg.bridge_url.clone();
    let body = json!({
        "probe": true,
        "source": "copyNpaste",
        "sentAt": chrono::Local::now().to_rfc3339(),
    });
    match ureq::post(&endpoint).timeout(Duration::from_millis(2500)).send_json(body) {
        Ok(resp) => Ok(format!("Endpunkt erreichbar — HTTP {}", resp.status())),
        Err(e) => Err(format!("Endpunkt nicht erreichbar: {e}")),
    }
}

// ---------------------------------------------------------------------------
// Unit-Tests: URL-Regex (Belegpflicht aus AGENTS.md)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echte_urls_werden_erkannt() {
        for ok in [
            "https://github.com",
            "https://github.com/tauri-apps/tauri",
            "http://localhost:8080/dashboard",
            "http://127.0.0.1:3000/ui",
            "https://de.wikipedia.org/wiki/Zwischenablage",
            "https://example.com/path?q=1#top",
            "https://sub.domain.example.co.uk/a/b/c",
        ] {
            assert!(extract_url(ok).is_some(), "sollte URL sein: {ok}");
        }
    }

    #[test]
    fn lokale_adressen_werden_markiert() {
        let m = extract_url("http://localhost:8080/dashboard").unwrap();
        assert!(m.is_local && !m.is_github);
        let m = extract_url("https://example.com").unwrap();
        assert!(!m.is_local);
    }

    #[test]
    fn github_wird_als_solches_markiert() {
        assert!(extract_url("https://github.com/Johi251/aKlik-Deployer").unwrap().is_github);
        assert!(!extract_url("https://webflow.com").unwrap().is_github);
    }

    #[test]
    fn keine_urls_werden_verworfen() {
        for bad in [
            "Hallo Welt",
            "Schau mal auf github.com vorbei",   // kein Schema, Leerzeichen
            "https://github.com und noch Text",  // Leerzeichen innerhalb
            "ftp://files.example.com",           // falsches Schema
            "https://intranet",                  // Ein-Wort-Host ohne Punkt und ohne localhost
            "",                                  // leer
        ] {
            assert!(extract_url(bad).is_none(), "sollte KEINE URL sein: {bad}");
        }
    }

    #[test]
    fn zu_lange_eingaben_werden_verworfen() {
        let long = format!("https://example.com/{}", "a".repeat(3000));
        assert!(extract_url(&long).is_none());
    }
}
