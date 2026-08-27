#!/usr/bin/env bash
# Tests for prune-stale-channels.sh's selection logic. Run:
#   .github/scripts/prune-stale-channels.test.sh
#
# The selection is the dangerous half — this script deletes releases and
# their tags. A false positive would take down a live channel's install URL,
# so `channel-int`/`channel-prod` surviving, and `v*` releases never being
# considered at all, are the assertions that matter.
set -uo pipefail
cd "$(dirname "$0")/../.."

PRUNE_CHANNELS_LIB=1 . .github/scripts/prune-stale-channels.sh

pass=0; fail=0
check() { # check <label> <expected> <actual>
  if [ "$2" = "$3" ]; then pass=$((pass+1)); else
    fail=$((fail+1)); echo "FAIL: $1"; echo "  expected: [$2]"; echo "  actual:   [$3]"
  fi
}
stale() { printf '%s\n' "$@" | stale_tags | tr '\n' ' ' | sed 's/ $//'; }

# --- live channels must never be selected -----------------------------
check "int survives"  "" "$(stale channel-int)"
check "prod survives" "" "$(stale channel-prod)"

# --- a retired channel is selected ------------------------------------
check "main is stale"        "channel-main"        "$(stale channel-main)"
check "pre-release is stale" "channel-pre-release" "$(stale channel-pre-release)"

# --- version releases are not channel releases ------------------------
check "v tags ignored" "" "$(stale v2.0.0 v2.0.2 v1.9.9-rc1)"

# --- a realistic mixed listing ----------------------------------------
check "mixed listing" "channel-main" \
  "$(stale v2.0.2 channel-prod channel-int channel-main)"

# --- empty input is not an error --------------------------------------
check "empty input" "" "$(printf '' | stale_tags | tr -d '\n')"

echo "prune-stale-channels: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
