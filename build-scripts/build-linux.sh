#!/usr/bin/env bash
# copyNpaste — Linux-Produktionsbuild (.deb)
# Voraussetzungen (Debian/Ubuntu):
#   sudo apt install -y libxdo-dev libwebkit2gtk-4.1-dev libgtk-3-dev \
#                       libayatana-appindicator3-dev librsvg2-dev
set -euo pipefail
cd "$(dirname "$0")/.."
echo "=== copyNpaste: bun install ==="
bun install --frozen-lockfile
echo "=== copyNpaste: tauri build (Linux) ==="
npm run tauri build -- --bundles deb
echo
echo "FERTIG. Artefakt:"
echo "  src-tauri/target/release/bundle/deb/copyNpaste_1.2.0_amd64.deb"
