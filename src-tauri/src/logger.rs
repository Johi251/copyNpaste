//! Minimalistische Datei-Protokollierung für Nachweise (Belegpflicht).
//! Schreibt zeilenweise nach `<AppData>/logs/copyNpaste.log`.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager};

static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

pub fn init(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let dir: PathBuf = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("logs");
    fs::create_dir_all(&dir)?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("copynpaste.log"))?;
    *LOG_FILE.lock().unwrap() = Some(file);
    Ok(())
}

/// Eine Zeile mit Zeitstempel protokollieren.
pub fn log(app: &AppHandle, msg: &str) {
    log_static(msg);
    let _ = app; // AppHandle wird für zukünftige Erweiterungen mitgeführt
}

/// Wie `log`, ohne AppHandle (aus Threads ohne Zugriff).
pub fn log_static(msg: &str) {
    let line = format!("[{}] {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), msg);
    #[cfg(debug_assertions)]
    eprint!("{line}");
    if let Some(f) = LOG_FILE.lock().unwrap().as_mut() {
        let _ = f.write_all(line.as_bytes());
    }
}

/// `log` mit format!-Komfort.
pub fn log_fmt(app: &AppHandle, msg: String) {
    log(app, &msg);
}
