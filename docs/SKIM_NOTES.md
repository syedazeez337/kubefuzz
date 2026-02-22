# Skim Library — Integration Notes

How skim works as a library and exactly how KubeFuzz uses it.

---

## Skim Overview

Skim (crate: `skim`, version ~0.15) is a fuzzy finder library + CLI.
We use it **as a library** — embedding the TUI directly in our binary.

Repository: https://github.com/skim-rs/skim
License: MIT
Current version: 3.4.0 (check crates.io for latest)

---

## Core Types We Use

### `SkimItem` trait

The trait you implement for items that appear in the list:

```rust
pub trait SkimItem: Send + Sync + 'static {
    /// Text skim fuzzy-matches against
    fn text(&self) -> Cow<str>;

    /// Colored display string shown in the list (optional, defaults to text())
    fn display(&self, context: DisplayContext) -> AnsiString { ... }

    /// Content shown in preview pane (optional)
    fn preview(&self, context: PreviewContext) -> ItemPreview { ... }

    /// What's written to stdout on selection (optional, defaults to text())
    fn output(&self) -> Cow<str> { ... }

    /// Custom highlight ranges (optional — skim auto-highlights matches)
    fn get_matching_ranges(&self) -> &[(usize, usize)] { ... }
}
```

For `K8sItem`, we implement all four:
- `text()` → the searchable string: `"pod production/api-server-abc123 Running 2d"`
- `display()` → colored version of text with ANSI codes
- `preview()` → kubectl describe/logs/yaml output
- `output()` → machine-parseable: `"pod/production/api-server-abc123/my-context"`

### `SkimOptionsBuilder`

Builder for all skim configuration:

```rust
let options = SkimOptionsBuilder::default()
    .multi(true)                    // enable multi-select with TAB
    .preview(Some(""))              // enable preview pane (content from SkimItem::preview)
    .preview_window(Some("right:50%"))  // position and size
    .height(Some("60%"))            // TUI takes 60% of terminal height
    .bind(vec![                     // custom keybindings
        "ctrl-l:execute-silent(echo logs {1})",
        "ctrl-e:execute(echo exec {1})",
    ])
    .header(Some("KubeFuzz — <tab> multi-select  ctrl-l logs  ctrl-e exec"))
    .prompt(Some("❯ "))
    .build()
    .unwrap();
```

### `SkimItemSender` / `SkimItemReceiver`

Channel pair for streaming items into skim asynchronously:

```rust
use skim::prelude::*;

let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

// In a background task:
tokio::spawn(async move {
    for item in fetch_k8s_resources().await {
        // Each item must be Arc<dyn SkimItem>
        tx.send(Arc::new(item)).unwrap();
    }
    // When tx drops, skim knows input is done
});

// skim consumes from rx — items appear as they arrive
let output = Skim::run_with(&options, Some(rx));
```

This is the key: we send resources to skim as they come from the K8s API — no waiting for all resources to load first.

### `Skim::run_with()`

The main blocking call that runs the TUI:

```rust
let output: Option<SkimOutput> = Skim::run_with(&options, Some(rx));
```

Returns `None` if user pressed Escape/Ctrl-C (aborted).
Returns `Some(SkimOutput)` if user accepted a selection.

### `SkimOutput`

```rust
pub struct SkimOutput {
    pub is_abort: bool,                           // true if user aborted
    pub final_key: Key,                           // which key ended the session (Enter, ctrl-d, etc.)
    pub query: String,                            // the query string user typed
    pub selected_items: Vec<Arc<dyn SkimItem>>,   // all selected items
}
```

We downcast `Arc<dyn SkimItem>` back to `Arc<K8sItem>`:

```rust
for item in output.selected_items {
    // Method 1: downcast_ref on the inner value
    if let Some(k8s) = (*item).as_any().downcast_ref::<K8sItem>() {
        handle(k8s).await?;
    }
}
```

Note: to enable downcasting, `K8sItem` must implement `as_any()`. Add this to `SkimItem` impl or use a wrapper.

---

## Preview Pane

Two ways to implement preview:

### Option A: `ItemPreview` from `SkimItem::preview()` (our approach)

