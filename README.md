# kf — KubeFuzz

> A fuzzy-first interactive Kubernetes resource navigator

`kf` lets you fuzzy-search every resource across every namespace in your cluster from a single terminal window. Select one or many, then describe, exec, tail logs, delete, port-forward, restart, or dump YAML — all without typing a single `kubectl` command.

![demo](https://raw.githubusercontent.com/syedazeez337/kubefuzz/master/docs/demo.gif)

---

## Features

- **Fuzzy search everything** — pods, deployments, services, secrets, configmaps, nodes, namespaces, PVCs, jobs, cronjobs, statefulsets, daemonsets, ingresses — all at once
- **Live preview pane** — inline `describe`, YAML manifest, or pod logs, cycled with `ctrl-p`
- **Live watch** — resources appear and update in real time as the cluster changes; deleted resources show `[DELETED]`
- **Unhealthy-first ordering** — `CrashLoopBackOff`, `Error`, `ImagePullBackOff` pods surface to the top automatically
- **Color-coded status** — red for critical, yellow for warning, green for healthy, dimmed for deleted
- **Multi-select bulk actions** — `tab` to select multiple resources, then describe/delete/restart them all at once
- **Multi-cluster support** — watch all kubeconfig contexts simultaneously with `--all-contexts`, or switch contexts interactively with `ctrl-x`
- **Context persistence** — last-used context is remembered across sessions
- **Demo mode** — works without a cluster; shows sample data so you can explore the UI

---

## Requirements

- Rust toolchain (`cargo`) — to build from source
- `kubectl` in `$PATH` — used for all actions (describe, logs, exec, delete, etc.)
- A valid kubeconfig (`~/.kube/config` or `$KUBECONFIG`) — optional; demo mode activates automatically if absent

---

## Installation

```bash
git clone https://github.com/syedazeez337/kubefuzz.git
cd kubefuzz
cargo build --release
# Binary is at target/release/kf
# Optionally move it somewhere on your PATH:
sudo mv target/release/kf /usr/local/bin/kf
```

---

## Usage

```
kf [RESOURCE] [OPTIONS]
```

### Show all resources (default)

```bash
kf
```

Opens the TUI with every resource type streaming from the current kubeconfig context.

### Filter to a specific resource type

```bash
kf pods        # or: pod, po
kf deploy      # or: deployment, deployments
kf svc         # or: service, services
kf sts         # or: statefulset, statefulsets
kf ds          # or: daemonset, daemonsets
kf cm          # or: configmap, configmaps
kf secret      # or: secrets
kf ing         # or: ingress, ingresses
kf node        # or: nodes, no
kf ns          # or: namespace, namespaces
kf pvc         # or: persistentvolumeclaim
kf job         # or: jobs
kf cj          # or: cronjob, cronjobs
```

### Use a specific context

```bash
kf --context my-prod-cluster
```

Overrides both the kubeconfig current context and the last-saved context.

### Watch all contexts simultaneously

```bash
kf --all-contexts
```

Streams resources from every context in your kubeconfig in parallel. Each item is prefixed with its cluster name (color-coded per cluster).

---

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| Type anything | Fuzzy filter the list in real time |
| `↑` / `↓` | Move cursor |
| `tab` | Toggle selection on current item (multi-select) |
| `esc` | Quit |

### Actions (on selected item(s))

| Key | Action | Multi-select |
|-----|--------|:---:|
| `enter` | `kubectl describe` | ✓ |
| `ctrl-l` | Stream pod logs (`--tail=200`) | ✓ |
| `ctrl-e` | `kubectl exec -it` into shell | — |
| `ctrl-d` | Delete with `y/N` confirmation | ✓ |
| `ctrl-f` | Port-forward (prompts for local/remote port) | — |
| `ctrl-r` | `kubectl rollout restart` (deploy/sts/ds) | ✓ |
| `ctrl-y` | Print YAML to stdout | ✓ |

### Preview & context

| Key | Action |
|-----|--------|
| `ctrl-p` | Cycle preview mode: **describe → yaml → logs** |
| `ctrl-x` | Open context picker — switch cluster without restarting |

---

## Preview Modes

The right-hand preview pane updates as you move the cursor. Press `ctrl-p` to cycle through three modes:

| Mode | Content |
|------|---------|
| `describe` | `kubectl describe <resource>` output |
| `yaml` | `kubectl get <resource> -o yaml` |
| `logs` | Last 100 lines of pod logs (pods only) |

---

## Multi-cluster Mode

```bash
kf --all-contexts
```

All contexts from your kubeconfig are loaded in parallel. Items are prefixed with the cluster name:

```
pod   prod-cluster/default/api-server-7d9f     Running   2d
pod   staging/default/api-server-5c2a          Pending   5m
```

Each cluster gets a distinct color so items are immediately identifiable.

### Switching contexts interactively

Press `ctrl-x` while `kf` is running to open a secondary fuzzy picker showing all your kubeconfig contexts. Selecting a context restarts the resource stream from that cluster. The selected context is saved to `~/.config/kubefuzz/last_context` and restored on the next launch.

---

## Status Colors

| Color | Meaning | Example statuses |
|-------|---------|-----------------|
| Red | Critical — needs attention | `CrashLoopBackOff`, `Error`, `ImagePullBackOff`, `OOMKilled`, `Failed`, `Evicted` |
| Yellow | Warning — transitional | `Pending`, `Terminating`, `Init:0/1`, `ContainerCreating` |
| Green | Healthy | `Running`, `Succeeded`, `Active`, `Bound`, `ClusterIP` |
| Gray | Gone | `[DELETED]`, `Unknown` |

Unhealthy resources (red) automatically sort to the top of the list so critical issues are visible immediately without scrolling.

---

## Demo Mode

If `kubectl` cannot connect to a cluster (no kubeconfig, invalid context, or network error), `kf` falls back to demo mode and displays 11 sample resources so you can explore the interface:

```bash
KUBECONFIG=/nonexistent kf
# [kubefuzz] No cluster (...). Showing demo data.
```

---

## Config & State

| File | Purpose |
|------|---------|
| `~/.config/kubefuzz/last_context` | Last-used context, restored on next launch |
| `/tmp/kubefuzz-preview-mode` | Preview mode state (0=describe, 1=yaml, 2=logs) |
| `/tmp/kubefuzz-preview-toggle` | Shell script installed at startup for ctrl-p |

---

## License

MIT — same as [skim](https://github.com/skim-rs/skim), the underlying fuzzy engine.
