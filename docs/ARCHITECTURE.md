# KubeFuzz — Technical Architecture

---

## High-Level Design

```
┌─────────────────────────────────────────────────────────────────┐
│                         kubefuzz (kf)                           │
│                        CLI entry point                          │
│                   src/main.rs + src/cli.rs                      │
└──────────────────────────┬──────────────────────────────────────┘
                           │
           ┌───────────────┼───────────────┐
           │               │               │
    ┌──────▼──────┐ ┌──────▼──────┐ ┌─────▼──────┐
    │  K8s Client │ │ Resource    │ │   Config   │
    │  (kube-rs)  │ │ Streamer    │ │  Manager   │
    │             │ │             │ │            │
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
Entry point. Parses CLI args, loads config, bootstraps the K8s client and skim runner.

### `src/cli.rs`
Clap-based argument definitions:
```rust
kubefuzz [RESOURCE_TYPE] [FLAGS]

RESOURCE_TYPE:  Optional filter (pods, svc, deploy, cm, secret, node, ...)
                Omit to search ALL resource types simultaneously.

FLAGS:
  --context <ctx>       Use specific kubeconfig context
  --all-contexts        Search across ALL contexts (multi-cluster)
  --namespace <ns>      Restrict to namespace (default: all namespaces)
  --read-only           Disable any write/exec actions
  --preview <cmd>       Override default preview command
```

### `src/k8s/`

#### `src/k8s/client.rs`
Wraps `kube::Client`. Handles:
- Loading kubeconfig (default: `~/.kube/config`)
- Multi-context support — creates one client per context
- Async resource discovery via `kube::discovery::Discovery`

#### `src/k8s/resources.rs`
Defines which resource types to fetch and how to display them:
```rust
pub enum ResourceKind {
    Pod, Service, Deployment, StatefulSet, DaemonSet,
    ConfigMap, Secret, Ingress, Node, Namespace,
    PersistentVolumeClaim, Job, CronJob, ReplicaSet,
}
```
Each variant knows:
- Its API group/version
- How to format its display line in skim
- Its preview strategy (logs vs YAML vs describe)

#### `src/k8s/watcher.rs`
Uses `kube::runtime::watcher` to stream live resource updates.
Sends new/updated/deleted items to skim via `SkimItemSender` channel.
This is the key async integration point with skim.

### `src/items.rs`
Implements `skim::SkimItem` for Kubernetes resources:

```rust
pub struct K8sItem {
    pub kind: ResourceKind,
    pub name: String,
    pub namespace: String,
    pub context: String,           // which cluster
    pub status: ResourceStatus,    // Running, Pending, Failed, etc.
    pub age: Duration,
    pub raw: serde_json::Value,    // full resource JSON for preview
}

impl SkimItem for K8sItem {
    fn text(&self) -> Cow<str> {
        // The string skim fuzzy-matches against:
        // "<kind>  <namespace>/<name>  <status>  <age>"
        // e.g. "pod  production/api-server-7d9f8  Running  2d"
    }

    fn display(&self, context: DisplayContext) -> AnsiString {
        // Colored display: kind in cyan, namespace in yellow,
        // name in white, status colored by health (green/red/yellow)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        // Triggers kubectl describe / logs / get -o yaml
        // Returns text to show in skim preview pane
    }

    fn output(&self) -> Cow<str> {
        // What gets written to stdout on selection:
        // "kind/namespace/name/context" — parseable for piping
    }
}
```

### `src/preview.rs`
Handles the preview pane content. Three preview modes per resource:
1. **Default**: `kubectl describe <kind> <name> -n <namespace>`
2. **Logs** (pods only): `kubectl logs <pod> --tail=50`
3. **YAML**: `kubectl get <kind> <name> -n <namespace> -o yaml`

Preview mode cycles with a keybinding (e.g., `ctrl-p`).

### `src/actions.rs`
Handles what happens after selection:

```rust
pub enum Action {
    Describe,           // default — print describe output
    Logs,               // stream logs (pods)
    Exec,               // kubectl exec -it (pods)
    Delete,             // kubectl delete (with confirmation prompt)
    PortForward,        // kubectl port-forward (pods/services)
    Edit,               // kubectl edit — opens $EDITOR
    CopyName,           // copy resource name to clipboard
    SwitchContext,      // switch kubeconfig context (special resource type)
    PrintYaml,          // output YAML to stdout
}
```

### `src/config.rs`
TOML config file at `~/.config/kubefuzz/config.toml`:

```toml
[defaults]
namespace = "all"          # or specific namespace
preview_mode = "describe"  # describe | logs | yaml
height = "60%"             # skim window height

