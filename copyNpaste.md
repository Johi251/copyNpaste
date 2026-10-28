# SYSTEMAUFTRAG: GREENFIELD-ENTWICKLUNG v1.0.0 — copyNpaste (CLIPBOARD-MAGAZIN)

Du agierst im autonomen GOAL-Modus ohne Rückfragen. Dieser Workspace ist aktuell leer. Deine Aufgabe ist es, die application "copyNpaste" von Grund auf als plattformübergreifende Tauri 2 Desktop-App (Windows EXE, Mac App, Linux deb) neu zu erschaffen. Du nutzt als fundamentale Wissensbasis die Datei: `C:\Users\johan\MASTER_SETUP\00 PROJEKTE\PROJECT_MINDSHARE.md`.

## SCHRITT 0: SYSTEM-DIAGNOSE & ABHÄNGIGKEITEN (CRITICAL FOR NEW SETUP)
1. Führe im Terminal eine autonome Prüfung durch, ob alle notwendigen Compiler und Runtimes vorhanden sind (Node.js/npm, Rust/Cargo, Windows C++ Build Tools).
2. Falls essenzielle CLI-Tools oder Bibliotheken für das plattformübergreifende Clipboard-Hooking fehlen, installiere sie selbstständig und lautlos über das Terminal, bevor du mit dem Coden beginnst.

## SCHRITT 1: ARCHITEKTUR & BRANDING (ZARION-INSPIRED)
1. Setze das UI als absolut edles, minimalistisches SaaS-Dashboard um (Stilvorbild: Linear, Raycast). 
2. Nutze einen tiefen Dark-Mode, ultradünne Rahmen (`border-neutral-800`) und dezente Unschärfe-Effekte (`backdrop-blur`).
3. **Akzentfarbe:** Wähle ein edles, hochmodernes Türkis-Jadegrün (`#0D9488`). Erstelle ein passendes App-Logo in dieser Farbe.

## SCHRITT 2: DIE CLIPBOARD-MAGAZIN-LOGIK (CORE FUNCTION)
1. Implementiere im Rust-Backend von Tauri ein globales Abfangen von Kopier-Befehlen (`Strg+C` / `Cmd+C`), das im Hintergrund läuft.
2. **Das Magazin-Prinzip (Stack):** Wenn ich mehrfach hintereinander kopiere (Text, Text, Bild), speichere alle Elemente nacheinander in einem Magazin (Kapazität: mindestens 10+ Plätze).
3. **Massen-Einfügen (Reverse Paste via Alt+V):** Wenn ich `Alt+V` drücke, füge das *letzte* kopierte Element ein und lösche es sofort aus dem Magazin. Das vorletzte Element rückt in die Paste-Bereitschaft nach.
4. **Zwei-Spalten-UI:** Gestalte das Clipboard-Fenster zweigeteilt: Linke Spalte für scrollbare Texte/Codes, rechte Spalte für scrollbare Bilder/Dateien. Beim Pasten muss intelligent erkannt werden, ob das Zielfeld Text oder Bilder akzeptiert.

## SCHRITT 3: DIE LOCALHOST-SMART-BRIDGE (aKlik-ANKÜPFUNG)
1. Integriere einen Regex-Filter für den Clipboard-Stream. Sobald ein kopiertes Element eine echte URL (insb. GitHub/Software) ist, schicke diese vollautomatisch als asynchronen Background-Webhook an den Bookmarks-Manager (Localhost-Schnittstelle vorberiten).
2. Sorge dafür, dass dieser Hintergrund-Prozess den normalen Clipboard-Ablauf zu 0% blockiert oder verzögert. Und zeichne am Anfang des Links für 0.2 bis 0.3 Sekunden eine extrem schnelle, Lottie-inspirierte Haken-Animation (✓) direkt unter dem Link-Anfang als visuelles Feedback.

## SCHRITT 4: COMPILATION & README
1. Initialisiere das Tauri 2 / Rust Projekt und erstelle die grundlegende Dateistruktur.
2. Schreibe die Build-Pipelines, um native Produktions-Builds (Windows .exe, Mac .app) zu kompilieren.
3. Generiere eine detaillierte `README.md` zur Installation und Validierung der Tastatur-Shortcuts.

STARTE JETZT VOLLSTÄNDIG AUTONOM MIT SCHRITT 0.
