---
lang: de
title: copyNpaste 1.2.0 – Fertigstellung und Prüfung
template: doc
theme: paper
---

# copyNpaste 1.2.0 – Fertigstellung und Prüfumfang

Stand: 29.10.2026. Implementierungsstand: `7ea86973660d73b78e277053a9bc58784f698ddb`.

## Anforderungsabgleich

Die 16 Nutzerbeiträge der öffentlich freigegebenen [ZCode-Unterhaltung](https://zcode.z.ai/share/luoLzbm_iVjCnB4u3dLxoetcxlZdlD1K) wurden gelesen. Neben dem ursprünglichen Desktop-Auftrag verlangt der Verlauf ausdrücklich Windows, Mac, Browser-Add-on, Web-App auf GitHub und eine ZIP mit allen Ausgaben. Die vier später vorgeschlagenen Ergänzungen waren Persistenz, Pins, Passwortmanager-Marker und Quick-Paste. Diese sind im Projekt enthalten.

Das ergänzende Ideendokument nennt außerdem ein Handbuch, Beispiele, Workflow-Grafiken, dezente Verweise auf die verwandten Tools und eine spätere Videoproduktion. Ein visuelles Handbuch und ein Video-Storyboard sind enthalten. Es wurde kein fertiges Promo-Video behauptet. Abonnementpreise, Lifetime-Lizenzen und Testzeitsperren waren offene Ideen; sie wurden nicht eigenmächtig als Geschäftsmodell eingebaut.

## Behobene Fehler

| Befund der Projektprüfung | Umsetzung |
|---|---|
| Vorzeitige Entnahme bei Paste-Fehlern | Element bleibt bis zum erfolgreichen Clipboard-Schreiben und Tastensignal erhalten; Fehler werden zurückgegeben. |
| Veränderte Einrückungen/Zeilenenden | Originaltext bleibt erhalten; kein trim() des gespeicherten Inhalts. |
| Verlust von Pins beim Verkleinern | Aufnahme und Einstellungen verwenden denselben Pin-schonenden Kürzungspfad. |
| Erneutes Kopieren verbrauchter Inhalte verworfen | Duplikatprüfung bezieht sich auf den tatsächlich vorhandenen jüngsten ungepinnten Eintrag. |
| Konkurrierende oder unterbrochene Speicherung | Ein einzelner Hintergrund-Schreibpfad mit atomarem Dateiersatz; abschließendes Speichern beim regulären Beenden. |
| Bilder über vier MiB nicht gespeichert | Alle akzeptierten Bilder bis zur Erfassungsgrenze werden gespeichert. |
| Unbelegter Bridge-Erfolg nach Neustart | Versandstatus wird gespeichert; fehlende alte Bestätigungen werden als unbekannt angezeigt. |
| Shortcut-Fehler hinterlassen kaputte Einstellungen | Neue Belegungen zuerst reservieren; bei Fehlern alte Konfiguration erhalten. |
| Endpunkt-Test verwendet alte Adresse | Der Test bekommt die aktuelle Formularadresse. |
| Suche nach Zeichen 600 erfolglos | Suche umfasst den vollständigen Text. |
| Clipboard-Zugriff im Add-on-Service-Worker | Chromium nutzt ein Offscreen-Dokument; Firefox ein eigenes Hintergrund-Manifest. |
| Gleichzeitige Paste-Aufrufe | Native Aufrufe werden serialisiert, einschließlich einer Wartezeit zum Lesen durch das Zielfenster. |
| Veraltete Add-on-Speicherstände überschreiben neue Kopien | Aufnahme, Entnahme und Leeren laufen im Hintergrund über denselben seriellen Schreibpfad. |

Zusätzlich: einfache Zielfelderkennung unter Windows/macOS, fokusfreies Erfolgs-Overlay, dauerhafte Web-Bildspeicherung über IndexedDB, sichtbare Fehlermeldungen sowie getrennte Chromium-/Firefox-Pakete.

## Nachweise

- `executed`: TypeScript-Prüfung, vier JavaScript-Regressionstests und 15 Rust-Tests bestanden. Die JavaScript-Tests führen tatsächlichen Web-/Add-on-Code mit isolierten Browser-Schnittstellen aus.
- `live verified`: Der [endgültige Hauptzweig-Lauf](https://github.com/Johi251/copyNpaste/actions/runs/40831253264) baute und startete alle vier nativen Ausgaben. Prozess, Versionslog, Clipboard-Watcher und Tray-Initialisierung wurden geprüft; es wurden keine Tastensignale in fremde Programme gesendet.
- `executed`: Alle fünf Jobs des endgültigen Hauptzweig-Laufs sind erfolgreich, einschließlich des Gesamtpakets. Geprüft wurden exakt der genannte Implementierungsstand und die daraus erzeugten Artefakte. Spätere Änderungen dieses Dokuments und des statischen Hosting-Markers ändern keinen Anwendungscode.
- `live verified`: Öffentliche Startseite, app.js und Handbuch liefern HTTP 200 und entsprechen nach Normalisierung der Zeilenenden den lokalen Web-Dateien.
- `executed`: Paketierung prüft beide Add-on-Manifeste, alle vier nativen Plattformen und die Integrität des Gesamt-ZIP; SHA256-Prüfsummen werden erzeugt.

## Lieferumfang

Die [GitHub-Releases](https://github.com/Johi251/copyNpaste/releases) erhalten Windows-EXE und NSIS-Setup, Mac-Intel-DMG, Mac-Apple-Silicon-DMG, Linux-x64-DEB, Chromium- und Firefox-ZIP, Quellcode-ZIP, Gesamt-ZIP und Prüfsummen.

Das Gesamt-ZIP enthält die echten nativen Artefakte, Web-App und visuelles Handbuch, beide Erweiterungsvarianten und den Quellcode. Die [Web-App](https://johi251.github.io/copyNpaste/) und das [Handbuch](https://johi251.github.io/copyNpaste/manual.html) sind separat online.

## Ausdrückliche Grenzen der Abnahme

- `not verified`: Echte Alt+V-Bedienung, Clipboard-Übertragung von Bildern/Dateien in verschiedene Zielprogramme, Fokuswechsel und Installer-/Gatekeeper-Dialoge wurden nicht vollständig interaktiv abgenommen. Der lokale Bedienhelfer lieferte trotz ausgewähltem Editorfenster Browserbilder. Es wurden darüber keine Eingaben gesendet.
- `blocked`: Zwei lokale Starttest-Befehle wurden vor Ausführung von der automatischen Prüfung mit „blocked by policy“ abgelehnt; eine nähere Begründung wurde nicht ausgegeben. Die nativen Startnachweise stammen stattdessen aus frischen isolierten GitHub-Runnern. Der private App-Datenordner wurde nicht verschoben oder für die Prüfung ausgelesen.
- Die Zielfeldentscheidung ist an verfügbare Accessibility-Metadaten gebunden. Linux verwendet weiterhin die normale Stapelreihenfolge; Rich-Text- und nicht unterstützte Anwendungen können nicht universell klassifiziert werden. Ein gesendetes Tastensignal ist keine Empfangsbestätigung der Zielanwendung.
- Das Overlay sitzt unter der Auswahl, sofern Windows deren Position liefert; andernfalls am Mauszeiger. Es kann nicht in jeder fremden Anwendung exakt am Link-Anfang platziert werden.
- Web und Add-on besitzen getrennte Magazine und senden keine systemweiten Tastensignale. Die native Smart-Bridge gehört zur Desktop-App.
- Die Programme sind nicht kostenpflichtig codesigniert/notarisiert. Das Firefox-Paket ist für temporäre Installation vorbereitet; permanente Verteilung benötigt Mozilla-Signierung.
- Magazin-Inhalte liegen lokal unverschlüsselt. Passwortmanager-Schutz hängt von deren Markern ab; Browser kennen diese Betriebssystem-Marker nicht.

**Urteil:** Die dokumentierten Fehler sind im Code behoben und durch gezielte Tests sowie plattformübergreifende Builds abgesichert. Eine universelle Funktionsgarantie für jedes Zielfenster oder eine vollständig bestandene interaktive Gesamtabnahme wird ausdrücklich nicht behauptet.
