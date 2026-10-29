"""Packt nur freigegebene Projektdateien und die tatsächlich gebauten Plattformen."""
import hashlib
import json
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VERSION = json.loads((ROOT / 'package.json').read_text(encoding='utf-8'))['version']
OUT = ROOT / 'dist-delivery'


def extension(firefox=False):
    manifest = json.loads((ROOT / 'web-extension/manifest.json').read_text(encoding='utf-8'))
    if firefox:
        manifest['background'] = {'scripts': ['background.js']}
        manifest['permissions'].remove('offscreen')
        manifest.pop('minimum_chrome_version', None)
    path = OUT / f'copyNpaste-{VERSION}-{"firefox" if firefox else "chromium"}.zip'
    with zipfile.ZipFile(path, 'w', zipfile.ZIP_DEFLATED) as z:
        for file in sorted((ROOT / 'web-extension').rglob('*')):
            if not file.is_file() or file.name == 'manifest.json':
                continue
            z.write(file, file.relative_to(ROOT / 'web-extension'))
        z.writestr('manifest.json', json.dumps(manifest, ensure_ascii=False, indent=2))
    with zipfile.ZipFile(path) as z:
        assert z.testzip() is None
        assert json.loads(z.read('manifest.json'))['version'] == VERSION
    return path


def main():
    OUT.mkdir(exist_ok=True)
    extensions = [extension(), extension(True)]
    source = OUT / f'copyNpaste-{VERSION}-source.zip'
    subprocess.run(['git', 'archive', '--format=zip', '-o', str(source), 'HEAD'], cwd=ROOT, check=True)
    native = ROOT / 'release-artifacts'
    platforms = ['windows-x64', 'macos-intel', 'macos-apple-silicon', 'linux-x64']
    for platform in platforms:
        files = list((native / f'copyNpaste-{platform}').rglob('*'))
        suffix = '.exe' if platform.startswith('windows') else '.deb' if platform.startswith('linux') else '.dmg'
        assert any(f.is_file() and f.suffix == suffix for f in files), f'Plattform fehlt: {platform}'
    complete = OUT / f'copyNpaste-{VERSION}-All-Platforms.zip'
    with zipfile.ZipFile(complete, 'w', zipfile.ZIP_DEFLATED) as z:
        for file in sorted(native.rglob('*')):
            if file.is_file(): z.write(file, 'Desktop/' + file.relative_to(native).as_posix())
        for file in sorted((ROOT / 'web-app').rglob('*')):
            if file.is_file(): z.write(file, 'Web-App/' + file.relative_to(ROOT / 'web-app').as_posix())
        for file in extensions: z.write(file, 'Browser-Add-ons/' + file.name)
        z.write(source, 'Quellcode/' + source.name)
        z.write(ROOT / 'README.md', 'START-HIER.md')
    with zipfile.ZipFile(complete) as z:
        assert z.testzip() is None
        assert 'Web-App/manual.html' in z.namelist()
    checksums = OUT / f'copyNpaste-{VERSION}-SHA256.txt'
    checksums.write_text(''.join(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n' for p in [*extensions, source, complete]), encoding='utf-8')
    print(complete)


if __name__ == '__main__':
    main()
