# KubeRift demo recording
# Requires: vhs (https://github.com/charmbracelet/vhs)
#
# Prerequisites:
#   1. A kind cluster named "kind-kuberift-dev" with test workloads
#      (see docs/TESTING.md for the full test cluster setup)
#   2. kf binary on PATH: export PATH="$PWD/target/release:$PATH"
#
# Usage:
#   cargo build --release
#   export PATH="$PWD/target/release:$PATH"
#   vhs contrib/kf.tape
#
# Output: docs/demo.gif

Output docs/demo.mp4
Output docs/demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set TypingSpeed 80ms
Set Framerate 60
Set Padding 10

# ── Open kf against the dev cluster ──────────────────────────────────────────
Type "kf --context kind-kuberift-dev"
Enter
Sleep 3s

# ── Fuzzy filter: type "crash" to surface CrashLoopBackOff pods ──────────────
Type "crash"
Sleep 1500ms

# ── Cycle preview: describe → yaml → logs ────────────────────────────────────
Ctrl+p
Sleep 1s
Ctrl+p
Sleep 1s
Ctrl+p
Sleep 800ms

# ── Clear filter and show unhealthy-first sort ────────────────────────────────
Ctrl+u
Sleep 500ms
Type "pod-"
Sleep 800ms
Down
Down
Sleep 500ms

# ── Clear and filter by namespace ────────────────────────────────────────────
Escape
Sleep 200ms
Type "kf -n testbed --context kind-kuberift-dev"
Enter
Sleep 2500ms
Escape
Sleep 200ms

# ── Demo mode (no cluster) ───────────────────────────────────────────────────
Type "KUBECONFIG=/nonexistent kf"
Enter
Sleep 2s
Escape
Sleep 500ms
