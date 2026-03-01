# KubeRift — Technical Architecture

> **Note:** This document is derived from the actual source code. Every claim has been verified against the implementation.

---

## High-Level Design

```
┌─────────────────────────────────────────────────────────────────┐
│                         kuberift (kf)                           │
│                        CLI entry point                          │
│                   src/main.rs + src/cli.rs                      │
└──────────────────────────┬──────────────────────────────────────┘
                           │
           ┌───────────────┼───────────────┐
           │               │               │
    ┌──────▼──────┐ ┌──────▼──────┐ ┌─────▼──────┐
    │  K8s Client │ │  Resource   │ │  Context   │
    │  (kube-rs)  │ │  Watcher    │ │  Manager   │
    │  client.rs  │ │ resources.rs│ │ client.rs  │
    └──────┬──────┘ └──────┬──────┘ └─────┬──────┘
           │               │              │
           └───────────────▼──────────────┘
                           │
                  ┌────────▼────────┐
                  │  Item Builder   │
                  │ (skim items)    │
                  │ src/items.rs    │
                  └────────┬────────┘
                           │
                  ┌────────▼────────┐
                  │   skim engine   │  ← fuzzy match, multi-select,
                  │  (library API)  │    preview, keybindings
                  └────────┬────────┘
                           │
                  ┌────────▼────────┐
                  │ Action Handler  │
                  │ src/actions.rs  │  ← delete, exec, logs, port-fwd
                  └─────────────────┘
```

---

## Module Breakdown

### `src/main.rs`

Entry point. Parses CLI args via `clap`, determines single-context vs multi-context mode, builds the skim channel pair `(SkimItemSender, SkimItemReceiver)`, spawns the K8s watcher task, and runs the skim TUI loop.

Two top-level execution paths:

- **`run_single_context`** — default mode; supports `ctrl-x` context switching in a loop
- **`run_all_contexts`** — `--all-contexts` mode; spawns one watcher per context and merges all into a single skim channel

After skim returns, `dispatch()` routes the final key to the appropriate action function.

### `src/cli.rs`

Clap-based argument definitions:

```
kf [RESOURCE] [OPTIONS]

RESOURCE:   Optional filter (pods, svc, deploy, sts, ds, cm, secret,
            ing, node, ns, pvc, job, cronjob). Omit to show ALL types.

OPTIONS:
  --context <CONTEXT>     Use specific kubeconfig context
  --all-contexts          Watch all kubeconfig contexts simultaneously
  -n, --namespace <NS>    Restrict to a specific namespace
  --read-only             Disable all write/exec actions
  --kubeconfig <PATH>     Path to kubeconfig file
```

`Args::resource_filter()` converts the resource string to `Vec<ResourceKind>`, with case-insensitive alias matching (e.g. `"PODS"` → `[Pod]`). Returns `None` for unknown types (falls back to all).

### `src/k8s/`

#### `src/k8s/client.rs`

- **`build_client_for_context(context_name, kubeconfig)`** — builds a `kube::Client` for a named context. Accepts an optional path to an alternate kubeconfig file; otherwise uses `$KUBECONFIG` or `~/.kube/config`.
- **`current_context()`** — reads the active context name from kubeconfig for display.
- **`list_contexts()`** — returns all context names from kubeconfig, sorted alphabetically.
- **`save_last_context(ctx)`** — persists the last-used context to `~/.config/kuberift/last_context`. Sets `0o700` on the directory and `0o600` on the file (Unix only).
- **`load_last_context()`** — restores the saved context on startup.

#### `src/k8s/resources.rs`

Defines the resource types and live-streaming logic.

```rust
pub const ALL_KINDS: &[ResourceKind] = &[
    Pod, Deployment, StatefulSet, DaemonSet, Service, Ingress,
    Job, CronJob, ConfigMap, Secret, PersistentVolumeClaim, Namespace, Node,
];
```

**`watch_resources(client, tx, kinds, context, namespace)`** — spawns one tokio task per `ResourceKind`, each running `watch_typed`.

**`watch_typed<T, F>(client, tx, kind, status_fn, context, namespace)`** — generic watcher using `kube::runtime::watcher` with automatic reconnect via `default_backoff()`. Lifecycle:

- `Init` — new watch cycle; clears the init buffer
- `InitApply` — buffers existing objects for sorting
- `InitDone` — sorts buffer by `status_priority` (unhealthy first) and sends the entire batch
- `Apply` — sends live add/update immediately
- `Delete` — sends item with status `[DELETED]`

Namespace filtering uses a field selector (`metadata.namespace=<ns>`) applied to `Api::all`, avoiding the `NamespaceResourceScope` type constraint. Cluster-scoped resources (Node, Namespace) always use `None` for the namespace parameter.

