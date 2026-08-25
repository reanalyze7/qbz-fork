#!/usr/bin/env bash
# measure-cpu-window.sh — sample CPU% of QBZ + WebKit children over a window.
#
# Usage: ./scripts/measure-cpu-window.sh [label] [duration_seconds]
#
# The method, the output format and the cross-run comparison recipe are in
# scripts/README.md. Process discovery, the /proc readers, the sampling loop
# and the statistics live in scripts/lib/measure-cpu-window/.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIB="$ROOT/scripts/lib/measure-cpu-window"
# shellcheck source=lib/measure-cpu-window/discover.sh
. "$LIB/discover.sh"
# shellcheck source=lib/measure-cpu-window/procstat.sh
. "$LIB/procstat.sh"
# shellcheck source=lib/measure-cpu-window/sample.sh
. "$LIB/sample.sh"
# shellcheck source=lib/measure-cpu-window/stats.sh
. "$LIB/stats.sh"

LABEL="${1:-run-$(date +%H%M%S)}"
DURATION="${2:-30}"
LOG_CSV="/tmp/qbz-cpu-measurements.csv"
CLK_TCK="$(getconf CLK_TCK)"

if [[ ! "$DURATION" =~ ^[0-9]+$ ]] || (( DURATION < 5 )); then
  echo "error: duration must be a positive integer >= 5" >&2
  exit 2
fi

QBZ_PID="$(find_qbz_pid || true)"
if [[ -z "${QBZ_PID:-}" ]]; then
  echo "error: no qbz process found. Is the app running?" >&2
  exit 1
fi

WEBKIT_PIDS=()
mapfile -t WEBKIT_PIDS < <(find_webkit_pids)

print_header
run_sampler

echo ""
echo "=== Summary ==="
summary_qbz="$(summarize "qbz"     "${samples_qbz[@]}"       | tail -1)"

read -r mean_qbz p50_qbz p95_qbz max_qbz <<< "$(echo "$summary_qbz" | tr ',' ' ')"

mean_wk=0 p50_wk=0 p95_wk=0 max_wk=0
if (( ${#samples_wk_total[@]} > 0 )); then
  summary_wk="$(summarize "webkit"  "${samples_wk_total[@]}"  | tail -1)"
  read -r mean_wk p50_wk p95_wk max_wk <<< "$(echo "$summary_wk" | tr ',' ' ')"
fi

summary_total="$(summarize "total"   "${samples_total[@]}"     | tail -1)"
read -r mean_total p50_total p95_total max_total <<< "$(echo "$summary_total" | tr ',' ' ')"


# summarize is pure (one CSV line); this prints the human-readable table.
print_summary_table

# CSV header if first run
if [[ ! -f "$LOG_CSV" ]]; then
  echo "timestamp,label,duration_s,window_size,qbz_mean,qbz_p50,qbz_p95,qbz_max,webkit_mean,webkit_p50,webkit_p95,webkit_max,total_mean,total_p50,total_p95,total_max" > "$LOG_CSV"
fi

WINDOW="$(read_window_size)"
echo "$(date -Iseconds),$LABEL,$DURATION,$WINDOW,$mean_qbz,$p50_qbz,$p95_qbz,$max_qbz,$mean_wk,$p50_wk,$p95_wk,$max_wk,$mean_total,$p50_total,$p95_total,$max_total" >> "$LOG_CSV"

echo ""
echo "Appended to $LOG_CSV"
echo "Compare runs:  cat $LOG_CSV | column -t -s,"
