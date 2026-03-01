#!/usr/bin/env bash
# Runs all VHS tapes sequentially with nice priority.
# Logs progress and resource snapshots to /tmp/tape_run.log

set -euo pipefail

PROJECT="/home/aze/projects/kuberift"
LOG="/tmp/tape_run.log"
TAPES=(
  "contrib/tapes/filter.tape"
  "contrib/tapes/delete.tape"
  "contrib/tapes/preview.tape"
  "contrib/tapes/actions.tape"
  "contrib/tapes/multicluster.tape"
  "contrib/tapes/demo.tape"
  "contrib/kf.tape"
  "contrib/tapes/tour.tape"
)

cd "$PROJECT"
export PATH="$PROJECT/target/release:$PATH"

log() { echo "[$(date '+%H:%M:%S')] $*" | tee -a "$LOG"; }
snapshot() {
  echo "--- resource snapshot ---" >> "$LOG"
  free -h >> "$LOG"
  vmstat 1 1 >> "$LOG"
  echo "load: $(cat /proc/loadavg)" >> "$LOG"
  echo "-------------------------" >> "$LOG"
}

log "=== KubeRift VHS tape runner starting ==="
log "Tapes to run: ${#TAPES[@]}"
snapshot

PASS=0
FAIL=0
FAILED_TAPES=()

for tape in "${TAPES[@]}"; do
  log ">>> START: $tape"
  snapshot

  if nice -n 15 vhs "$tape" >> "$LOG" 2>&1; then
    log "<<< PASS: $tape"
    PASS=$((PASS + 1))
  else
    log "<<< FAIL: $tape (exit $?)"
    FAIL=$((FAIL + 1))
    FAILED_TAPES+=("$tape")
  fi

  snapshot
  # Brief pause between tapes so the system breathes
  sleep 3
done

log "=== Done: $PASS passed, $FAIL failed ==="
if [[ ${#FAILED_TAPES[@]} -gt 0 ]]; then
  log "Failed tapes:"
  for t in "${FAILED_TAPES[@]}"; do log "  - $t"; done
fi
