#!/usr/bin/env bash
# copyNpaste — macOS-Produktionsbuild (.app + .dmg)
# Voraussetzungen: Node.js, Rust, Xcode Command Line Tools
set -euo pipefail
cd "$(dirname "$0")/.."
echo "=== copyNpaste: npm install ==="
npm install --no-fund --no-audit
echo "=== copyNpaste: tauri build (macOS) ==="
npm run tauri build -- --bundles app,dmg
echo
echo "FERTIG. Artefakte:"
echo "  src-tauri/target/release/bundle/macos/copyNpaste.app"
echo "  src-tauri/target/release/bundle/dmg/copyNpaste_1.0.0_aarch64.dmg"
