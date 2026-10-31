#!/usr/bin/env bash
# One-command dev runner: starts Vite (frontend) and builds/runs the Tauri app.
# Equivalent to `npm run tauri dev`, kept as a script for convenience.
set -euo pipefail
cd "$(dirname "$0")"
npm run tauri dev
