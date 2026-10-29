"""Startprüfung ausschließlich auf frischen CI-Rechnern, ohne Tastatureingaben."""
import json
import os
import subprocess
import sys
import time
from pathlib import Path

root = Path(__file__).resolve().parent.parent
assert os.environ.get('GITHUB_ACTIONS') == 'true', 'Nur auf isolierten GitHub-Runnern ausführen'
identifier = 'de.johan.copynpaste'
if sys.platform == 'win32':
    config_dir = data_dir = Path(os.environ['APPDATA']) / identifier
    binary = root / 'src-tauri/target/release/copynpaste.exe'
elif sys.platform == 'darwin':
    config_dir = data_dir = Path.home() / 'Library/Application Support' / identifier
    binary = root / 'src-tauri/target/release/bundle/macos/copyNpaste.app/Contents/MacOS/copynpaste'
else:
    config_dir = Path(os.environ.get('XDG_CONFIG_HOME', str(Path.home() / '.config'))) / identifier
    data_dir = Path(os.environ.get('XDG_DATA_HOME', str(Path.home() / '.local/share'))) / identifier
    binary = root / 'src-tauri/target/release/copynpaste'
config = config_dir / 'config.json'
assert not config.exists(), 'Vorhandene Konfiguration wird nicht angefasst'
assert binary.is_file(), f'Programm fehlt: {binary}'
config_dir.mkdir(parents=True, exist_ok=True)
config.write_text(json.dumps({'capacity': 25, 'pasteShortcut': 'alt+v', 'toggleShortcut': 'alt+shift+v',
    'bridgeEnabled': False, 'bridgeUrl': 'http://127.0.0.1:9911/api/todays-hunt', 'effects': True}), encoding='utf-8')
process = subprocess.Popen([str(binary)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
try:
    deadline = time.monotonic() + 25
    ready_at = None
    while time.monotonic() < deadline:
        assert process.poll() is None, f'App vorzeitig beendet: {process.returncode}'
        log = data_dir / 'logs/copynpaste.log'
        text = log.read_text(encoding='utf-8') if log.exists() else ''
        if all(marker in text for marker in ['v1.2.0 startet', 'Clipboard-Watcher gestartet', 'Tray-Icon eingerichtet']):
            ready_at = ready_at or time.monotonic()
            if time.monotonic() - ready_at >= 3:
                print('Native startup verified: process, version, clipboard watcher, tray; no keyboard input')
                break
        time.sleep(0.25)
    else:
        raise AssertionError('Startnachweise fehlen nach 25 Sekunden')
finally:
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
