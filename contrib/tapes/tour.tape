# KubeRift — comprehensive product tour
#
# Duration:  ~4 minutes
# Outputs:   docs/tour.mp4   (primary — high quality H.264)
#            docs/tour.gif   (fallback for platforms that don't support video)
#
# Prerequisites:
#   1. Two kind clusters:
#        kind-kuberift-dev   (always required)
#        kind-cilium-test    (for Section 5; skip ctrl-x demo if absent)
#   2. Test workloads deployed (see docs/TESTING.md)
#   3. kf on PATH:  export PATH="$PWD/target/release:$PATH"
#
# Record:
#   cargo build --release
#   export PATH="$PWD/target/release:$PATH"
#   vhs contrib/tapes/tour.tape

Output docs/tour.mp4
Output docs/tour.gif

Set Shell       "bash"
Set FontSize    13
Set Width       1400
Set Height      800
Set Theme       "Dracula"
Set TypingSpeed 50ms
Set Framerate   60
Set Padding     16

# ── Shell setup: minimal prompt ─────────────────────────────────────────────
Hide
Type "export PS1='$ '"
Enter
Type "clear"
Enter
Show

# ══════════════════════════════════════════════════════════════════════════════
#  INTRO  ─  what kf is and why it exists
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "printf '\\n  \\033[1;96mkf\\033[0m  --  fuzzy-first Kubernetes resource navigator\\n\\n  \\033[90m* Instantly surfaces unhealthy resources across every namespace.\\n  * Fuzzy-search, preview (describe / YAML / logs), and act\\n    without leaving the terminal.\\033[0m\\n'"
Enter
Show
Sleep 4s

# ══════════════════════════════════════════════════════════════════════════════
#  SECTION 1  ─  Launch + unhealthy-first sort
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear && printf '\\n  \\033[1;33m[ 1 / 5 ]  Launch and Unhealthy-First Sort\\033[0m\\n  \\033[90mBroken pods float to the top automatically -- no filter needed.\\033[0m\\n'"
Enter
Sleep 2.5s
Type "clear"
Enter
Show

Type "kf pods --context kind-kuberift-dev"
Enter
Wait+Screen@14s /res:pod/
Sleep 1.5s

# Type "crash" to surface the CrashLoopBackOff pod immediately
Type "crash"
Wait+Screen@5s /CrashLoop/
Sleep 2.5s

# Clear filter — unhealthy pods remain first in the full list
Ctrl+u
Sleep 2s

# Scroll to show the diversity of pod statuses
Down 3
Sleep 400ms
Down 3
Sleep 800ms
Up 6
Sleep 1.5s

Escape
Sleep 1s

# ══════════════════════════════════════════════════════════════════════════════
#  SECTION 2  ─  Fuzzy filtering
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear && printf '\\n  \\033[1;33m[ 2 / 5 ]  Fuzzy Filtering\\033[0m\\n  \\033[90mType any substring -- matches name, namespace, kind, or status.\\033[0m\\n'"
Enter
Sleep 2.5s
Type "clear"
Enter
Show

Type "kf --context kind-kuberift-dev"
Enter
Wait+Screen@14s /res:all/
Sleep 1s

# Filter: surface the crashloop pod
Type "crash"
Sleep 2s

# Switch filter: find secrets across all namespaces
Ctrl+u
Sleep 300ms
Type "secret"
Sleep 2s

# Scope to a namespace
Ctrl+u
Sleep 300ms
Type "testbed"
Sleep 2s

# Show all resources again
Ctrl+u
Sleep 1.5s

Escape
Sleep 1s

# Narrow within a resource type using a short alias
Type "kf svc --context kind-kuberift-dev"
Enter
Wait+Screen@10s /res:svc/
Sleep 1.5s

# Filter to NodePort services only
Type "node"
Sleep 1.5s

Escape
Sleep 1s

# ══════════════════════════════════════════════════════════════════════════════
#  SECTION 3  ─  Preview modes (ctrl-p cycles describe → YAML → logs)
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear && printf '\\n  \\033[1;33m[ 3 / 5 ]  Preview Modes\\033[0m\\n  \\033[90mctrl-p cycles: describe -> YAML -> logs -> describe.\\033[0m\\n'"
Enter
Sleep 2.5s
Type "clear"
Enter
Show

