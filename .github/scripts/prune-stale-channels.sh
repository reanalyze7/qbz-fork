#!/usr/bin/env bash
# Delete the channel releases whose channel no longer exists.
#
# A channel release is a ROLLING tag (`channel-<name>`): nothing ever
# supersedes it, so when a channel is retired its release just sits there
# forever, at the top of the release list, offering a .deb that no build
# will ever refresh. `channel-main` is exactly that since main stopped
# being a channel (2026-08-27).
#
# The list of live channels is not duplicated here: channel-meta.sh already
# IS that gate — it maps a branch to its channel identity and rejects
# everything else — so this script asks it, and a future channel change
# stays a one-line edit in one file.
#
# Dry run by default. Nothing is deleted without --apply.
#
#   .github/scripts/prune-stale-channels.sh              # show what would go
#   .github/scripts/prune-stale-channels.sh --apply      # actually delete
#
# Needs an authenticated `gh` (gh auth login) with contents:write.
set -euo pipefail
# BASH_SOURCE, not $0: the test file sources this script to reach stale_tags,
# and under `source` $0 is still the CALLER's path — which would send this cd
# somewhere else entirely.
cd "$(dirname "${BASH_SOURCE[0]}")/../.."

# Pure: tags on stdin -> the channel-* ones with no live channel behind them.
# Split out so the selection can be tested without gh or a network.
stale_tags() {
  local tag branch
  while read -r tag; do
    [ -n "$tag" ] || continue
    case "$tag" in channel-*) ;; *) continue ;; esac
    branch="${tag#channel-}"
    if ! .github/scripts/channel-meta.sh "$branch" >/dev/null 2>&1; then
      printf '%s\n' "$tag"
    fi
  done
}

# Sourced by the test file, which wants stale_tags and nothing else. Before
# the argument parsing: `source` inherits the caller's positional parameters.
[ "${PRUNE_CHANNELS_LIB:-0}" = "1" ] && return 0

APPLY=0
REPO_ARGS=()
while [ $# -gt 0 ]; do
  case "$1" in
    --apply) APPLY=1 ;;
    --repo)  REPO_ARGS=(--repo "${2:?--repo needs a value}"); shift ;;
    -h|--help) sed -n '2,20p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

command -v gh >/dev/null || { echo "gh is not installed" >&2; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "gh is not authenticated — run: gh auth login" >&2; exit 1; }

STALE=$(gh release list --limit 100 --json tagName --jq '.[].tagName' "${REPO_ARGS[@]}" | stale_tags)

if [ -z "$STALE" ]; then
  echo "Nothing to prune: every channel-* release still has a live channel."
  exit 0
fi

echo "Stale channel releases (channel retired, build will never refresh them):"
printf '  %s\n' $STALE

if [ "$APPLY" != "1" ]; then
  echo
  echo "Dry run. Re-run with --apply to delete the releases AND their tags."
  exit 0
fi

for tag in $STALE; do
  echo "==> deleting release and tag $tag"
  gh release delete "$tag" --yes --cleanup-tag "${REPO_ARGS[@]}"
done
echo "Done."
