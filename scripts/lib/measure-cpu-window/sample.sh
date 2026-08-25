# measure-cpu-window.sh — the banner and the sampling loop.
# Sourced by scripts/measure-cpu-window.sh. Function definitions only.
#
# run_sampler deliberately declares samples_qbz / samples_wk_total /
# samples_total WITHOUT `local`: the summary and CSV stages in the entry
# script read them after this returns. `prev_wk` is `declare -A`, which IS
# function-local, and that is correct — nothing outside the loop reads it.

print_header() {
  printf '%s\n' "=== qbz CPU sampler ==="
  printf 'label:     %s\n' "$LABEL"
  printf 'qbz pid:   %s\n' "$QBZ_PID"
  if (( ${#WEBKIT_PIDS[@]} > 0 )); then
    printf 'webkit:    %s\n' "${WEBKIT_PIDS[*]}"
  else
    printf 'webkit:    (none found — is the GUI window open?)\n'
  fi
  printf 'duration:  %ss\n' "$DURATION"
  printf 'window:    %s (persisted)\n' "$(read_window_size)"
  printf 'log:       %s\n' "$LOG_CSV"
  printf '\nSampling — leave the window UNTOUCHED for the full window.\n\n'
}

run_sampler() {
  # Take initial snapshot
  prev_qbz="$(read_jiffies "$QBZ_PID")"
  declare -A prev_wk
  for pid in "${WEBKIT_PIDS[@]:-}"; do
    [[ -z "$pid" ]] && continue
    prev_wk[$pid]="$(read_jiffies "$pid")"
  done
  prev_ts="$(date +%s.%N)"

  samples_qbz=()
  samples_wk_total=()
  samples_total=()

  printf '%4s  %8s  %8s  %8s\n' "t/s" "qbz%" "webkit%" "total%"
  printf '%4s  %8s  %8s  %8s\n' "---" "----" "-------" "------"

  for (( t=1; t<=DURATION; t++ )); do
    sleep 1
    now_ts="$(date +%s.%N)"
    dt="$(awk -v a="$now_ts" -v b="$prev_ts" 'BEGIN{printf "%.4f", a-b}')"
    prev_ts="$now_ts"

    cur_qbz="$(read_jiffies "$QBZ_PID")"
    d_qbz=$(( cur_qbz - prev_qbz ))
    prev_qbz="$cur_qbz"
    pct_qbz="$(awk -v j="$d_qbz" -v dt="$dt" -v hz="$CLK_TCK" 'BEGIN{ if (dt<=0) print 0; else printf "%.1f", (j/hz)/dt*100 }')"

    wk_total=0
    for pid in "${WEBKIT_PIDS[@]:-}"; do
      [[ -z "$pid" ]] && continue
      if [[ ! -r "/proc/$pid/stat" ]]; then
        continue
      fi
      cur="$(read_jiffies "$pid")"
      d=$(( cur - ${prev_wk[$pid]:-$cur} ))
      prev_wk[$pid]="$cur"
      (( d < 0 )) && d=0
      wk_total=$(( wk_total + d ))
    done
    pct_wk="$(awk -v j="$wk_total" -v dt="$dt" -v hz="$CLK_TCK" 'BEGIN{ if (dt<=0) print 0; else printf "%.1f", (j/hz)/dt*100 }')"

    pct_total="$(awk -v a="$pct_qbz" -v b="$pct_wk" 'BEGIN{ printf "%.1f", a+b }')"

    samples_qbz+=("$pct_qbz")
    samples_wk_total+=("$pct_wk")
    samples_total+=("$pct_total")

    printf '%4d  %8s  %8s  %8s\n' "$t" "$pct_qbz" "$pct_wk" "$pct_total"
  done
}
