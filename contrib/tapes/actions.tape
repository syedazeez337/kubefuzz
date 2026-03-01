# KubeRift — actions: logs (ctrl-l), yaml (ctrl-y), rollout restart (ctrl-r)
# Output: docs/actions.gif

Output docs/actions.mp4
Output docs/actions.gif

Set Shell    "bash"
Set FontSize  14
Set Width    1200
Set Height    700
Set Theme    "Dracula"
Set TypingSpeed 60ms
Set Framerate  60
Set Padding  10

# ── Stream pod logs with ctrl-l ───────────────────────────────────────────────
Type "kf pods --context kind-kuberift-dev"
Enter
Sleep 3.5s

Type "pod-running"
Sleep 1.5s

# ctrl-l: print last 200 lines of logs to terminal
Ctrl+l
Sleep 3.5s

# skim relaunches automatically after action

# ── Print YAML to stdout with ctrl-y ─────────────────────────────────────────
Type "pod-running"
Sleep 1.5s

# ctrl-y: dump full YAML manifest to stdout
Ctrl+y
Sleep 3s

# skim relaunches
Escape
Sleep 400ms

# ── Rollout restart a deployment with ctrl-r ──────────────────────────────────
Type "kf deploy --context kind-kuberift-dev"
Enter
Sleep 3s

Type "deploy-healthy"
Sleep 1.5s

# ctrl-r: kubectl rollout restart + tracks rollout status to completion
Ctrl+r
Sleep 15s

# rollout completes, skim relaunches
Escape
Sleep 500ms
