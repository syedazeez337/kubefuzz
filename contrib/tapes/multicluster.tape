# KubeRift — multi-cluster: --all-contexts and ctrl-x context switching
# Output: docs/multicluster.gif

Output docs/multicluster.mp4
Output docs/multicluster.gif

Set Shell    "bash"
Set FontSize  14
Set Width    1200
Set Height    700
Set Theme    "Dracula"
Set TypingSpeed 60ms
Set Framerate  60
Set Padding  10

# ── All-contexts: stream from every cluster simultaneously ────────────────────
Type "kf --all-contexts pods"
Enter
Sleep 5s

# Both clusters visible, prefixed with cluster name and color-coded
Sleep 2s

# Filter to one cluster
Type "cilium"
Sleep 1.5s

# Clear — see both clusters again
Ctrl+u
Sleep 1.5s

Escape
Sleep 400ms

# ── Interactive context switching with ctrl-x ─────────────────────────────────
Type "kf --context kind-kuberift-dev"
Enter
Sleep 3.5s

# Open context picker
Ctrl+x
Sleep 1.5s

# Pick the cilium-test cluster
Down
Sleep 500ms
Enter
Sleep 4s

# Now watching cilium-test — header updates to new context
Sleep 1.5s

Escape
Sleep 500ms
