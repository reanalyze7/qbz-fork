#!/usr/bin/env bash
# Map a release-channel branch to its channel identity, and mint the Debian
# package version for this build. Pure derivation from (branch, Cargo.toml,
# commit) — no side effects, so it can be run locally and is unit-tested by
# channel-meta.test.sh next to it.
#
# Version scheme (dpkg ordering is the whole point):
#   prod -> 2.0.2+prod.20260825.1430.gabc1234   '+' sorts ABOVE plain 2.0.2
#   int  -> 2.0.2~int.20260825.1430.gabc1234    '~' sorts BELOW plain 2.0.2
# So a machine on the prod channel never gets silently downgraded by the
# tagged 2.0.2 release, and an int box is always "older" than a real release
# of the same base version — which is exactly what that channel is: work in
# progress towards it.
#
# `main` is deliberately NOT a channel: it is the trunk int is cut from and
# prod is promoted from, and a push to it must build and publish nothing.
# Passing it here is an error, like any other non-channel branch.
set -euo pipefail

BRANCH="${1:?usage: channel-meta.sh <branch>}"
# Injectable for the tests; CI leaves them unset and gets the real values.
SHA="${CHANNEL_META_SHA:-${GITHUB_SHA:-$(git rev-parse HEAD)}}"
STAMP="${CHANNEL_META_STAMP:-$(date -u +%Y%m%d.%H%M)}"
MANIFEST="${CHANNEL_META_MANIFEST:-crates/Cargo.toml}"

case "$BRANCH" in
  prod) CHANNEL=prod; PRERELEASE=false; SEP='+' ;;
  int)  CHANNEL=int;  PRERELEASE=true;  SEP='~' ;;
  *)
    echo "::error::branch '$BRANCH' is not a release channel (prod|int)" >&2
    exit 1
    ;;
esac

BASE="$(grep -m1 '^version = ' "$MANIFEST" | sed 's/version = "\(.*\)"/\1/')"
if [ -z "$BASE" ]; then
  echo "::error::no 'version = ' line in $MANIFEST" >&2
  exit 1
fi
VERSION="${BASE}${SEP}${CHANNEL}.${STAMP}.g${SHA:0:7}"

emit() { echo "$1=$2"; [ -n "${GITHUB_OUTPUT:-}" ] && echo "$1=$2" >> "$GITHUB_OUTPUT"; return 0; }
emit channel "$CHANNEL"
emit prerelease "$PRERELEASE"
emit version "$VERSION"
emit tag "channel-${CHANNEL}"
