# KubeFuzz — preview mode cycling (describe → yaml → logs)
# Output: docs/preview.gif

Output docs/preview.gif

Set Shell    "bash"
Set FontSize  14
Set Width    1200
Set Height    700
Set Theme    "Dracula"
Set TypingSpeed 60ms
Set Padding  10

# ── Open pods, navigate to the running nginx pod ──────────────────────────────
Type "kf pods --context kind-kubefuzz-dev"
Enter
Sleep 3.5s

Type "pod-running"
Sleep 1.5s

# ── Mode 0: describe (default) ───────────────────────────────────────────────
# Preview pane shows full kubectl describe output
Sleep 2s

# ── Mode 1: YAML manifest ────────────────────────────────────────────────────
Ctrl+p
Sleep 2.5s

# ── Mode 2: live pod logs (last 100 lines) ───────────────────────────────────
Ctrl+p
Sleep 2.5s

# ── Back to Mode 0: describe ─────────────────────────────────────────────────
Ctrl+p
Sleep 1.5s

# Navigate to a crashloop pod — describe shows the back-off events
Ctrl+u
Sleep 300ms
Type "crash"
Sleep 1.5s

# Describe shows error events
Sleep 2s

# YAML shows the broken container spec
Ctrl+p
Sleep 2s

Escape
Sleep 500ms
