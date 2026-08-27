#!/usr/bin/env bash
# Tests for channel-meta.sh. Run: .github/scripts/channel-meta.test.sh
#
# The version scheme is the part worth testing: a wrong separator silently
# ships a package that dpkg refuses to upgrade to (or, worse, that shadows a
# real release), and that only shows up on a user's machine days later.
set -uo pipefail
cd "$(dirname "$0")/../.."

export CHANNEL_META_SHA=abc1234567890
export CHANNEL_META_STAMP=20260825.1430
MANIFEST=$(mktemp); printf 'version = "2.0.2"\n' > "$MANIFEST"
export CHANNEL_META_MANIFEST="$MANIFEST"

pass=0; fail=0
check() { # check <label> <expected> <actual>
  if [ "$2" = "$3" ]; then pass=$((pass+1)); else
    fail=$((fail+1)); echo "FAIL: $1"; echo "  expected: $2"; echo "  actual:   $3"
  fi
}
meta() { .github/scripts/channel-meta.sh "$1" 2>/dev/null | grep "^$2=" | cut -d= -f2-; }

# --- channel / prerelease mapping -------------------------------------
check "prod channel"      prod    "$(meta prod channel)"
check "int channel"       int     "$(meta int channel)"
check "prod is not a pre" false   "$(meta prod prerelease)"
check "int is a pre"      true    "$(meta int prerelease)"

# --- rolling tags -----------------------------------------------------
check "prod tag" channel-prod "$(meta prod tag)"
check "int tag"  channel-int  "$(meta int tag)"

# --- version minting --------------------------------------------------
check "prod version" "2.0.2+prod.20260825.1430.gabc1234" "$(meta prod version)"
check "int version"  "2.0.2~int.20260825.1430.gabc1234"  "$(meta int version)"

# --- dpkg ordering is the reason for the separators -------------------
if command -v dpkg >/dev/null 2>&1; then
  dpkg --compare-versions "$(meta prod version)" gt 2.0.2 \
    && pass=$((pass+1)) || { fail=$((fail+1)); echo "FAIL: prod must sort above 2.0.2"; }
  dpkg --compare-versions "$(meta int version)" lt 2.0.2 \
    && pass=$((pass+1)) || { fail=$((fail+1)); echo "FAIL: int must sort below 2.0.2"; }
else
  echo "note: dpkg absent, ordering assertions skipped"
fi

# --- a non-channel branch must be rejected, not guessed ---------------
# `main` is in this list on purpose: it used to be a channel, and the whole
# point of this change is that a push to it now publishes nothing.
for branch in feature/whatever main pre-release; do
  if .github/scripts/channel-meta.sh "$branch" >/dev/null 2>&1; then
    fail=$((fail+1)); echo "FAIL: non-channel branch '$branch' was accepted"
  else
    pass=$((pass+1))
  fi
done

rm -f "$MANIFEST"
echo "channel-meta: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
