# KubeRift — main showcase
# Output: docs/demo.gif

Output docs/demo.mp4
Output docs/demo.gif

Set Shell    "bash"
Set FontSize  14
Set Width    1200
Set Height    700
Set Theme    "Dracula"
Set TypingSpeed 60ms
Set Framerate  60
Set Padding  10

# ── Launch kf ─────────────────────────────────────────────────────────────────
Type "kf --context kind-kuberift-dev"
Enter
Sleep 4s

# Unhealthy resources (red/yellow) auto-sort to top — no filter needed
# CrashLoopBackOff, ImagePullBackOff, Error pods are immediately visible
Sleep 2s

# ── Fuzzy filter to CrashLoop pod ─────────────────────────────────────────────
Type "crash"
Sleep 2s

# ── Preview: describe (default) already showing ───────────────────────────────
Sleep 500ms

# ── Preview: cycle → YAML manifest ───────────────────────────────────────────
Ctrl+p
Sleep 2s

# ── Preview: cycle → pod logs ─────────────────────────────────────────────────
Ctrl+p
Sleep 2s

# ── Preview: cycle back → describe ────────────────────────────────────────────
Ctrl+p
Sleep 500ms

# ── Clear filter: see full sorted list ────────────────────────────────────────
Ctrl+u
Sleep 1s

Down
Down
Down
Sleep 1s

# ── Quit and show namespace-scoped view ───────────────────────────────────────
Escape
Sleep 400ms

Type "kf -n testbed --context kind-kuberift-dev"
Enter
Sleep 3.5s

Type "svc"
Sleep 1.5s

Ctrl+u
Sleep 400ms
Type "deploy"
Sleep 1.5s

Escape
Sleep 500ms
