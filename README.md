# copyNpaste — Das Clipboard-Magazin

> **Globales Betriebssystem-Zwischenablagen-Magazin mit Reverse-Paste.**
> Tauri 2 Desktop-App (Windows · macOS · Linux) von johan — Teil des aKlik-Ökosystems
> (App C laut `C:\Users\johan\MASTER_SETUP\00 PROJEKTE\PROJECT_MINDSHARE.md`).

![Akzentfarbe](https://img.shields.io/badge/Akzent-T%C3%BCrkis--Jade_%230D9488-0D9488) ![Plattform](https://img.shields.io/badge/Plattform-Windows_%C2%B7_macOS_%C2%B7_Linux-neutral) ![Stack](https://img.shields.io/badge/Stack-Tauri_2_%C2%B7_Rust_%C2%B7_TypeScript_%C2%B7_Tailwind-0D9488)

---

## Was copyNpaste tut

copyNpaste läuft als **schlanker Hintergrund-Dienst im System-Tray** und verwandelt die
Zwischenablage in ein **Magazin** (Stack):

1. **Kopieren ohne Limit** — jede Kopieraktion (`Strg+C` / `Cmd+C`, Rechtsklick → Kopieren,
   Drag&Drop-Kopien) wird im Hintergrund erfasst. Text, Bilder und Dateien wandern
   nacheinander ins Magazin (Standard: **25 Plätze**, konfigurierbar 10–100).
2. **Reverse-Paste mit `Alt+V`** — fügt das **zuletzt kopierte** Element in das fokussierte
   Zielfeld ein **und entfernt es sofort aus dem Magazin**. Das vorletzte Element rückt in
   die Paste-Bereitschaft. Mehrfach `Alt+V` = rückwärts durch den Stapel pumpen.
3. **Zwei-Spalten-Magazin** (`Alt+Shift+V` oder Tray-Klick) — links scrollbare Texte/Code,
   rechts scrollbare Bilder/Dateien. Klick oder `Enter` auf eine Karte fügt genau dieses
   Element ein; `Entf` wirft es weg.
4. **Localhost-Smart-Bridge** — kopierte echte URLs (insbesondere GitHub/Software-Links)
   werden per Regex erkannt und **vollautomatisch und asynchron** als Webhook an das
   „Today's Hunt"-Modul des *Bookmarks Dynamic Managers* geschickt. Der Clipboard-Fluss wird
   dabei zu **0 %** blockiert. Unter dem Link-Anfang erscheint für ~250 ms eine
   Lottie-inspirierte **Haken-Animation (✓)** als visuelles Erfolgs-Feedback — ohne Pop-up.

## Shortcuts

| Aktion | Standard | freely konfigurierbar |
|---|---|---|
| Reverse-Paste (Spitze einfügen + entfernen) | `Alt+V` | ✔ Einstellungen |
| Magazin-Fenster zeigen/verstecken | `Alt+Shift+V` | ✔ Einstellungen |
| Karte einfügen | Klick / `Enter` (Fokus auf Karte) | — |
| Karte entfernen | `Entf` (Fokus auf Karte) | — |
| Fenster schließen (läuft im Tray weiter) | `Esc` / `X` | — |
| Magazin leeren | Button „Leeren" | — |

> Shortcuts werden **global** registriert — sie funktionieren in jeder Anwendung.

## Installation

### Voraussetzungen (Entwicklung)

- **Node.js ≥ 20** + npm — <https://nodejs.org>
- **Rust ≥ 1.90** (rustup) — <https://rustup.rs>
- **Windows:** „Desktop development with C++" (VS 2022 Build Tools) + WebView2 Runtime (Win 11 vorinstalliert)
- **macOS:** Xcode Command Line Tools (`xcode-select --install`)
- **Linux:** `libxdo-dev` (enigo), X11/XWayland (Clipboard-Watching unter Wayland ist eingeschränkt), `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev` (Tauri)

```bash
npm install          # Abhängigkeiten
npm run tauri dev    # Entwicklung mit Hot-Reload
```

### Produktions-Build

```bash
npm run tauri build          # Windows: NSIS-Installer + eigenständige .exe
npm run tauri build -- --no-bundle   # nur die rohe .exe (schneller)
```

Ergebnis unter `src-tauri\target\release\`:

| Plattform | Artefakt |
|---|---|
| Windows | `copyNpaste.exe` + `bundle\nsis\copyNpaste_1.0.0_x64-setup.exe` |
| macOS | `copyNpaste.app` (+ `.dmg` via `--bundles dmg`) |
| Linux | `bundle\deb\copyNpaste_1.0.0_amd64.deb` |

Bequeme Wrapper: `build-scripts\build-windows.cmd`, `build-macos.sh`, `build-linux.sh`.
Für alle drei Plattformen automatisch: `.github/workflows/build.yml` (erzeugt Build-Artefakte — **ohne** Veröffentlichung).

### Benutzung

1. copyNpaste starten — es erscheint **als Icon im System-Tray** (Fenster bleibt zu).
2. Irgendwo `Strg+C` ausführen — Elemente stapeln sich im Magazin.
3. `Alt+V` im Ziel-Feld drücken — Spitze wird eingefügt und entfernt. Wiederholen = rückwärts pumpen.
4. `Alt+Shift+V` oder Tray-Linksklick — Magazin-Fenster öffnen.

**macOS-Hinweis:** Für den simulierten `Cmd+V` muss der App einmalig die Berechtigung
*Systemeinstellungen → Datenschutz & Sicherheit → Bedrohungshilfe* (Accessibility) erteilt werden.

**Linux-Hinweis:** Unter reinem Wayland funktionieren globale Clipboard-Watcher und
Tastensimulation nur eingeschränkt — X11/XWayland-Sitzung empfohlen.

## Validierung der Tastatur-Shortcuts (Checkliste)

| # | Test | Erwartung |
|---|---|---|
| 1 | App starten, `Strg+C` auf Text | Log `<AppData>\logs\copynpaste.log` zeigt `Erfasst: Text (…)` |
| 2 | `Alt+V` im Editor | Letzter Text wird eingefügt; Karte verschwindet; neue Spitze bekommt „Nächster Paste" |
| 3 | 3× `Strg+C` (Text, Text, Screenshot) | Linke Spalte: 2 Karten · rechte Spalte: 1 Karte |
| 4 | 3× `Alt+V` | Inhalte erscheinen in umgekehrter Reihenfolge (Screenshot, Text2, Text1) |
| 5 | `Strg+C` auf `https://github.com/…` | Karte mit URL + Haken ✓ „GitHub → Today's Hunt" (Bridge-Stub: `node scripts/test-webhook.mjs`) |
| 6 | Bridge aus, URL kopieren | Karte zeigt „Bridge offline" — Magazin unberührt, kein Verzögern |
| 7 | Dateien im Explorer `Strg+C`, `Alt+V` | Dateiliste wird als Kopie/Einfügen übertragen |
| 8 | `Alt+Shift+V` | Fenster zeigt/versteckt sich; Tray-Linksklick ebenfalls |
| 9 | X / `Esc` | Fenster zu, Prozess bleibt im Tray aktiv |
| 10 | Shortcut ändern (z. B. `ctrl+shift+v`) | Alt+V reagiert nicht mehr, neuer Shortcut pastet |

Nachweis-Log: `%APPDATA%\de.johan.copynpaste\logs\copyNpaste.log`
(config: `%APPDATA%\de.johan.copynpaste\config.json`).

## Die Smart-Bridge-Schnittstelle (Vertrag)

Kopiert ein Element eine **exakte URL** (Regex: `^https?://Host[ischer Teil]$`, keine
Leerzeichen, ≤ 2048 Zeichen), feuert copyNpaste **asynchron**:

```http
POST http://127.0.0.1:9911/api/todays-hunt
Content-Type: application/json

{
  "url": "https://github.com/tauri-apps/tauri",
  "source": "copyNpaste",
  "capturedAt": "2026-10-26T14:32:05.123+02:00",
  "isGithub": true,
  "hint": "software-repository"      // bzw. "general-url"
}
```

- Endpunkt + Ein/Aus sind **konfigurierbar** (Einstellungen → Localhost-Smart-Bridge; Button „Endpunkt testen").
- Timeout 2,5 s, fire-and-forget im eigenen Thread — der Bookmarks Manager kann fehlen,
  ohne den Clipboard-Fluss auch nur zu berühren (Status dann „Bridge offline" in der Karte).
- Test-Empfänger für Entwicklung: `node scripts/test-webhook.mjs` (schreibt `scripts/.webhook-received.jsonl`).

## Architektur

```
copyNpaste/
├─ index.html                  UI-Gerüst (Titelzeile, Toolbar, zwei Spalten)
├─ src/                        Frontend (Vanilla-TypeScript + Tailwind CSS 4)
│  ├─ main.ts                  Bootstrap, Events (magazine-changed), Aktionen
│  ├─ ui.ts                    Karten, Spalten, Haken-Animation (250 ms SVG-Strichzeichnung)
│  ├─ settings.ts              Einstellungen-Dialog inkl. Bridge-Test
│  └─ styles.css               Glas-Dark-Theme (#0b0c0e), backdrop-blur, Teal #0D9488
├─ src-tauri/src/              Rust-Backend
│  ├─ monitor.rs               OS-Clipboard-Watcher (clipboard-rs) als Dauergthread
│  ├─ state.rs                 Magazin (VecDeque), Fingerprints, Selbst-Reflektions-Schutz
│  ├─ paste.rs                 Reverse-Paste: Clipboard schreiben + Strg+V/Cmd+V (enigo)
│  ├─ bridge.rs                URL-Regex + asynchroner Webhook (ureq, 2,5-s-Timeout)
│  ├─ config.rs                config.json: Kapazität, Shortcuts, Bridge, Effekte
│  ├─ shortcuts.rs             globale Shortcuts (tauri-plugin-global-shortcut)
│  ├─ tray.rs                  Tray-Icon + Menü (Öffnen / Reverse Paste / Beenden)
│  ├─ logger.rs                Nachweis-Log <AppData>/logs/copynpaste.log
│  └─ commands.rs              IPC: magazine_state, paste_item, delete_item, …
├─ assets/logo.svg             Marken-Logo (Magazin-Stapel + Haken, #0D9488)
├─ scripts/                    gen-icon.mjs · test-webhook.mjs
└─ build-scripts/              Plattform-Wrapper
```

### Technische Entscheidungen (ehrlich dokumentiert)

- **„Globales Abfangen von Strg+C"** ist als **Clipboard-Change-Watcher** umgesetzt: Nicht die
  Taste wird abgefangen, sondern das *Ergebnis jeder Kopieraktion* wird beobachtet. Das ist
  zuverlässiger (erfasst auch Rechtsklick → Kopieren) und plattformübergreifend stabil.
- **„Intelligente Zielfeld-Erkennung":** Ein Hintergrund-Prozess darf den Inhalt fremder
  Fenster nicht inspizieren (OS-Sicherheitsmodell von Windows/macOS/Linux). copyNpaste löst
  das so: Jedes Element behält seinen **nativem Format** (Text → Text, Bild → Bitmap,
  Dateien → Dateiliste) und fügt exakt dieses ein; das Zielfeld akzeptiert oder ignoriert es
  wie bei einem manischen `Strg+V`. Über die Zwei-Spalten-Ansicht wählt man bewusst Text
  **oder** Bild/Datei — das ist die verlässlichste „Intelligenz" unter OS-Restriktionen.
- **Haken-Animation:** Das Zeichnen *in fremden Fenstern* unter dem kopierten Link ist einem
  Hintergrund-Dienst vom OS nicht gestattet. Die 250-ms-Strichzeichnung (✓) erscheint
  daher **in der Magazin-Karte direkt unter dem Link-Anfang** — exakt dem Sketch aus dem
  Projektauftrag folgend, ohne nerviges Pop-up.

## Verwandte Projekte

- **aKlik-Deployer** (App A) — Infrastruktur-/Deployment-Scanner · Akzent: Cobalt `#1E40AF`
- **Bookmarks Dynamic Manager** (App B) — KI-Wissensdatenbank für Lesezeichen · Akzent: Spicy Orange `#EA580C`
- **copyNpaste** (App C, dieses Repo) — Clipboard-Magazin · Akzent: Türkis-Jade `#0D9488`

Alle drei kommunizieren über Localhost-Webhooks; copyNpaste → Bookmarks (Today's Hunt) ist
die erste implementierte Brücke.

---

*Status: v1.0.0 · Build & Tests: siehe Abschnitt „Validierung" · Maintainer: johan*
