# KubeFuzz — Roadmap

---

## Phase 0 — Skeleton ✅ COMPLETE

- [x] `cargo new kubefuzz --bin`
- [x] Add skim, kube-rs, tokio to Cargo.toml
- [x] Confirm skim compiles
- [x] Create `K8sItem` struct implementing `SkimItem` with stub data
- [x] Run skim with hardcoded fake pod items — TUI works
- [x] Commit: "feat: skeleton with skim integration"

---

## Phase 1 — MVP: Single Cluster Pod Browser ✅ COMPLETE

- [x] Load kubeconfig from `~/.kube/config` using kube-rs
- [x] List all pods across all namespaces
- [x] Format each pod as a `K8sItem` with kind/namespace/name/status/age
- [x] Stream pods into skim via `SkimItemSender`
- [x] Preview pane: `kubectl describe pod` output
- [x] On `<enter>`: describe selected resource
- [x] `ctrl-l` → stream `kubectl logs` to terminal
- [x] Handle errors gracefully (no kubeconfig → demo mode)

---

## Phase 2 — All Resource Types ✅ COMPLETE

- [x] Service, Deployment, StatefulSet, DaemonSet, ConfigMap, Secret, Ingress, Node, Namespace, PVC, Job, CronJob
- [x] `kf` with no args → stream ALL resource types simultaneously
- [x] Color-code by resource type (pods=green, svc=blue, deploy=yellow…)
- [x] Color-code by status (Running=green, Pending=yellow, CrashLoop/Error=red)
- [x] Unhealthy resources sort to top (`StatusHealth` priority system)
- [x] CLI: `kf pods`, `kf svc`, `kf deploy` — filter to one resource type
- [x] Preview pane: cycle describe/yaml/logs with `ctrl-p`

---

## Phase 3 — Actions ✅ COMPLETE

- [x] `ctrl-d`: delete with `[y/N]` prompt; >10 items requires typing `"yes"`
- [x] `ctrl-e`: `kubectl exec -it <pod> -- /bin/sh`
- [x] `ctrl-f`: `kubectl port-forward <pod/svc> <local>:<remote>` (port prompts, privileged warning, guard for non-pod/svc)
- [x] `ctrl-r`: rollout restart + tracks `kubectl rollout status`
- [x] `ctrl-y`: print full YAML to stdout
- [x] Multi-select (`<tab>`) + bulk actions
- [x] `--read-only` flag disables write/exec actions

---

## Phase 4 — Live Watching ✅ COMPLETE

- [x] `kube::runtime::watcher` with `default_backoff()` auto-reconnect
- [x] Items added/updated/deleted reflect in real time
- [x] `InitDone` batch sorted unhealthy-first before first render
- [x] `[DELETED]` status shown for deleted resources

---

## Phase 5 — Multi-Cluster ✅ COMPLETE

- [x] `ctrl-x`: fuzzy-search kubeconfig contexts → switch active context
- [x] `--all-contexts`: stream resources from all contexts simultaneously, color-coded by cluster
- [x] Context persistence: `~/.config/kubefuzz/last_context`, restored on next launch
- [x] All kubectl actions pass `--context` in multi-cluster mode
- [x] `build_client_for_context`, `list_contexts`, `save/load_last_context` helpers

---

## Phase 6 — Distribution & Polish ✅ COMPLETE

- [x] GitHub Actions CI: fmt + clippy + test + audit + release build on every push
- [x] Shell completions (bash, zsh, fish) via `clap_complete` — `kf --completions <shell>`
- [x] Man page generation via `clap_mangen` — `kf --mangen`
- [x] GitHub Releases with pre-built binaries (x86_64 + arm64, linux + macos) — `.github/workflows/release.yml`
- [x] Homebrew formula — `contrib/kf.rb` (update sha256 after tagging)
- [x] AUR package for Arch Linux — `contrib/PKGBUILD`
- [ ] README demo GIF (vhs or asciinema) — tape script at `contrib/kf.tape`; run after cluster setup

---

## Future / Pro Features

- [ ] Resource usage metrics (CPU/memory) in list view via metrics-server
- [ ] Saved views / named filters (save a search for "all failing pods in prod")
- [ ] RBAC-aware mode (hide resources current user can't access)
- [ ] Helm release browser
- [ ] kubectl plugin compatibility (`kubectl kf`)
- [ ] Plugin system for custom resource types (CRDs)
- [ ] Audit log (which user ran what action on which resource)

---

## Development Setup

```bash
# Clone the repo
git clone https://github.com/syedazeez337/kubefuzz.git
cd kubefuzz

# Install Rust (if not already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# skim requires these system deps on Linux
sudo apt install -y libncurses5-dev libncursesw5-dev    # Ubuntu/Debian
# sudo dnf install ncurses-devel                         # Fedora/RHEL
# sudo pacman -S ncurses                                 # Arch

# Build
cargo build --release

# Run (needs a working kubeconfig at ~/.kube/config)
./target/release/kf

# Run tests
cargo test
```
