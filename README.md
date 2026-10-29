# copyNpaste 1.2.0 — Clipboard-Magazin

Text, Code, Bilder und Dateien sammeln und mit **Alt+V** rückwärts einfügen. Native Tauri-2-App für Windows, Mac Intel, Mac Apple Silicon und Linux. Dazu eine Web-App und ein Browser-Add-on.

- [Downloads und Gesamt-ZIP](https://github.com/Johi251/copyNpaste/releases/latest)
- [Web-App](https://johi251.github.io/copyNpaste/)
- [Visuelles Handbuch und Workflows](https://johi251.github.io/copyNpaste/manual.html)
- [Build- und Testläufe](https://github.com/Johi251/copyNpaste/actions)

## Desktop benutzen

1. Windows-Setup installieren oder `copynpaste.exe` starten. Auf dem Mac das passende DMG öffnen und die App in Programme ziehen. Auf Linux das DEB installieren.
2. Die App läuft im System-Tray. Kopiere Text, ein Bild oder Dateien; das Magazin hat standardmäßig 25 Plätze (einstellbar 10–100).
3. **Alt+V** sendet das jüngste passende ungepinnte Element an das aktive Zielfenster. Erst nach erfolgreichem Clipboard-Schreiben und Tastensignal wird es entnommen. **Alt+Shift+V** öffnet das Magazin.
4. Suche nach vollständigem Text, füge eine bestimmte Karte mit Klick/Enter ein oder pinne sie für später an. Pins überleben automatische Verdrängung und Kapazitätsänderungen. Gezieltes Einfügen und „Leeren“ schließen Pins ein.
5. Schließen/Esc versteckt das Fenster. Beenden erfolgt über das Tray-Menü und schreibt den neuesten Stand.

Beide globalen Shortcuts sind einstellbar und reagieren nach dem Loslassen der Tasten. Ein Registrierungsfehler lässt die bisherigen Einstellungen erhalten.

## Zielfeld und Feedback

Windows liest ausschließlich Accessibility-Metadaten des fokussierten Elements. Bei erkannten einfachen Textfeldern werden Bilder/Dateien übersprungen; sie bleiben im Magazin. Auf macOS werden einfache AX-Textfelder erkannt. Rich-Text-Editoren und Anwendungen ohne passende Metadaten verwenden die normale Stapelreihenfolge. Linux hat noch keine AT-SPI-Zielfelderkennung. Die endgültige Annahme eines Formats entscheidet die Zielanwendung; ein gesendetes Tastensignal ist keine universelle Empfangsbestätigung.

Reine HTTP-/HTTPS-URLs gehen in einem eigenen Thread an `http://127.0.0.1:9911/api/todays-hunt` (2,5 Sekunden Timeout, konfigurierbar/abschaltbar). Dafür muss die native App **Bookmarks Dynamic Manager** laufen. Erst nach HTTP-Erfolg erscheint ein fokusfreier Haken für ungefähr 280 ms: Windows nutzt die Position der Auswahl, sofern die Quelle sie liefert; sonst erscheint er am Mauszeiger. Die Anzeige in der Magazin-Karte bleibt zusätzlich vorhanden.

Webhook-Vertrag:

```json
{"url":"https://github.com/tauri-apps/tauri","source":"copyNpaste","capturedAt":"2026-10-29T12:00:00+01:00","isGithub":true,"hint":"software-repository"}
```

„Endpunkt testen“ sendet `{ "probe": true, "source": "copyNpaste" }` an die eingegebene Adresse. Ist der Empfänger offline, bleibt die URL im Magazin. Es gibt keinen automatischen erneuten Versand. Alte URL-Einträge ohne gespeicherte Zustellbestätigung erhalten keinen erfundenen Erfolgsstatus.

## Browser und Web

Das Chromium-ZIP entpacken und als entpackte Erweiterung laden. Firefox erhält ein eigenes Manifest für temporäres Laden über `about:debugging`; eine dauerhafte Installation verlangt Mozilla-Signierung. Textkopien auf normalen Webseiten werden automatisch erfasst. **Alt+Shift+Y** übernimmt die Zwischenablage über einen unterstützten Dokumentkontext. Das Popup erlaubt Suche und Entnahme; nach dem Zurückkopieren im Ziel **Strg+V** drücken.

Die Web-App nimmt Text/Bilder per Button, Strg+V außerhalb der Suchfelder oder Bild-Drop auf. **Alt+V innerhalb der Seite** oder ein Kartenklick kopiert und entnimmt ein Element. Danach im Zielfeld Strg+V drücken. IndexedDB speichert Text und Bilder; vorhandene Textbestände der alten Version werden übernommen. Fehler beim Clipboard-Zugriff oder Speichern werden angezeigt.

Browser, Web und Desktop besitzen getrennte lokale Magazine. Browser können keine systemweiten Paste-Tastensignale in fremde Anwendungen senden. Die native Smart-Bridge und Dateilisten gehören zur Desktop-Ausgabe. Für Web-Dateien im Gesamt-ZIP einen lokalen HTTP-Server verwenden, beispielsweise `python -m http.server --directory Web-App 8080`.

## Datenschutz und Grenzen

- Magazin und Konfiguration werden lokal **unverschlüsselt** gespeichert. Die Bridge übermittelt erkannte URLs an den eingestellten Endpunkt.
- Windows-Ausschlussmarker von Passwortmanagern werden beachtet. Geheimnisse ohne Marker können trotzdem erfasst werden. Die Browser-Erweiterung erfasst keine Textauswahl aus Passwortfeldern; sie kennt keine Betriebssystem-Marker.
- Pro Eintrag maximal 5 MiB Text beziehungsweise 20 MiB PNG. Alle akzeptierten Bilder werden gespeichert. Dateien werden als Pfade gespeichert, nicht dupliziert.
- Atomare Hintergrundspeicherung und abschließendes Speichern beim Beenden. Bei beschädigter JSON-Datei stoppt der Start, statt den Bestand zu überschreiben. Ein abrupter Prozessabbruch kann noch nicht gespeicherte letzte Änderungen verlieren.
- Windows benötigt WebView2; macOS benötigt Bedienungshilfen-Rechte für Tastensimulation. Linux: X11/XWayland. Rein natives Wayland wird nicht als unterstützt ausgegeben.
- Die Builds sind nicht kostenpflichtig codesigniert/notarisiert. Eine erfolgreiche CI-Kompilierung ist keine vollständige Installer-, Gatekeeper- oder Fensterabnahme.

Unter Windows liegen `magazine.json`, `config.json` und `logs/copynpaste.log` normalerweise unter `%APPDATA%/de.johan.copynpaste`. Andere Plattformen verwenden das jeweilige Tauri-Datenverzeichnis.

## Entwicklung und Prüfung

Benötigt: Node 20+, Bun, Rust 1.90+; Windows C++ Build Tools, macOS Xcode Command Line Tools beziehungsweise Linux GTK/WebKit/X11-Bibliotheken.

```sh
bun install --frozen-lockfile
npm run check
npm test
cargo test --locked --manifest-path src-tauri/Cargo.toml
npm run tauri dev
npm run tauri build -- --bundles nsis
```

Die GitHub-Pipeline prüft TypeScript, JavaScript-Regressionen und Rust-Tests auf vier nativen Plattformen. Alle müssen erfolgreich sein, bevor `scripts/package-release.py` das Gesamt-ZIP aus echten EXE-/DMG-/DEB-Artefakten, beiden Add-on-Varianten, Web-App, Handbuch und Quellcode erzeugt. SHA256-Prüfsummen liegen daneben. Der Release-Upload erfolgt erst nach erfolgreicher Prüfung.

Plattform-Buildwrapper liegen in `build-scripts/`; macOS verwendet `--bundles app,dmg`, Linux `--bundles deb`. Der Windows-Wrapper baut NSIS. Die JavaScript-Tests führen den tatsächlichen Web-/Add-on-Code isoliert aus; sie ersetzen keinen echten Browserlauf.

## Verwandte Tools und Anforderungsquelle

[Bookmarks Dynamic Manager](https://johi251.github.io/bookmarks-dynamic-manager/) übernimmt URLs in Today's Hunt. [aKlik-Deployer](https://aklik-deployer.pages.dev) untersucht Software-Repositories und Deployment-Wege. Die Werkzeuge bleiben eigenständige Anwendungen im aKlik-Ökosystem.

Der [freigegebene ZCode-Verlauf](https://zcode.z.ai/share/luoLzbm_iVjCnB4u3dLxoetcxlZdlD1K) verlangt ausdrücklich Windows, Mac, Browser-Add-on, Web-App auf GitHub und ein Gesamt-ZIP. Das [Handbuch](docs/HANDBUCH.md) dokumentiert Bedienung, Workflows, Grenzen und ein Storyboard für das später gewünschte Promo-Video. Bezahlmodelle und Testzeitbegrenzungen wurden lediglich als Ideen diskutiert und nicht eingebaut.