[keybindings]
logs     = "ctrl-l"
exec     = "ctrl-e"
delete   = "ctrl-d"
forward  = "ctrl-f"
context  = "ctrl-x"
yaml     = "ctrl-y"
copy     = "ctrl-c"

[clusters]
# Aliases for context names (kubeconfig contexts can be long)
prod  = "arn:aws:eks:us-east-1:123456789:cluster/prod-cluster"
stage = "gke_myproject_us-central1_staging"
```

---

## Skim Integration Pattern

Skim is used as a **library** (not spawned as a subprocess). This is critical:
- Items are streamed via `SkimItemSender` channel — no blocking
- Preview is rendered inline by skim's ratatui TUI
- Multi-select returns `Vec<Arc<dyn SkimItem>>` — we downcast to `K8sItem`
- Keybindings are bound via `SkimOptionsBuilder`

```rust
// Pseudocode for the core run loop
async fn run(opts: AppOptions) -> Result<()> {
    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    // Spawn K8s watcher — sends items to skim as they arrive
    let k8s_task = tokio::spawn(async move {
        stream_all_resources(tx, &opts).await
    });

    // Build skim options
    let skim_opts = SkimOptionsBuilder::default()
        .multi(true)
        .preview(Some(""))      // preview handled by SkimItem::preview()
        .bind(vec![
            "ctrl-l:execute(kubefuzz-logs {1})",
            "ctrl-e:execute(kubefuzz-exec {1})",
            "ctrl-d:execute(kubefuzz-delete {1})",
        ])
        .height("60%")
        .build()?;

    // Run skim — blocks until user selects or cancels
    let output = Skim::run_with(&skim_opts, Some(rx))
        .ok_or(anyhow!("skim exited"))?;

    // Handle selected items
    for item in output.selected_items {
        let k8s_item = item.as_any().downcast_ref::<K8sItem>().unwrap();
        handle_action(k8s_item, &output.final_key).await?;
    }

    k8s_task.abort();
    Ok(())
}
```

---

## Key Dependencies (Cargo.toml)

```toml
[dependencies]
# Fuzzy finder engine
skim = "0.15"

# Kubernetes client
kube = { version = "0.98", features = ["runtime", "derive"] }
k8s-openapi = { version = "0.23", features = ["v1_32"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# CLI
clap = { version = "4", features = ["derive"] }

# Error handling
anyhow = "1"
thiserror = "1"

# Config file path
dirs = "5"

# Clipboard (optional, for copy action)
arboard = "3"

# Colors in terminal
owo-colors = "4"

# Crossbeam for channels (skim uses this internally too)
crossbeam-channel = "0.5"
```

---

## Directory Structure

```
kubefuzz/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── docs/
│   ├── RESEARCH.md         ← Market research & competitive analysis
│   ├── ARCHITECTURE.md     ← This file
│   ├── ROADMAP.md          ← Feature phases and MVP definition
│   ├── SKIM_NOTES.md       ← How skim library works
│   └── CONTRIBUTING.md     ← Contribution guidelines
├── src/
│   ├── main.rs             ← Entry point
│   ├── cli.rs              ← Clap CLI definition
│   ├── config.rs           ← Config file loading/defaults
│   ├── items.rs            ← K8sItem: implements SkimItem
│   ├── preview.rs          ← Preview pane content generation
│   ├── actions.rs          ← Post-selection action handlers
│   └── k8s/
│       ├── mod.rs
│       ├── client.rs       ← kube::Client wrapper
│       ├── resources.rs    ← Resource type definitions
│       └── watcher.rs      ← Live resource streaming
└── tests/
    └── integration/
        └── basic.rs
```

---

## Design Decisions & Rationale

### Why Rust?
- skim is a Rust library — native integration, no subprocess overhead
- kube-rs is the most mature async K8s client in Rust
- Single static binary — easy distribution, no runtime deps
- Fast startup (< 100ms) critical for interactive TUI

### Why skim over fzf?
- skim is a Rust library — we embed it directly, no subprocess spawn
- fzf is a Go binary — would require spawning a process and piping, losing type safety and preview control
- skim's `SkimItem` trait gives us full control over display, preview, and output format
- MIT license — identical to our target license

### Why kube-rs over shelling out to kubectl?
- `kubectl` subprocess calls are slow and require kubectl installed
- kube-rs uses the same kubeconfig, works identically
- Async streaming via `kube::runtime::watcher` feeds skim's item channel perfectly
- Type-safe resource handling

### Multi-select Action Model
When user selects multiple items and hits a bound key:
- All selected items receive the same action
- Actions that are dangerous (delete) show a confirmation with count
- Actions that don't make sense in bulk (exec) use only the first selected item

### Status-based Sorting
Failed/Pending pods surface above Running pods in default sort.
This is implemented via `SkimItem`'s ranking — failed resources get a score boost.