Type "kf pods --context kind-kuberift-dev"
Enter
Wait+Screen@14s /res:pod/
Sleep 500ms

# Filter to the running pod — preview pane shows describe by default
Type "pod-running"
Sleep 1.5s

# Mode 0 (default): kubectl describe
Sleep 3s

# Mode 1: raw YAML manifest
Ctrl+p
Sleep 3s

# Mode 2: live pod logs (last 100 lines)
Ctrl+p
Sleep 3s

# Cycle back to describe
Ctrl+p
Sleep 1s

# Switch to crashloop pod — describe shows the back-off events
Ctrl+u
Sleep 300ms
Type "crash"
Wait+Screen@5s /CrashLoop/
Sleep 1.5s

# Describe reveals the CrashLoopBackOff events
Sleep 3s

# YAML shows the broken container image/command
Ctrl+p
Sleep 3s

Escape
Sleep 1s

# ══════════════════════════════════════════════════════════════════════════════
#  SECTION 4  ─  Built-in actions
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear && printf '\\n  \\033[1;33m[ 4 / 5 ]  Built-In Actions\\033[0m\\n  \\033[90mctrl-l logs  ctrl-y YAML dump  ctrl-r rollout restart  ctrl-d delete\\033[0m\\n'"
Enter
Sleep 2.5s
Type "clear"
Enter
Show

Type "kf pods --context kind-kuberift-dev"
Enter
Wait+Screen@14s /res:pod/
Sleep 500ms
Type "pod-running"
Sleep 1.5s

# ctrl-l: stream last 200 lines of logs to the terminal
Ctrl+l
Sleep 4s

# skim re-opens automatically after the log stream finishes
Wait+Screen@8s /res:pod/
Sleep 500ms
Escape
Sleep 1s

# Rollout restart a deployment — use Sleep for load time (follows actions.tape pattern)
Type "kf deploy --context kind-kuberift-dev"
Enter
Sleep 4s

Type "deploy-healthy"
Sleep 1.5s

# ctrl-r: kubectl rollout restart + live rollout status tracking
Ctrl+r
Sleep 16s

# skim re-opens once rollout completes
Escape
Sleep 1s

# Safe delete with confirmation
Type "kf pods --context kind-kuberift-dev"
Enter
Wait+Screen@14s /res:pod/
Sleep 500ms
Type "crash"
Wait+Screen@5s /CrashLoop/
Sleep 1.5s

# Tab to select the pod
Tab
Sleep 700ms

# ctrl-d triggers the confirmation prompt
Ctrl+d
Sleep 1s

# "Delete 1 resource? [y/N]" — cancel to demonstrate the safety guard
Type "n"
Enter
Sleep 1.5s

# Pod is still here — safe
Escape
Sleep 1s

# ══════════════════════════════════════════════════════════════════════════════
#  SECTION 5  ─  Multi-cluster support
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear && printf '\\n  \\033[1;33m[ 5 / 5 ]  Multi-Cluster Support\\033[0m\\n  \\033[90m--all-contexts streams every cluster simultaneously.  ctrl-x switches live.\\033[0m\\n'"
Enter
Sleep 2.5s
Type "clear"
Enter
Show

# Stream from every cluster at once
Type "kf --all-contexts pods"
Enter
Wait+Screen@16s /res:pod/
Sleep 2s

# Filter to one cluster by name
Type "cilium"
Sleep 1.5s

# Show both clusters again
Ctrl+u
Sleep 1.5s

Escape
Sleep 1s

# Live context switch inside the TUI
Type "kf --context kind-kuberift-dev"
Enter
Wait+Screen@14s /res:all/
Sleep 1s

# Open the context picker
Ctrl+x
Sleep 1.5s

# Select the cilium-test cluster
Down
Sleep 500ms
Enter
Sleep 5s

# Header now shows the new context — all resources from cilium-test
Sleep 1.5s

Escape
Sleep 1s

# ══════════════════════════════════════════════════════════════════════════════
#  OUTRO  ─  installation
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear"
Enter
Type "printf '\\n  \\033[1;32mInstall kf:\\033[0m\\n\\n    \\033[96mcargo install kuberift\\033[0m\\n    \\033[96mbrew install kf\\033[0m\\n\\n  \\033[90mgithub.com/syedazeez337/kuberift\\033[0m\\n'"
Enter
Show
Sleep 5s
