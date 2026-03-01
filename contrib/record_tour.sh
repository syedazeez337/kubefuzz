#!/usr/bin/env bash
# record_tour.sh — resource-aware runner for the KubeRift tour tape.
#
# Why this exists:
#   Running vhs directly competes for CPU/RAM with the browser, compositor,
#   and everything else on your desktop. This script lowers the process
#   priority, checks prerequisites, and runs a single tape (MP4 only —
#   no GIF encoding, which is the main cause of system hangs).
#
# Usage:
#   cd /path/to/kuberift
#   bash contrib/record_tour.sh

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TAPE="contrib/tapes/tour.tape"
OUTPUT="docs/media/tour.mp4"

cd "$PROJECT_DIR"

# ── Color helpers ─────────────────────────────────────────────────────────────
ok()   { printf '\033[1;32m  ✔  %s\033[0m\n' "$*"; }
info() { printf '\033[1;34m  →  %s\033[0m\n' "$*"; }
warn() { printf '\033[1;33m  !  %s\033[0m\n' "$*"; }
die()  { printf '\033[1;31m  ✖  %s\033[0m\n' "$*" >&2; exit 1; }

echo
printf '\033[1;96m  KubeRift tour recorder\033[0m\n'
echo

# ── Prerequisite checks ───────────────────────────────────────────────────────

# kf binary
if ! command -v kf &>/dev/null; then
  if [[ -f "$PROJECT_DIR/target/release/kf" ]]; then
    export PATH="$PROJECT_DIR/target/release:$PATH"
    ok "kf found at target/release/kf — added to PATH"
  else
    info "kf not found — building now (cargo build --release) …"
    cargo build --release
    export PATH="$PROJECT_DIR/target/release:$PATH"
    ok "build complete"
  fi
else
  ok "kf on PATH: $(command -v kf)"
fi

# vhs
if ! command -v vhs &>/dev/null; then
  die "vhs not found. Install with: go install github.com/charmbracelet/vhs@latest"
fi
ok "vhs: $(vhs --version 2>/dev/null | head -1)"

# ffmpeg (vhs uses it internally — verify it is present)
if ! command -v ffmpeg &>/dev/null; then
  die "ffmpeg not found. Install with your package manager (dnf install ffmpeg / apt install ffmpeg)."
fi
ok "ffmpeg: $(ffmpeg -version 2>&1 | head -1 | awk '{print $3}')"

# kind cluster
if ! kubectl config get-contexts kind-kuberift-dev &>/dev/null 2>&1; then
  warn "kind-kuberift-dev context not found in kubeconfig."
  warn "Section 1-4 require this cluster. See docs/guides/TESTING.md."
  warn "Continuing — demo mode (Section 6) will still work."
else
  ok "kind-kuberift-dev context present"
fi

# Output directory
mkdir -p "$(dirname "$OUTPUT")"

# ── Resource snapshot before recording ───────────────────────────────────────
echo
info "System state before recording:"
free -h | grep Mem | awk '{printf "    RAM: %s used / %s total\n", $3, $2}'
printf "    Load: %s\n" "$(cut -d' ' -f1-3 /proc/loadavg)"
echo

# ── Warn about desktop apps ───────────────────────────────────────────────────
info "Tip: close your browser and any heavy apps before recording."
info "Even with nice priority, a loaded system produces dropped frames."
echo

# ── Countdown ────────────────────────────────────────────────────────────────
info "Starting in 5 seconds — switch to a quiet desktop/workspace now …"
for i in 5 4 3 2 1; do
  printf '\r  \033[1;33m%d\033[0m ' "$i"
  sleep 1
done
printf '\r  \033[1;32mRecording …\033[0m\n\n'

# ── Record ────────────────────────────────────────────────────────────────────
# nice -n 10   → CPU priority below normal desktop apps (range: -20 high to 19 low)
# ionice -c 3  → idle I/O class: only uses disk when no other process needs it
# Together these prevent vhs from starving your compositor/mouse/audio.

START=$(date +%s)

nice -n 10 ionice -c 3 vhs "$TAPE"

END=$(date +%s)
ELAPSED=$(( END - START ))
MINS=$(( ELAPSED / 60 ))
SECS=$(( ELAPSED % 60 ))

echo
ok "Recording complete in ${MINS}m ${SECS}s"
ok "Output: $PROJECT_DIR/$OUTPUT"

# File size
SIZE=$(du -sh "$OUTPUT" 2>/dev/null | cut -f1)
info "File size: $SIZE"

# Duration via ffprobe if available
if command -v ffprobe &>/dev/null; then
  DUR=$(ffprobe -v error -show_entries format=duration \
        -of default=noprint_wrappers=1:nokey=1 "$OUTPUT" 2>/dev/null | \
        awk '{m=int($1/60); s=int($1%60); printf "%dm %ds", m, s}')
  info "Video duration: $DUR"
fi

echo
info "To preview:"
printf '    mpv %s\n' "$OUTPUT"
info "To upload to LinkedIn — use this file directly:"
printf '    %s\n' "$PROJECT_DIR/$OUTPUT"
echo
