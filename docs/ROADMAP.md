# KubeFuzz — Roadmap

---

## Phase 0 — Skeleton (Start Here on Linux)

**Goal**: Compiling Rust project with skim integrated, prints hello.

- [ ] `cargo new kubefuzz --bin`
- [ ] Add skim, kube-rs, tokio to Cargo.toml
- [ ] Confirm skim compiles (it has C deps — ncurses/termion — check on Linux)
- [ ] Create `K8sItem` struct implementing `SkimItem` with stub data
- [ ] Run skim with 5 hardcoded fake pod items — verify TUI works
- [ ] Commit: "feat: skeleton with skim integration"

**Exit criteria**: `cargo run` opens a skim TUI showing fake items.

---

## Phase 1 — MVP: Single Cluster Pod Browser

**Goal**: Browse real pods from current kubectl context.

- [ ] Load kubeconfig from `~/.kube/config` using kube-rs
- [ ] List all pods across all namespaces (`Api::<Pod>::all(client).list()`)
- [ ] Format each pod as a `K8sItem` with kind/namespace/name/status/age
- [ ] Stream pods into skim via `SkimItemSender`
- [ ] Preview pane: run `kubectl describe pod <name> -n <ns>` and show output
- [ ] On `<enter>`: print selected resource to stdout (composable with pipes)
- [ ] Basic keybinding: `ctrl-l` → stream `kubectl logs <pod>` in preview
- [ ] Config: read `~/.config/kubefuzz/config.toml` with sane defaults
- [ ] Handle errors gracefully (no kubeconfig, API unreachable)

**Exit criteria**: `kf pods` shows all real pods, preview works, logs work.

---

## Phase 2 — All Resource Types

**Goal**: Search across every common K8s resource type simultaneously.

- [ ] Add resource types: Service, Deployment, StatefulSet, DaemonSet, ConfigMap, Secret, Ingress, Node, Namespace, PVC, Job, CronJob
- [ ] `kf` with no args → stream ALL resource types into skim simultaneously
- [ ] Color-code by resource type in display (pods=green, svc=blue, deploy=yellow…)
- [ ] Color-code by status (Running=green, Pending=yellow, Failed=red, CrashLoop=magenta)
- [ ] Failed/Pending resources sort to top (implement ranking in `SkimItem`)
- [ ] CLI: `kf pods`, `kf svc`, `kf deploy` — filter to one resource type
- [ ] Preview pane: switch between describe/yaml/logs with `ctrl-p`

**Exit criteria**: `kf` shows all resource types; user can search "nginx" and get pods + services + deployments matching.

---

## Phase 3 — Actions

**Goal**: Take actions on selected resources from within the TUI.

- [ ] `ctrl-d`: Delete selected resource(s) — with count confirmation prompt
- [ ] `ctrl-e`: `kubectl exec -it <pod> -- /bin/sh` — exec into container
- [ ] `ctrl-f`: `kubectl port-forward <pod/svc> <local>:<remote>`
- [ ] `ctrl-r`: Rollout restart (`kubectl rollout restart deploy/<name>`)
- [ ] `ctrl-y`: Print full YAML to stdout and exit
- [ ] `ctrl-c` (or custom): Copy resource name to clipboard
- [ ] Multi-select (`<tab>`) + `ctrl-d`: delete all selected
- [ ] Confirm destructive actions (delete) with `y/N` prompt

**Exit criteria**: Full day-to-day K8s workflow completable without leaving kubefuzz.

---

## Phase 4 — Live Watching

**Goal**: Resources update in real-time without restarting.

- [ ] Use `kube::runtime::watcher` instead of one-shot list
- [ ] Items added/updated/deleted reflect instantly in skim list
- [ ] Status changes (pod goes from Pending → Running) update in-place
- [ ] Show "last updated" timestamp and event count in status bar
- [ ] Refresh rate control in config (`watch_interval_ms = 2000`)

**Exit criteria**: Open kubefuzz while deploying — watch pods come up live.

---

## Phase 5 — Multi-Cluster

**Goal**: Switch between clusters without leaving the TUI.

- [ ] `ctrl-x`: fuzzy-search kubeconfig contexts → switch active context
- [ ] `--all-contexts` flag: load resources from ALL contexts simultaneously, prefix with cluster name
- [ ] Color-code resources by cluster in multi-context mode
- [ ] Config: short aliases for long context names (`prod = "arn:aws:eks:..."`)
- [ ] Persist last-used context across sessions

**Exit criteria**: `kf --all-contexts` shows all pods from all clusters; user can switch seamlessly.

---

## Phase 6 — Distribution & Polish

**Goal**: Ready for public launch.

- [ ] Shell completions (bash, zsh, fish) via clap
- [ ] Man page generation
- [ ] `--version` with build info
- [ ] GitHub Actions CI: build + test on ubuntu-latest, macos-latest
- [ ] GitHub Releases with pre-built binaries (x86_64 + arm64, linux + macos)
- [ ] Homebrew tap (`homebrew-kubefuzz`)
- [ ] AUR package for Arch Linux
- [ ] README GIF/demo (use vhs or asciinema)
- [ ] Write "Show HN" post draft

**Exit criteria**: `brew install kubefuzz` works; HN post drafted.

---

## Future / Pro Features

- [ ] Resource usage metrics (CPU/memory) in list view via metrics-server
- [ ] Saved views / named filters (save a search for "all failing pods in prod")
- [ ] RBAC-aware mode (hide resources current user can't access)
- [ ] Helm release browser (fuzzy search Helm releases, see chart version, values)
- [ ] kubectl plugin compatibility (`kubectl kf`)
- [ ] Plugin system for custom resource types (CRDs)
- [ ] Team sharing of config/keybindings
- [ ] Audit log (which user ran what action on which resource)

---

## Development Notes

### Setting Up on Linux

```bash
# Clone the repo
git clone https://github.com/syedazeez337/kubefuzz.git
cd kubefuzz

# Install Rust (if not already)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# skim requires these system deps on Linux
sudo apt install -y libncurses5-dev libncursesw5-dev
# or on Fedora/RHEL:
# sudo dnf install ncurses-devel
# or on Arch:
# sudo pacman -S ncurses

# Build
cargo build

# Run (needs a working kubeconfig at ~/.kube/config)
cargo run

# Run with fake data (Phase 0 mode, no K8s needed)
cargo run -- --demo
```

### Useful Commands During Dev

```bash
# Check skim compiles and links correctly
cargo check

# Run with RUST_LOG for debug output
RUST_LOG=debug cargo run

# Test with a local kind cluster
kind create cluster --name test
cargo run

# Benchmark matching performance
cargo bench
```

### K8s Local Dev Options

| Tool | Command | Notes |
|---|---|---|
| kind | `kind create cluster` | Lightest, Docker-based |
| minikube | `minikube start` | More features, heavier |
| k3s | `curl -sfL https://get.k3s.io \| sh -` | Production-like, Linux only |
| k3d | `k3d cluster create` | k3s in Docker |
