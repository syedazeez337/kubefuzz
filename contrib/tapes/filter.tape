# KubeRift — resource type and namespace filtering
# Output: docs/filter.gif

Output docs/filter.mp4
Output docs/filter.gif

Set Shell    "bash"
Set FontSize  14
Set Width    1200
Set Height    700
Set Theme    "Dracula"
Set TypingSpeed 60ms
Set Framerate  60
Set Padding  10

# ── Resource type: pods only ──────────────────────────────────────────────────
Type "kf pods --context kind-kuberift-dev"
Enter
Sleep 3.5s
Escape
Sleep 400ms

# ── Resource type: deployments only ──────────────────────────────────────────
Type "kf deploy --context kind-kuberift-dev"
Enter
Sleep 3s
Escape
Sleep 400ms

# ── Resource type: services only ──────────────────────────────────────────────
Type "kf svc --context kind-kuberift-dev"
Enter
Sleep 3s
Escape
Sleep 400ms

# ── Namespace filter: testbed only ────────────────────────────────────────────
Type "kf -n testbed --context kind-kuberift-dev"
Enter
Sleep 3.5s

# Further fuzzy-narrow within the namespace
Type "pod"
Sleep 1.5s

Ctrl+u
Sleep 400ms
Type "job"
Sleep 1.5s

Escape
Sleep 500ms