```rust
impl SkimItem for K8sItem {
    fn preview(&self, _ctx: PreviewContext) -> ItemPreview {
        // Run kubectl synchronously (preview is called in a thread)
        let output = std::process::Command::new("kubectl")
            .args(["describe", &self.kind_str(), &self.name, "-n", &self.namespace])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_else(|e| format!("Error: {e}"));

        ItemPreview::Text(output)
        // Or: ItemPreview::AnsiText(output) to preserve kubectl's colors
    }
}
```

### Option B: Shell command string (simpler but less control)

```rust
SkimOptionsBuilder::default()
    .preview(Some("kubectl describe {1}"))  // {1} is the output() of hovered item
    ...
```

We use Option A because we want to:
- Intercept the preview and format it ourselves
- Support multiple preview modes (describe/logs/yaml)
- Avoid spawning extra processes for each hover

---

## Keybinding Actions

Skim's `--bind` supports these action types:

```
execute(<cmd>)         # Run cmd in foreground, suspend TUI, resume after
execute-silent(<cmd>)  # Run cmd in background, TUI stays active
reload(<cmd>)          # Re-run item source command
preview(<cmd>)         # Override preview for this key
accept                 # Accept selection (like Enter)
abort                  # Cancel
toggle-select          # Toggle current item in multi-select
select-all             # Select all
```

For KubeFuzz, we detect which key ended the session via `output.final_key` and dispatch accordingly — this avoids spawning subprocesses from skim's bind, keeping everything in Rust.

```rust
match output.final_key {
    Key::Enter      => Action::Describe,
    Key::Ctrl('l')  => Action::Logs,
    Key::Ctrl('e')  => Action::Exec,
    Key::Ctrl('d')  => Action::Delete,
    Key::Ctrl('f')  => Action::PortForward,
    Key::Ctrl('y')  => Action::PrintYaml,
    _               => Action::Describe,
}
```

---

## Async + Skim

Skim's `run_with()` is synchronous (blocks the calling thread).
Tokio doesn't like blocking calls on async executor threads.

**Solution**: Run skim on a dedicated OS thread using `tokio::task::spawn_blocking`:

```rust
let (tx, rx) = unbounded();

// Stream K8s resources from async context
tokio::spawn(stream_resources(tx));

// Run skim on a blocking thread
let output = tokio::task::spawn_blocking(move || {
    Skim::run_with(&options, Some(rx))
})
.await??;
```

Alternatively, skim has an async `run_until()` API in newer versions — check skim docs.

---

## Common Pitfalls

1. **ncurses linking on Linux**: skim depends on ncurses. Install `libncurses5-dev` (Ubuntu) or `ncurses-devel` (Fedora) before `cargo build`.

2. **Preview performance**: `SkimItem::preview()` is called every time the cursor moves. Keep it fast — cache the output or run kubectl async and return cached result.

3. **Downcast pattern**: `Arc<dyn SkimItem>` can't be downcast directly. Either store extra data in `output()` string and parse it back, or implement `as_any()` on your struct.

4. **Item updates**: Once an item is sent to skim, it can't be updated in-place. For live watching, implement a reload mechanism (skim has `reload` action, or restart skim with new items).

5. **Multi-select + Enter**: By default, `Enter` selects the current item only. With `--multi`, `Tab` selects items into a list; `Enter` then accepts all of them.

---

## Minimal Working Example

Paste this into `src/main.rs` for Phase 0 validation:

```rust
use skim::prelude::*;
use std::sync::Arc;

fn main() {
    let options = SkimOptionsBuilder::default()
        .multi(true)
        .preview(Some(""))
        .height(Some("50%"))
        .header(Some("KubeFuzz dev — press Enter to select, Esc to quit"))
        .build()
        .unwrap();

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    // Send some fake pod items
    let fake_pods = vec![
        "pod  production/api-server-abc123     Running   2d",
        "pod  production/worker-xyz789         Running   2d",
        "pod  staging/frontend-def456          Pending   5m",
        "pod  staging/db-migrator-ghi321       Failed    10m",
        "svc  production/api-service           ClusterIP 2d",
    ];

    for pod in fake_pods {
        tx.send(Arc::new(pod.to_string())).unwrap();
    }
    drop(tx); // signal end of input

    let output = Skim::run_with(&options, Some(rx));

    if let Some(out) = output {
        if !out.is_abort {
            for item in out.selected_items {
                println!("Selected: {}", item.output());
            }
        }
    }
}
```

Note: `String` already implements `SkimItem` — so you can test with raw strings before building `K8sItem`.
