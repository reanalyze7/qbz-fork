#!/usr/bin/env bash
# QBZ Slint — LIVE-PREVIEW dev loop. Build once, then edit .slint files and see
# the change without any Rust rebuild.
#
# How it works, and why it is worth a dedicated script: slint-build normally
# generates a ~1.6M-line Rust module from ui/app.slint, and that single crate
# (qbz_ui) is where this project's build time and its ~30 GB rustc peak live.
# With SLINT_LIVE_PREVIEW=1 in the build environment plus the `live-preview`
# Cargo feature, slint-build emits stubs that load the .slint tree at RUNTIME
# instead. The expensive module is never produced.
#
# Consequence: as long as you do not change the Rust <-> UI bridge (globals,
# callbacks, struct fields), a .slint edit needs NO recompile at all.
# Changing the bridge does, because the Rust side references it by name.
#
# NOT SHIPPABLE. The binary reads ui/*.slint from the source tree at startup,
# so it only runs from this checkout. Use scripts/slint-run.sh for anything
# you intend to keep or hand to someone.
#
# Usage: ./scripts/slint-live.sh [extra app args]
#   FASTCC=1 ./scripts/slint-live.sh   # + Cranelift backend (see below)
#   RELEASE=1 ./scripts/slint-live.sh  # release profile instead of dev
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/.."

PROFILE_ARGS=()
[[ "${RELEASE:-0}" == 1 ]] && PROFILE_ARGS+=(--release)

# Cranelift trades generated-code quality for compile speed. Opt-in, never the
# default: this app decodes and resamples audio on a realtime thread, and that
# is exactly the kind of code a lower-quality backend degrades. Use it when you
# are iterating on UI wiring, not when you are judging playback.
#
# Set via ENV, deliberately, and never in crates/Cargo.toml. `codegen-backend`
# is an unstable profile key, and cargo rejects the WHOLE manifest when it sees
# one it cannot parse — so a checked-in `[profile.dev-fast]` broke the qbzd CI
# job, which builds this same workspace on STABLE for its glibc floor. Env vars
# only exist while this script runs, so stable cargo never sees them.
if [[ "${FASTCC:-0}" == 1 ]]; then
  if ! rustup component list --installed 2>/dev/null | grep -q rustc-codegen-cranelift; then
    echo "[slint-live] FASTCC=1 ignored — install the backend first:" >&2
    echo "  rustup component add rustc-codegen-cranelift-preview --toolchain nightly-2026-06-23" >&2
  elif [[ "${RELEASE:-0}" == 1 ]]; then
    echo "[slint-live] FASTCC=1 ignored: pointless with RELEASE=1 (you asked for" >&2
    echo "  optimised code, Cranelift's whole trade is giving that up)." >&2
  else
    export CARGO_UNSTABLE_CODEGEN_BACKEND=true
    export CARGO_PROFILE_DEV_CODEGEN_BACKEND=cranelift
    export CARGO_PROFILE_DEV_OPT_LEVEL=0
    echo "[slint-live] Cranelift backend — audio paths will be slower" >&2
  fi
fi

# The env var must be set for the BUILD (slint-build reads it), and the feature
# must be on for the RUNTIME (it pulls in the interpreter the stubs need).
# Either half alone gives you a broken build, not a faster one.
export SLINT_LIVE_PREVIEW=1
export RUSTFLAGS="${RUSTFLAGS:--C link-arg=-fuse-ld=mold}"

echo "[slint-live] live preview ON — .slint edits reload without a Rust rebuild" >&2
echo "[slint-live] the Rust <-> UI bridge (globals/callbacks/structs) still needs one" >&2

exec cargo run --manifest-path crates/Cargo.toml -p qbz \
  --features live-preview "${PROFILE_ARGS[@]}" -- "$@"
