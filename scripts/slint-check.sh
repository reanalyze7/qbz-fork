#!/usr/bin/env bash
# Compile the whole .slint tree locally, in seconds, with NO cargo build.
#
# This is the feedback loop the project was missing: every UI change used to
# wait on a ~55-minute CI build to find out whether it even compiled, so a
# misplaced brace cost an hour. `slint-viewer` embeds the SAME compiler the
# project pins (1.16.1), so what it accepts, slint-build accepts.
#
# How the exit codes map, because it is not obvious: a compile error makes
# slint-viewer exit 255 straight away with a diagnostic. Anything that
# compiles opens a window instead and keeps running, so we kill it — that
# shows up as 124, which is SUCCESS here. There is no headless backend in
# this build (SLINT_BACKEND=testing falls back and opens a window anyway).
#
# The 25s default is not a compile budget — it is how long we let a SUCCESSFUL
# run sit in its window before killing it. Errors never wait: they exit 255 at
# once. Raise SLINT_CHECK_TIMEOUT only if a cold machine reports a false ok.
#
# Usage: ./scripts/slint-check.sh [extra .slint files]
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/.."

command -v slint-viewer >/dev/null || {
  echo "slint-viewer absent — cargo install slint-viewer --version 1.16.1" >&2
  exit 127
}

TIMEOUT="${SLINT_CHECK_TIMEOUT:-25}"
targets=(crates/qbz-ui/ui/app.slint crates/qbz-ui/preview/preview.slint "$@")
fail=0

for t in "${targets[@]}"; do
  [ -f "$t" ] || { echo "  skip   $t (absent)"; continue; }
  out=$(timeout "$TIMEOUT" slint-viewer "$t" 2>&1)
  rc=$?
  if [ "$rc" -eq 124 ] || [ "$rc" -eq 0 ]; then
    echo "  ok     $t"
  else
    echo "  FAILED $t (exit $rc)"
    echo "$out" | sed 's/^/         /'
    fail=1
  fi
done

[ "$fail" -eq 0 ] && echo "slint: tout compile" || echo "slint: échec" >&2
exit "$fail"
