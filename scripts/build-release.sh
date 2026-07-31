#!/usr/bin/env bash
# Déclenche / suit les builds natifs via GitHub Actions (tauri-action).
#
#   ./scripts/build-release.sh            # lance le workflow (Win + macOS + Linux)
#   ./scripts/build-release.sh watch      # suit le run en cours
#   ./scripts/build-release.sh download   # télécharge les artefacts (.msi, .dmg, …)
#   ./scripts/build-release.sh macos      # build macOS local uniquement
#
# Windows : buildé sur une vraie machine windows-latest (recommandation Tauri).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

TARGET="${1:-ci}"

need_gh() {
  if ! command -v gh >/dev/null 2>&1; then
    echo "Erreur: installez GitHub CLI." >&2
    echo "  brew install gh && gh auth login" >&2
    exit 1
  fi
}

build_macos_local() {
  echo "==> Building macOS (local)…"
  npm run build
  npm run tauri -- build
  echo
  echo "Artefacts macOS :"
  find src-tauri/target -path '*/bundle/dmg/*.dmg' -o -path '*/bundle/macos/*.app' 2>/dev/null | sort -u || true
}

run_ci() {
  need_gh
  echo "==> Lancement du workflow build (Windows + macOS + Linux)…"
  gh workflow run build.yml
  echo
  echo "Ensuite :"
  echo "  npm run build:watch-ci"
  echo "  npm run build:download-ci"
}

case "$TARGET" in
  ci|all|windows|win)
    run_ci
    ;;
  macos|mac|osx|local)
    build_macos_local
    ;;
  watch)
    need_gh
    gh run watch
    ;;
  download)
    need_gh
    gh run download
    ;;
  *)
    echo "Usage: $0 {ci|macos|watch|download}" >&2
    exit 1
    ;;
esac
