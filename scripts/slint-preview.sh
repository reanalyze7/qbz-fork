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

# slint-viewer does NOT read QBZ_RENDERER — that variable is the app's own,
# parsed by qbz's renderer_select. The viewer picks a Slint backend, so the
# preview must be pointed at one explicitly or it renders text differently
# from the app and you end up judging the wrong thing.
#
# winit-software matches what the app should be running. On Linux the app
# compiles only renderer-femtovg-wgpu and renderer-femtovg (Skia is macOS
# only), so its `wgpu` and `gl` tiers share ONE text rasteriser — femtovg's —
# and only `software` uses a different one. That is why software measured
# sharper here, and why switching wgpu/gl changes nothing about blur.
export SLINT_BACKEND="${SLINT_BACKEND:-winit-software}"

echo "[slint-preview] backend=$SLINT_BACKEND — auto-reload actif" >&2
exec slint-viewer --auto-reload crates/qbz-ui/preview/preview.slint
