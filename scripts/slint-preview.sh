#!/usr/bin/env bash
# Render the real views with mock data, live, with NO Rust build.
#
# --auto-reload watches the file system: edit any .slint under crates/qbz-ui/
# and the window redraws. That is the whole point — layout and theming become
# a sub-second loop instead of a CI round trip.
#
# What it shows honestly: layout, sizing, theming, elision, virtualization,
# whether a list actually lists. What it cannot: anything a Rust callback
# computes, which here is every action. The buttons are inert by design.
#
# Usage: ./scripts/slint-preview.sh
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/.."

command -v slint-viewer >/dev/null || {
  echo "slint-viewer absent — cargo install slint-viewer --version 1.16.1" >&2
  exit 127
}

echo "[slint-preview] auto-reload actif — édite un .slint, la fenêtre se redessine" >&2
exec slint-viewer --auto-reload crates/qbz-ui/preview/preview.slint