Per-resource status extractors: `pod_status`, `deploy_status`, `statefulset_status`, `daemonset_status`, `service_status`, `secret_status`, `ingress_status`, `node_status`, `namespace_status`, `pvc_status`, `job_status`, `cronjob_status`.

**`status_priority(status)`** — delegates to `StatusHealth::classify(status).priority()`.

**`resource_age(meta)`** — computes human-readable age from `creation_timestamp` using `k8s_openapi::jiff`.

### `src/items.rs`

Implements `skim::SkimItem` for Kubernetes resources.

```rust
pub struct K8sItem {
    kind: ResourceKind,       // private — use accessors
    namespace: String,
    name: String,
    status: String,
    age: String,
    context: String,          // cluster label (empty in single-cluster mode)
}
```

All fields are private. Accessors: `kind()`, `namespace()`, `name()`, `status()`, `age()`, `context()`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ResourceKind {
    Pod, Service, Deployment, StatefulSet, DaemonSet,
    ConfigMap, Secret, Ingress, Node, Namespace,
    PersistentVolumeClaim, Job, CronJob,
}
```

`ResourceKind` implements `Display` (delegates to `as_str()`), `Copy`, and `Hash`.

**`StatusHealth`** — single source of truth for both color and sort priority:

```rust
pub enum StatusHealth { Critical, Warning, Healthy, Unknown }
```

- `classify(status)` — maps status strings to health categories (exact matches, prefix matches, ratio parsing)
- `color()` → `Red` / `Yellow` / `Green` / `DarkGray`
- `priority()` → `0` (critical) / `1` (warning) / `2` (healthy)

**`SkimItem` implementation:**
- `text()` — the string skim fuzzy-matches against: `"[ctx/]kind  [ns/]name  status  age"`
- `display()` — colored ratatui `Line` with kind, namespace, name, status, age columns
- `preview()` — runs `kubectl describe`, `kubectl get -o yaml`, or `kubectl logs` depending on the mode file at `$XDG_RUNTIME_DIR/<pid>/preview-mode`
- `output()` — parseable `"[ctx:]kind/[ns/]name"` for piping

**`truncate_name(name, max_chars)`** — UTF-8 safe truncation with `…` suffix.

**`context_color(ctx)`** — deterministic per-cluster color derived from a hash of the context name.

### `src/actions.rs`

Handles what happens after skim returns. All functions are synchronous (no `async`).

**Secure temp file handling (SEC-001):**

```rust
pub fn runtime_dir() -> &'static PathBuf
// Uses: $XDG_RUNTIME_DIR/<pid>/  or  /tmp/kuberift-<pid>/
// Directory is created with 0o700 permissions on Unix.
```

- `preview_toggle_path()` → shell script that cycles the preview mode (0/1/2)
- `install_preview_toggle()` → writes the script to `runtime_dir()`

**Action functions:**
- `action_describe(&items)` — `kubectl describe <kind> -n <ns> -- <name>` for each item
- `action_logs(&items)` — `kubectl logs --tail=200 -n <ns> -- <name>`
- `action_exec(&item)` — `kubectl exec -it <name> -n <ns> -- /bin/sh` (tries `/bin/sh` then `/bin/bash`)
- `action_delete(&items)` — prompts `[y/N]`; for >10 items requires typing `"yes"` (bulk guard)
- `action_portforward(&item)` — prompts for local and remote port (validates `u16`, rejects port 0, warns on privileged ports)
- `action_rollout_restart(&items)` — `kubectl rollout restart <kind>/<name>` then waits on `kubectl rollout status`
- `action_yaml(&items)` — `kubectl get <kind> -o yaml -n <ns> -- <name>`

All kubectl calls use `"--"` before resource names to prevent argument injection (SEC-002).

---

## Skim Integration Pattern

Skim is used as a **library** (not spawned as a subprocess). The core integration:

```rust
let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

// K8s watcher sends items to skim as they arrive
tokio::spawn(async move {
    watch_resources(client, tx, &kinds, &ctx, namespace).await
});

// Build skim options — header, keybindings, preview window
let options = build_skim_options(&ctx_label, &kind_label, show_ctx_switch, read_only, namespace)?;

// Run skim — blocks until user presses a key
let output = Skim::run_with(options, Some(rx))?;

