# KubeRift — complete product tour
# ─────────────────────────────────────────────────────────────────────────────
# Single video. All features. MP4 only — no GIF (GIF encoding causes hangs).
#
# Clusters:  kind-kubefuzz-dev (primary)  +  kind-cilium-test (multi-cluster)
# Workloads: testbed/pod-crashloop  testbed/pod-running  testbed/deploy-healthy
#
# Record:  bash contrib/record_tour.sh

Output docs/media/tour.mp4

Set Shell       "bash"
Set FontSize    14
Set Width       1280
Set Height      720
Set Theme       "Dracula"
Set TypingSpeed 55ms
Set Framerate   30
Set Padding     20

# ── Clean prompt ─────────────────────────────────────────────────────────────
Hide
Type "export PS1='$ ' && clear"
Enter
Show

# ══════════════════════════════════════════════════════════════════════════════
#  TITLE CARD
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "printf '\\n  \\033[1;96mkf\\033[0m  —  fuzzy-first Kubernetes\\n\\n  \\033[90mOpen it. Start typing. Find anything.\\n  Works across every resource type and namespace simultaneously.\\033[0m\\n'"
Enter
Show
Sleep 3.5s

# ══════════════════════════════════════════════════════════════════════════════
#  SECTION 1 — Launch + unhealthy-first sort
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear && printf '\\n  \\033[1;33m[ 1 / 6 ]  Unhealthy-first sort\\033[0m\\n  \\033[90mCrashLoopBackOff pods and degraded resources float to the top on launch.\\033[0m\\n'"
Enter
Sleep 3s
Type "clear"
Enter
Show

Type "kf --context kind-kubefuzz-dev"
Enter
Sleep 5s

Down 3
Sleep 500ms
Up 3
Sleep 2s

Escape
Sleep 1s

# ══════════════════════════════════════════════════════════════════════════════
#  SECTION 2 — Fuzzy search
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear && printf '\\n  \\033[1;33m[ 2 / 6 ]  Fuzzy search\\033[0m\\n  \\033[90mType any fragment — name, status, namespace, kind — matched simultaneously.\\033[0m\\n'"
Enter
Sleep 3s
Type "clear"
Enter
Show

Type "kf --context kind-kubefuzz-dev"
Enter
Sleep 5s

Type "crash"
Sleep 3s

Ctrl+u
Sleep 400ms
Type "testbed"
Sleep 2.5s

Ctrl+u
Sleep 400ms
Type "secret"
Sleep 2.5s

Ctrl+u
Sleep 400ms
Type "deploy"
Sleep 2.5s

Ctrl+u
Sleep 2s

Escape
Sleep 1s

# ══════════════════════════════════════════════════════════════════════════════
#  SECTION 3 — Preview modes
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear && printf '\\n  \\033[1;33m[ 3 / 6 ]  Preview modes\\033[0m\\n  \\033[90mCtrl-P cycles: describe → YAML → logs. No second terminal needed.\\033[0m\\n'"
Enter
Sleep 3s
Type "clear"
Enter
Show

Type "kf pods --context kind-kubefuzz-dev"
Enter
Sleep 5s

Type "pod-running"
Sleep 2s

# describe (default)
Sleep 3s

# YAML
Ctrl+p
Sleep 3s

# logs
Ctrl+p
Sleep 3s

# back to describe
Ctrl+p
Sleep 1.5s

# switch to crashloop — back-off events visible in describe
Ctrl+u
Sleep 400ms
Type "crashloop"
Sleep 2s

Sleep 3s

# YAML shows the broken container spec
Ctrl+p
Sleep 3s

Escape
Sleep 1s

# ══════════════════════════════════════════════════════════════════════════════
#  SECTION 4 — Actions
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear && printf '\\n  \\033[1;33m[ 4 / 6 ]  Actions\\033[0m\\n  \\033[90mCtrl-L logs  Ctrl-Y yaml  Ctrl-R rollout restart  Ctrl-D delete\\033[0m\\n'"
Enter
Sleep 3s
Type "clear"
Enter
Show

# ── Ctrl-L: stream logs → kf relaunches ──────────────────────────────────────
Type "kf pods --context kind-kubefuzz-dev"
Enter
Sleep 5s

Type "pod-running"
Sleep 1.5s

Ctrl+l
Sleep 6s

# kf relaunches — now in skim again
Type "pod-running"
Sleep 1.5s

# ── Ctrl-Y: dump YAML → kf exits (back at shell) ─────────────────────────────
Ctrl+y
Sleep 5s

# ── Ctrl-R: rollout restart with live tracking ────────────────────────────────
Type "kf deploy --context kind-kubefuzz-dev"
Enter
Sleep 5s

Type "deploy-healthy"
Sleep 2s

Ctrl+r
Sleep 35s

Escape
Sleep 1s

# ── Ctrl-D: delete with safety guard ─────────────────────────────────────────
Type "kf pods --context kind-kubefuzz-dev"
Enter
Sleep 5s

Type "crashloop"
Sleep 2s

Tab
Sleep 800ms

Ctrl+d
Sleep 1.5s

Type "n"
Enter
Sleep 1.5s

# Multi-select two items, show "Delete 2 resources?" prompt
Ctrl+u
Sleep 400ms
Tab
Sleep 500ms
Down
Sleep 400ms
Tab
Sleep 500ms

Ctrl+d
Sleep 1.5s

Type "n"
Enter
Sleep 1.5s

Escape
Sleep 1s

# ══════════════════════════════════════════════════════════════════════════════
#  SECTION 5 — Multi-cluster
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear && printf '\\n  \\033[1;33m[ 5 / 6 ]  Multi-cluster\\033[0m\\n  \\033[90m--all-contexts streams every cluster simultaneously. Ctrl-X switches live.\\033[0m\\n'"
Enter
Sleep 3s
Type "clear"
Enter
Show

Type "kf --all-contexts"
Enter
Sleep 8s

Type "kubefuzz"
Sleep 2.5s

Ctrl+u
Sleep 2s

Escape
Sleep 1s

Type "kf --context kind-kubefuzz-dev"
Enter
Sleep 5s

Ctrl+x
Sleep 2s

Down
Sleep 600ms
Enter
Sleep 6s

Escape
Sleep 1s

# ══════════════════════════════════════════════════════════════════════════════
#  SECTION 6 — Demo mode
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear && printf '\\n  \\033[1;33m[ 6 / 6 ]  Demo mode\\033[0m\\n  \\033[90mNo cluster? kf shows live sample data automatically.\\033[0m\\n'"
Enter
Sleep 3s
Type "clear"
Enter
Show

Type "KUBECONFIG=/nonexistent kf"
Enter
Sleep 5s

Type "crash"
Sleep 2.5s

Ctrl+u
Sleep 1s

Ctrl+p
Sleep 2.5s

Escape
Sleep 1s

# ══════════════════════════════════════════════════════════════════════════════
#  OUTRO
# ══════════════════════════════════════════════════════════════════════════════

Hide
Type "clear"
Enter
Type "printf '\\n  \\033[1;32mInstall\\033[0m\\n\\n    \\033[96mcargo install kuberift\\033[0m\\n\\n    \\033[96mbrew tap syedazeez337/kuberift && brew install kf\\033[0m\\n\\n  \\033[90mgithub.com/syedazeez337/kuberift\\033[0m\\n'"
Enter
Show
Sleep 7s
