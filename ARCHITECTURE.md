# Architecture

KubeFuzz (`kf`) is a single-binary TUI built in Rust. This document describes the actual source layout and data flow.

## Source layout

```
src/
├── main.rs          # Entry point, TUI event loop, skim integration
├── cli.rs           # CLI argument parsing (clap), ResourceKind enum
├── items.rs         # K8sItem, StatusHealth, sort/display logic
├── actions.rs       # kubectl subprocess actions (logs, exec, delete, …)
└── k8s/
    ├── mod.rs       # Re-exports
    ├── client.rs    # kubeconfig loading, context persistence
    └── resources.rs # Per-resource-type watchers and status extraction
```

## Data flow

```
CLI args (cli.rs)
      │
      ▼
k8s/client.rs  ── loads kubeconfig, resolves context ──▶  kube::Client
      │
      ▼
k8s/resources.rs  ── watches API objects via kube-rs ──▶  Vec<K8sItem>
      │                (one watcher per resource kind,
      │                 merged into a single sorted list)
      ▼
items.rs  ── StatusHealth::classify() ──▶  unhealthy-first sort
      │
      ▼
main.rs  ── feeds items into skim ──▶  interactive fuzzy TUI
      │
      ▼  (user presses a key binding)
actions.rs  ── spawns kubectl subprocess ──▶  logs / exec / delete / …
      │
      └──▶  loop restarts (returns to skim)
```

## Key types

### `ResourceKind` (`src/cli.rs`)
`#[non_exhaustive]` enum of every supported Kubernetes resource type (Pod, Deployment, Service, …). Implements `Display`, `Copy`, and `Hash`. Parsed from CLI positional args and short aliases (`po`, `deploy`, `svc`, …).

### `K8sItem` (`src/items.rs`)
Represents one Kubernetes object in the list. All fields are **private**; consumers use getter methods. Carries: name, namespace, kind, status string, age, raw JSON for YAML view, and context name for multi-cluster mode.

### `StatusHealth` (`src/items.rs`)
Single source of truth for health classification:

| Variant    | Examples |
|------------|----------|
| `Critical` | CrashLoopBackOff, Error, OOMKilled, Failed |
| `Warning`  | Pending, Terminating, ContainerCreating |
| `Healthy`  | Running, Completed, Active, Bound |
| `Unknown`  | anything else |

Both the sort order (unhealthy-first) and the colour in the TUI derive from `StatusHealth` — they cannot desync.

### `SkimOptions` (`src/main.rs`)
Built in `build_skim_options()` which returns `anyhow::Result<SkimOptions>`. Configures the preview command (a shell script written to a secure temp file), keybindings, and the initial query from `--filter`.

## Security properties

| Property | Implementation |
|----------|---------------|
| No `/tmp` races | Runtime dir via `dirs::runtime_dir()` → `$XDG_RUNTIME_DIR` → `std::env::temp_dir()` |
| Directory permissions | `0o700` on all runtime dirs created by kf |
| Context file permissions | `0o600` on `~/.config/kubefuzz/last-context` |
| Argument injection | `--` separator before all resource names in kubectl calls |
| Port validation | 1–65535 enforced; warning for privileged ports < 1024 |
| Bulk delete guard | >10 resources require typing `yes` at a prompt |
| Read-only mode | `--read-only` blocks all mutating actions at runtime |

## Concurrency model

KubeFuzz uses a **single tokio multi-thread runtime**. Each resource kind has an independent watcher task that streams updates into a shared `Arc<Mutex<Vec<K8sItem>>>`. The TUI loop runs synchronously on the main thread and reads a snapshot of the shared vec each iteration.

Tokio features used: `rt-multi-thread`, `macros`, `time`.

## Shell completions and man page

Generated at runtime via `--completions <shell>` and `--mangen`, using `clap_complete` and `clap_mangen`. No build-time codegen — the binary itself is the generator.
