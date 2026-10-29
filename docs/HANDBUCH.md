---
lang: de
title: copyNpaste 1.2 – Handbuch und Workflows
subtitle: Kopieren, stapeln und rückwärts einfügen · Türkis-Jade · aKlik-Ökosystem
template: doc
theme: paper
---

## A Dein Clipboard wird zum Magazin

Kopiere mehrere Inhalte nacheinander. Die Desktop-App merkt sich Text, Code, Bilder und Dateipfade. Alt+V sendet das jüngste passende ungepinnte Element an dein Zielfenster. Erst nach erfolgreichem Clipboard-Schreiben und Tastensignal wird es entnommen.

```flow LR
Kopieren -> Magazin: Text, Bild oder Datei
Magazin -> Zielfeld: Alt+V
Zielfeld -> Nächster Eintrag: Entnahme nach Tastensignal
```

Die Zielanwendung entscheidet endgültig, ob sie ein Tastensignal und das angebotene Format annimmt. Kein Clipboard-Tool kann jede Anwendung zur Annahme zwingen. Bei einem erkannten Schreib-, Fokus- oder Tastaturfehler bleibt das Element im Magazin.

## B Installation und erster Start

| System | Datei und Ablauf |
|---|---|
| Windows x64 | NSIS-Setup starten oder die enthaltene copynpaste.exe verwenden. WebView2 wird benötigt. |
| Mac Intel | Intel-DMG öffnen und copyNpaste.app in Programme ziehen. |
| Mac Apple Silicon | Apple-Silicon-DMG verwenden; nicht mit der Intel-Ausgabe verwechseln. |
| Linux x64 | DEB mit dem Paketmanager installieren. X11/XWayland wird benötigt. |
| Chromium-Browser | Chromium-ZIP entpacken und als entpackte Erweiterung laden. |
| Firefox | Firefox-ZIP ist für temporäres Laden über about:debugging vorbereitet; dauerhafte Verteilung benötigt Mozilla-Signierung. |
| Web | Veröffentlichte GitHub-Pages-Seite öffnen. Im Gesamt-ZIP enthaltene Web-Dateien über einen lokalen HTTP-Server bereitstellen. |

Die Desktop-App startet unsichtbar im System-Tray. Ein Klick auf das Tray-Icon oder Alt+Shift+V öffnet das Magazin. Auf dem Mac benötigt die Tastensimulation die vom Betriebssystem verlangte Bedienungshilfen-Berechtigung. Die Builds werden ohne kostenpflichtige Codesignierung oder Apple-Notarisierung ausgeliefert; Installer-/Gatekeeper-Freigaben sind kein automatisierter Prüfnachweis.

## C Texte, Bilder und Dateien rückwärts abarbeiten

Beispiel: Kopiere zuerst eine Überschrift, danach einen Absatz und danach einen Screenshot. In einem Bild- oder Dokumenteditor liefert Alt+V zuerst den Screenshot, danach den Absatz und zuletzt die Überschrift.

```sequence num
Du -> Quelle: Überschrift kopieren
Du -> Quelle: Absatz kopieren
Du -> Quelle: Screenshot kopieren
Du -> Ziel: Alt+V – Screenshot
Du -> Ziel: Alt+V – Absatz
Du -> Ziel: Alt+V – Überschrift
```

Für ein erkanntes einfaches Textfeld wird das jüngste Textelement gewählt; das Bild bleibt erhalten. Windows nutzt UI-Automation-Metadaten, macOS erkennt einfache AX-Textfelder. In Rich-Text-Editoren und bei fehlenden Accessibility-Metadaten gilt die normale Stapelreihenfolge. Linux hat derzeit keine verlässliche AT-SPI-Zielfelderkennung.

Dateien bleiben Verweise auf die Originaldateien. Verschieben oder Löschen des Originals macht den Verweis unbrauchbar. Es wird keine zweite Dateikopie angelegt.

## D Suchen, Pins und Einstellungen

| Aktion | Bedienung |
|---|---|
| Magazin öffnen/schließen | Alt+Shift+V oder Tray-Klick |
| Nächsten Inhalt einfügen | Alt+V; erst nach Loslassen der Tasten |
| Bestimmten Inhalt einfügen | Karte anklicken oder Karte fokussieren und Enter |
| Quick-Paste | Im Suchfeld tippen, dann Enter |
| Anpinnen | Pin-Schaltfläche auf der Karte |
| Eintrag verwerfen | Papierkorb oder Entf auf einer fokussierten Karte |
| Alle Einträge verwerfen | Leeren; schließt Pins ein |
| Fenster verstecken | Esc oder X; App bleibt im Tray aktiv |
| App beenden | Tray-Menü → Beenden |

Die Suche umfasst den vollständigen Text. Pins werden von Alt+V übersprungen und bei automatischer Verdrängung geschützt, auch beim Verkleinern der Kapazität. Ein gezielter Klick entnimmt auch ein gepinntes Element.

Standardkapazität: 25; einstellbar: 10–100. Mehr Pins als die gewählte Kapazität bleiben erhalten; neue ungepinnte Inhalte können dann keinen freien Platz erhalten.

Neue Shortcuts werden zuerst reserviert. Schlägt das fehl, bleiben alte Einstellungen und Belegungen erhalten. „Endpunkt testen“ prüft die gerade eingegebene Bridge-Adresse, auch vor dem Speichern.

## E Links sammeln und Tools kombinieren

```flow LR
Kopierte URL -> copyNpaste: Exakte HTTP-/HTTPS-Adresse
copyNpaste -> Bookmarks Dynamic Manager: Localhost-Webhook
Bookmarks Dynamic Manager -> Today's Hunt: Eingang übernehmen
Today's Hunt -> Wissensdatenbank: Im Manager speichern und zuordnen
Wissensdatenbank --> aKlik-Deployer: Software-Projekte weiter untersuchen
```

