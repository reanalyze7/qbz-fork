# measure-cpu-window.sh — process discovery: the QBZ main process and
# its WebKit children. Sourced by scripts/measure-cpu-window.sh.
# Function definitions only.

# Find QBZ main process — match both legacy binary name (qbz-nix, dev builds)
# and the production binary (qbz). Prefer the largest-RSS match to dodge
# wrapper shells.
find_qbz_pid() {
  local candidates pid best_rss=0 best_pid=""
  candidates="$(pgrep -f 'target/(debug|release)/qbz(-nix)?$|/usr/(local/)?bin/qbz$|^qbz$' 2>/dev/null || true)"
  if [[ -z "$candidates" ]]; then
    candidates="$(pgrep -x qbz 2>/dev/null || true)"
  fi
  if [[ -z "$candidates" ]]; then
    candidates="$(pgrep -f 'qbz-nix' 2>/dev/null || true)"
  fi
  for pid in $candidates; do
    [[ -r "/proc/$pid/status" ]] || continue
    local rss
    rss="$(awk '/^VmRSS:/ {print $2}' "/proc/$pid/status" 2>/dev/null || echo 0)"
    if (( rss > best_rss )); then
      best_rss=$rss
      best_pid=$pid
    fi
  done
  [[ -n "$best_pid" ]] && echo "$best_pid"
}

# All WebKit child processes. WebKitWebProcess is the one that does CSS
# layout/paint, so its CPU% is the most relevant for backdrop-filter cost.
# WebKitNetworkProcess does I/O — usually low CPU.
find_webkit_pids() {
  pgrep -f 'WebKit(Web|Network|GPU)Process' 2>/dev/null || true
}