// Dispatch based on final_key
dispatch(output, read_only)?;
// Loop continues — skim reopens after each action (BUG-002 fix)
```

Keybindings use `accept` (not `execute`) so the selected items and key are returned to Rust code for dispatch. The `ctrl-p` binding uses `execute(<preview-toggle-script>)+refresh-preview` to cycle the preview mode file.

---

## Key Dependencies (`Cargo.toml`)

```toml
[dependencies]
# Fuzzy finder engine (git fork — fixes simd module visibility)
skim = { git = "https://github.com/syedazeez337/skim", branch = "master", default-features = false }

# TUI rendering (must match skim's ratatui version)
ratatui = "0.30"

# Kubernetes client
kube = { version = "3.0.1", features = ["client", "config", "runtime"] }
k8s-openapi = { version = "0.27.0", features = ["latest"] }

# Async runtime (minimal feature set)
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }

# Async utilities
futures = "0.3"

# Terminal input events
crossterm = "0.29"

# Error handling
anyhow = "1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# CLI args
clap = { version = "4", features = ["derive"] }

# Config file paths
toml = "1"
dirs = "6"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

---

## Directory Structure

```
kuberift/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .github/
│   └── workflows/
│       └── ci.yml              ← fmt, clippy, test, audit, release build
├── docs/
│   ├── guides/                 ← all documentation markdown
│   │   ├── ARCHITECTURE.md     ← This file
│   │   ├── CONTRIBUTING.md
│   │   ├── REMEDIATION_PART1.md
│   │   ├── REMEDIATION_PART2.md
│   │   ├── RESEARCH.md
│   │   ├── ROADMAP.md
│   │   ├── SKIM_NOTES.md
│   │   └── TESTING.md
│   ├── manual/                 ← user manual (LaTeX source + compiled PDF)
│   │   ├── kuberift-manual.tex
│   │   └── kuberift-manual.pdf
│   └── media/                  ← demo GIFs and MP4 recordings
│       ├── demo.gif / demo.mp4
│       ├── actions.gif / actions.mp4
│       ├── delete.gif / delete.mp4
│       ├── filter.gif / filter.mp4
│       ├── multicluster.gif / multicluster.mp4
│       ├── preview.gif / preview.mp4
│       └── tour.gif
├── src/
│   ├── main.rs                 ← Entry point, skim run loop, action dispatch
│   ├── cli.rs                  ← Clap CLI definition + resource_filter()
│   ├── items.rs                ← K8sItem, ResourceKind, StatusHealth, SkimItem impl
│   ├── actions.rs              ← Post-selection action handlers + secure temp paths
│   └── k8s/
│       ├── mod.rs
│       ├── client.rs           ← kube::Client builder, context management
│       └── resources.rs        ← watch_resources, watch_typed, status extractors
└── tests/
    └── cli_integration.rs      ← Integration tests (--help, --version)
```

---

## Design Decisions & Rationale

### Why Rust?
- skim is a Rust library — native integration, no subprocess overhead
- kube-rs is the most mature async K8s client in Rust
- Single static binary — easy distribution, no runtime deps
- Fast startup critical for interactive TUI

### Why skim over fzf?
- skim is a Rust library — embedded directly, no subprocess spawn
- fzf is a Go binary — would require spawning a process and piping, losing type safety
- `SkimItem` trait gives full control over display, preview, and output format
- MIT license

### Why kube-rs over shelling out to kubectl?
- kube-rs uses the same kubeconfig, works identically
- Async streaming via `kube::runtime::watcher` feeds skim's item channel natively
- Type-safe resource handling with generated k8s-openapi types

### Why skim fork?
The published `skim 3.4.0` on crates.io has a broken build with `frizbee 0.8.2` (simd module became private). A patched fork at `github.com/syedazeez337/skim` fixes this. The `default-features = false` flag skips building the skim CLI binary.

### Status-based Sorting
`StatusHealth::classify()` is the single source of truth for both display color and sort priority. `InitDone` batches are sorted by `status_priority` before being sent to skim, so `CrashLoopBackOff` pods surface above `Running` pods in the initial render.

### Multi-select Action Model
- All selected items receive the same action
- Dangerous actions (delete) show count + confirmation; >10 items require typing `"yes"`
- Actions that don't make sense in bulk (exec, port-forward) use only the first selected item

### Secure Temp Files
Preview state is stored in `$XDG_RUNTIME_DIR/<pid>/` (falls back to `/tmp/kuberift-<pid>/`). The per-PID subdirectory prevents symlink attacks (CWE-59 / CWE-367). Directories are created with `0o700` and files with `0o600` (Unix). The runtime dir is cleaned up on exit.

### Read-Only Mode
`--read-only` disables `exec`, `delete`, `port-forward`, and `rollout-restart`. `describe`, `logs`, and `yaml` remain available. The header shows `[READ-ONLY]` to communicate the constraint.
