# KubeRift — safe delete with confirmation (multi-select + ctrl-d)
# Output: docs/delete.gif

Output docs/delete.mp4
Output docs/delete.gif

Set Shell    "bash"
Set FontSize  14
Set Width    1200
Set Height    700
Set Theme    "Dracula"
Set TypingSpeed 60ms
Set Framerate  60
Set Padding  10

# ── Open kf and filter to a specific pod ─────────────────────────────────────
Type "kf pods --context kind-kuberift-dev"
Enter
Sleep 3.5s

# Navigate to pod-crashloop (bad pod — natural delete candidate)
Type "crash"
Sleep 1.5s

# ── Multi-select with Tab ────────────────────────────────────────────────────
Tab
Sleep 700ms

# ── Trigger delete with ctrl-d ────────────────────────────────────────────────
Ctrl+d
Sleep 1s

# Confirmation prompt: "Delete 1 resource? [y/N]"
# Cancel — demonstrates the safety guard
Type "n"
Enter
Sleep 1s

# skim relaunches — pod is still here
Type "crash"
Sleep 1.5s

# ── Now show multi-select: Tab two pods and cancel ────────────────────────────
Ctrl+u
Sleep 300ms

# Select first item with Tab
Tab
Sleep 400ms
Down
Sleep 300ms
Tab
Sleep 400ms

# ctrl-d shows "Delete 2 resources?"
Ctrl+d
Sleep 1s

# Cancel again
Type "n"
Enter
Sleep 1s

# skim relaunches
Escape
Sleep 500ms
