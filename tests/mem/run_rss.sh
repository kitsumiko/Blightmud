#!/usr/bin/env bash
# Local memory-harness sampler (feat/tabs-mem-bench branch only). Renders the
# fill template with N/K/W, runs one headless Blightmud as its own process, and
# reports peak VmHWM plus final VmRSS/VmData from /proc.
#   Usage: run_rss.sh <example-bin> <N> <K> [W]
set -euo pipefail
BIN="$1"; N="$2"; K="$3"; W="${4:-80}"
TMPL="$(dirname "$0")/fill_tabs.lua.tmpl"
SCRIPT="$(mktemp /tmp/fill_XXXXXX.lua)"
sed -e "s/__N__/$N/" -e "s/__K__/$K/" -e "s/__W__/$W/" "$TMPL" > "$SCRIPT"

"$BIN" "$SCRIPT" >/tmp/headless.out 2>&1 &
PID=$!
PEAK_HWM=0; LAST_RSS=0; LAST_DATA=0
while kill -0 "$PID" 2>/dev/null; do
  if [ -r "/proc/$PID/status" ]; then
    HWM=$(awk '/^VmHWM:/{print $2}'  "/proc/$PID/status" 2>/dev/null || echo 0)
    RSS=$(awk '/^VmRSS:/{print $2}'  "/proc/$PID/status" 2>/dev/null || echo 0)
    DAT=$(awk '/^VmData:/{print $2}' "/proc/$PID/status" 2>/dev/null || echo 0)
    [ -n "${HWM:-}" ] && [ "$HWM" -gt "$PEAK_HWM" ] && PEAK_HWM="$HWM"
    [ -n "${RSS:-}" ] && LAST_RSS="$RSS"
    [ -n "${DAT:-}" ] && LAST_DATA="$DAT"
  fi
  sleep 0.02
done
wait "$PID" 2>/dev/null || true
rm -f "$SCRIPT"
printf 'N=%-3s K=%-5s VmHWM_kB=%-9s last_VmRSS_kB=%-9s last_VmData_kB=%-9s\n' \
  "$N" "$K" "$PEAK_HWM" "$LAST_RSS" "$LAST_DATA"