Starte die native Bookmarks-App. Standardempfänger ist http://127.0.0.1:9911/api/todays-hunt. Kopiere anschließend eine reine URL. Der Versand läuft in einem eigenen Thread mit 2,5 Sekunden Timeout. Fehlt der Manager, bleibt die URL im Magazin; es gibt keinen automatischen erneuten Versand.

Erst nach erfolgreicher HTTP-Zustellung erscheint der grüne Haken. Windows versucht, ihn direkt unter der Textauswahl zu platzieren. Liefert die Quellanwendung keine Auswahlposition, erscheint er am Mauszeiger. Das Overlay nimmt keinen Fokus und fängt keine Klicks ab. Seine Sichtbarkeit endet nach etwa 280 Millisekunden. Ein HTTP-Erfolg belegt die Annahme durch den Empfänger, nicht jede nachfolgende Verarbeitung im Manager.

[Bookmarks Dynamic Manager](https://johi251.github.io/bookmarks-dynamic-manager/) und [aKlik-Deployer](https://aklik-deployer.pages.dev) besitzen eigene Funktionen und lokale Bestände. Ein gemeinsamer synchronisierter Speicher wird nicht behauptet.

## F Web und Browser-Erweiterung

| Funktion | Web-App | Erweiterung | Desktop |
|---|---|---|---|
| Aufnahme | Button, Strg+V auf der Seite, Bild-Drop | Textkopien auf normalen Webseiten, Popup oder Alt+Shift+Y | Betriebssystem-Clipboard |
| Inhalt | Text und Bilder | Text | Text, Bilder und Dateien |
| Entnahme | Klick oder Alt+V innerhalb der Seite | Klick oder Enter im Popup | Alt+V systemweit |
| Einfügen | Danach Strg+V im Ziel | Danach Strg+V im Ziel | Automatisches Tastensignal |
| Speicherung | IndexedDB, einschließlich Bilder | Browser-Speicher | Lokale JSON-Datei, einschließlich Bilder |
| Smart-Bridge | Kein nativer Zugriff | Kein nativer Zugriff | Lokaler Webhook |

Web und Erweiterung können aus Sicherheitsgründen keine systemweiten Tastensignale an andere Programme senden. Sie behalten ihre eigenen Magazine. Die Web-App migriert bestehende Textbestände aus der früheren localStorage-Version. Bei fehlender Clipboard-Freigabe bleibt der Eintrag erhalten und ein Fehler wird angezeigt. Textkopien in Passwortfeldern werden vom Add-on nicht automatisch erfasst; Betriebssystem-Ausschlussmarker sind im Browser nicht verfügbar.

## G Datenschutz und Speicherung

Das native Magazin und die Konfiguration liegen im Daten-/Konfigurationsverzeichnis der App de.johan.copynpaste. Unter Windows befindet es sich üblicherweise unter %APPDATA%/de.johan.copynpaste. Dateien: magazine.json, config.json und logs/copynpaste.log.

Inhalte werden lokal und unverschlüsselt gespeichert. Die Bridge sendet ausschließlich erkannte URLs an den konfigurierten Empfänger. Ein Passwort kann nur dann zuverlässig ausgeschlossen werden, wenn der Passwortmanager den passenden Windows-Clipboard-Marker setzt. Diese Erkennung ersetzt keine allgemeine Geheimniserkennung. Manuell kopierte Geheimnisse ohne Marker können im Magazin landen.

Pro Eintrag gelten 5 MiB Text und 20 MiB PNG. Das Speichern erfolgt im Hintergrund und ersetzt die Datei atomar. Beim regulären Beenden wird der neueste Stand geschrieben. Eine beschädigte Magazin-Datei wird nicht stillschweigend überschrieben; der Start stoppt zum Schutz des vorhandenen Inhalts. Unmittelbar vor einem Prozessabbruch noch nicht gespeicherte Änderungen können verloren gehen.

## H Prüfen und für Promotion zeigen

Eine kurze Vorführung zeigt: drei Dinge kopieren, das Magazin öffnen, zweimal Alt+V drücken, einen wichtigen Text anpinnen und einen GitHub-Link nach Today's Hunt senden. Alle gezeigten Links und Texte sollten eigene Vorführdaten sein.

```timeline
0–5 Sekunden | Problem | Einzelnes Clipboard überschreibt vorherige Kopien.
5–12 Sekunden | Sammeln | Text, Code und Screenshot erscheinen im Magazin.
12–20 Sekunden | Abarbeiten | Alt+V entnimmt passende Inhalte in umgekehrter Reihenfolge.
20–26 Sekunden | Wiederfinden | Volltextsuche und Pin zeigen gespeicherte Arbeit.
26–30 Sekunden | Verbinden | URL geht an Today's Hunt; Haken bestätigt die Zustellung.
```

Dies ist ein Storyboard für die später gewünschte Videoproduktion; es ist kein fertiges Promo-Video. Der vorhandene Entwicklungsauftrag verlangt die Funktionsausgabe und ein Gesamt-ZIP. Abrechnung, Testzeitbegrenzung und Lifetime-Lizenzen sind im älteren Ideendokument diskutiert, aber nicht als fertiges Geschäftsmodell spezifiziert; sie werden nicht in die Clipboard-App eingebaut.

Entwicklungsprüfungen: npm run check, npm test und cargo test --locked --manifest-path src-tauri/Cargo.toml. Die GitHub-Pipeline baut alle vier nativen Ausgaben und packt das Gesamt-ZIP nur, wenn alle Plattformjobs erfolgreich waren. Ein Buildnachweis ersetzt keine Fensterabnahme auf jedem System.
