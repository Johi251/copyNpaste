#!/usr/bin/env bash
# copyNpaste — macOS-Produktionsbuild (.app + .dmg)
# Voraussetzungen: Node.js, Rust, Xcode Command Line Tools
set -euo pipefail
cd "$(dirname "$0")/.."
echo "=== copyNpaste: bun install ==="
bun install --frozen-lockfile
echo "=== copyNpaste: tauri build (macOS) ==="
npm run tauri build -- --bundles app,dmg
echo
echo "FERTIG. Artefakte:"
echo "  src-tauri/target/release/bundle/macos/copyNpaste.app"
echo "  src-tauri/target/release/bundle/dmg/copyNpaste_1.2.0_*.dmg"
