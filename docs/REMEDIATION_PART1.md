# KubeFuzz — Remediation Blueprint (Part 1: Security, Rust Standards, Infrastructure)

> **STATUS: COMPLETED** — All items in this document were implemented in commits `ad6f0d6`, `88b35eb`, `5e7c223`, and `f00caba` (2026-02-23 to 2026-02-25). This file is kept as a historical record.

> **Purpose**: This document was a precise, actionable remediation plan. Every item includes the exact file, line, current code, replacement code, and rationale.

---

## Table of Contents — Part 1

1. [SEC: Security Fixes (10 items)](#sec-security-fixes)
2. [RST: Rust Standard Practice Fixes (14 items)](#rst-rust-standard-practice-fixes)
3. [INFRA: Infrastructure Fixes (CI, Dependencies, Logging)](#infra-infrastructure-fixes)

---

## SEC: Security Fixes

### SEC-001: Replace hardcoded `/tmp` paths with secure per-user directory

**Files:** `src/actions.rs` lines 11-12, 17-28, 32  
**Severity:** CRITICAL — CWE-59 (Symlink), CWE-367 (TOCTOU)

**Current code (actions.rs:11-12):**
```rust
const PREVIEW_MODE_FILE: &str = "/tmp/kubefuzz-preview-mode";
pub const PREVIEW_TOGGLE_SCRIPT: &str = "/tmp/kubefuzz-preview-toggle";
```

**Replace with:**
```rust
use std::path::PathBuf;
use std::sync::OnceLock;

/// Returns a secure, per-process runtime directory for temp files.
/// Uses XDG_RUNTIME_DIR (Linux), or falls back to a PID-scoped temp dir.
fn runtime_dir() -> &'static PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let base = dirs::runtime_dir()
            .or_else(|| std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from))
            .unwrap_or_else(std::env::temp_dir);
        let dir = base.join(format!("kubefuzz-{}", std::process::id()));
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("[kubefuzz] warning: cannot create runtime dir: {e}");
        }
        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
        }
        dir
    })
}

fn preview_mode_path() -> PathBuf {
    runtime_dir().join("preview-mode")
}

fn preview_toggle_path() -> PathBuf {
    runtime_dir().join("preview-toggle")
}
```

**Then update every reference:**
- `PREVIEW_MODE_FILE` → `preview_mode_path()` (returns `PathBuf`)
- `PREVIEW_TOGGLE_SCRIPT` → `preview_toggle_path()` (returns `PathBuf`)
- `install_preview_toggle()` must use `preview_toggle_path().to_str().unwrap()` in the script content and skim binding
- `current_preview_mode()` must read from `preview_mode_path()`
- `build_skim_options()` in `main.rs` line 236: replace `PREVIEW_TOGGLE_SCRIPT` with `preview_toggle_path().display()`

**Update `install_preview_toggle()`:**
```rust
pub fn install_preview_toggle() {
    let mode_path = preview_mode_path();
    let toggle_path = preview_toggle_path();
    let mode_str = mode_path.display();
    let script = format!(
        "#!/bin/sh\n\
         n=$(cat \"{mode_str}\" 2>/dev/null || echo 0)\n\
         printf $(( (n + 1) % 3 )) > \"{mode_str}\"\n"
    );
    if let Err(e) = std::fs::write(&toggle_path, &script) {
        eprintln!("[kubefuzz] warning: cannot write preview toggle script: {e}");
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&toggle_path, std::fs::Permissions::from_mode(0o700));
    }
    if let Err(e) = std::fs::write(&mode_path, "0") {
        eprintln!("[kubefuzz] warning: cannot write preview mode file: {e}");
    }
}
```

**Update `current_preview_mode()`:**
```rust
pub fn current_preview_mode() -> u8 {
    std::fs::read_to_string(preview_mode_path())
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(0)
        % 3
}
```

**Update `build_skim_options()` in main.rs:**
```rust
// Replace line 236:
format!("ctrl-p:execute({})+refresh-preview", preview_toggle_path().display()),
```

**Also export `preview_toggle_path` instead of `PREVIEW_TOGGLE_SCRIPT`:**
- In `main.rs`, change import from `PREVIEW_TOGGLE_SCRIPT` to `preview_toggle_path`

**Add cleanup on exit** — add to `main.rs` after skim exits:
```rust
// Cleanup runtime files
let _ = std::fs::remove_dir_all(actions::runtime_dir());
```

---

### SEC-002: Add `--` separator before resource names in all kubectl calls

**Files:** `src/actions.rs` lines 76, 97, 144, 194, 228, 256, 277. Also `src/items.rs` lines 237-243.  
**Severity:** MEDIUM — CWE-88

**Pattern — current:**
```rust
let mut args = vec!["logs", "--tail=200", &item.name];
```

**Pattern — fixed:**
```rust
let mut args = vec!["logs", "--tail=200", "--", &item.name];
```

**Apply this to every action function. Full list of changes:**

| Function | Line | Current last positional arg | Add `"--"` before |
|----------|------|----------------------------|-------------------|
| `action_logs` | 76 | `&item.name` | `vec!["logs", "--tail=200", "--", &item.name]` |
| `action_exec` | 97 | `&item.name` | `vec!["exec", "-it", "--", &item.name]` — but note `--` for exec goes before the container command, so this needs: `vec!["exec", "-it", &item.name, "-n", &ns, "--", shell]` (already correct on line 101) |
| `action_delete` | 144 | `&item.name` | `vec!["delete", item.kind.as_str(), "--", &item.name]` |
| `action_rollout_restart` | 228 | `&target` | The target is `kind/name` format, kubectl interprets this correctly. BUT add validation that name doesn't start with `-` |
| `action_yaml` | 256 | `&item.name` | `vec!["get", item.kind.as_str(), "--", &item.name, "-o", "yaml"]` — note: `--` must come after subcommand flags |
| `action_describe` | 277 | `&item.name` | `vec!["describe", item.kind.as_str(), "--", &item.name]` |
| `preview()` in items.rs | 238-242 | `&self.name` | Add `"--"` before `&self.name` in all three branches |

**For `action_exec` specifically — line 97-101, the current code is actually safe because `--` is already used before the shell command. But the pod name itself should also be protected:**
```rust
let mut args = vec!["exec", "-it"];
if !item.namespace.is_empty() {
    args.extend_from_slice(&["-n", &item.namespace]);
}
args.extend_from_slice(&["--", &item.name, "--", shell]);
```
Wait — kubectl exec syntax is: `kubectl exec -it <pod> -n <ns> -- <command>`. The pod name is positional. The fix for exec is to validate the name doesn't start with `-`:
```rust
if item.name.starts_with('-') {
    eprintln!("[kubefuzz] suspicious resource name: {}", item.name);
    return Ok(());
}
```

---

### SEC-003: Validate port numbers in port-forward

**File:** `src/actions.rs` lines 174-189

**Replace the port reading section with:**
```rust
fn read_port(prompt: &str, default: Option<&str>) -> Result<Option<u16>> {
    let prompt_str = match default {
        Some(d) => format!("{prompt} [{d}]: "),
        None => format!("{prompt}: "),
    };
    print!("{prompt_str}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if input.is_empty() {
        return match default {
            Some(d) => Ok(Some(d.parse::<u16>().map_err(|_| anyhow::anyhow!("Invalid default port"))?)),
            None => Ok(None),
        };
    }
    let port: u16 = input.parse().map_err(|_| anyhow::anyhow!("'{input}' is not a valid port number (1-65535)"))?;
    if port == 0 {
        anyhow::bail!("Port 0 is not valid");
    }
    if port < 1024 {
        eprintln!("[kubefuzz] warning: port {port} is privileged (may require root/admin)");
    }
    Ok(Some(port))
}
```

**Then update `action_portforward`:**
```rust
pub fn action_portforward(item: &K8sItem) -> Result<()> {
    if !matches!(item.kind, ResourceKind::Pod | ResourceKind::Service) {
        eprintln!("[kubefuzz] port-forward only works with pods and services (got {})", item.kind.as_str());
        return Ok(());
    }

    let local = match read_port("Local port", None)? {
        Some(p) => p,
        None => { println!("Cancelled."); return Ok(()); }
    };
    let remote = read_port("Remote port", Some(&local.to_string()))?.unwrap_or(local);

    let target = format!("{}/{}", item.kind.as_str(), item.name);
    let ports = format!("{local}:{remote}");
    let mut args = vec!["port-forward", &target, &ports];
    if !item.namespace.is_empty() {
        args.extend_from_slice(&["-n", &item.namespace]);
    }
    println!("Forwarding localhost:{local} → {target} port {remote}  (Ctrl-C to stop)");
    let status = kubectl(item).args(&args).status()?;
    if !status.success() {
        eprintln!("[kubefuzz] port-forward exited with {status}");
    }
    Ok(())
}
```

---

### SEC-004: Add batch size warning for bulk delete

**File:** `src/actions.rs` lines 113-161

**After the count check (line 114), add:**
```rust
if count > 10 {
    eprintln!("[kubefuzz] ⚠ WARNING: You are about to delete {count} resources.");
    print!("Type 'yes' (not just 'y') to confirm bulk delete: ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    if confirm.trim() != "yes" {
        println!("Cancelled.");
        return Ok(());
    }
} else {
    print!("\nDelete {count} {noun}? [y/N] ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Cancelled.");
        return Ok(());
    }
}
```

---

### SEC-005: Set restrictive permissions on context file

**File:** `src/k8s/client.rs` lines 36-41

**Replace `save_last_context`:**
```rust
pub fn save_last_context(context: &str) {
    if let Some(dir) = dirs::config_dir() {
        let dir = dir.join("kubefuzz");
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("[kubefuzz] warning: cannot create config dir: {e}");
            return;
        }
        let path = dir.join("last_context");
        if let Err(e) = std::fs::write(&path, context) {
            eprintln!("[kubefuzz] warning: cannot save context: {e}");
            return;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
    }
}
```

---

## RST: Rust Standard Practice Fixes

### RST-001: Fix panicking string slice

**File:** `src/items.rs` lines 171-175, 211-215

**Add this helper function at the top of `items.rs` (after the `use` statements):**
```rust
/// Truncate a string to at most `max_chars` characters, appending "…" if truncated.
/// Always splits on a valid UTF-8 char boundary.
fn truncate_name(name: &str, max_chars: usize) -> Cow<'_, str> {
    if name.len() <= max_chars {
        return Cow::Borrowed(name);
    }
    // Find the last char boundary at or before max_chars
    let mut end = max_chars;
    while !name.is_char_boundary(end) && end > 0 {
        end -= 1;
    }
    Cow::Owned(format!("{}…", &name[..end]))
}
```

**Replace lines 171-175:**
```rust
let name_truncated = truncate_name(&self.name, 31);
```

**Replace lines 211-215:**
```rust
let name_col = {
    let truncated = truncate_name(&self.name, 31);
    if truncated.len() < 32 {
        format!("{:<32} ", truncated)
    } else {
        format!("{} ", truncated)
    }
};
```

---

### RST-002: Add standard derives and attributes to `ResourceKind`

**File:** `src/items.rs` lines 8-24

**Replace the enum declaration:**
```rust
/// The kind of Kubernetes resource this item represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ResourceKind {
    Pod,
    Service,
    Deployment,
    StatefulSet,
    DaemonSet,
    ConfigMap,
    Secret,
    Ingress,
    Node,
    Namespace,
    PersistentVolumeClaim,
    Job,
    CronJob,
}
```

**Add `Display` impl after the `impl ResourceKind` block:**
```rust
impl std::fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
```

**After adding `Copy`, search the entire codebase for `.clone()` on `ResourceKind` values and remove them:**
- `src/k8s/resources.rs` line 57: `let k = kind.clone();` → `let k = *kind;`
- `src/k8s/resources.rs` line 277: `K8sItem::new(kind.clone(), ...)` → `K8sItem::new(*kind, ...)`
- `src/main.rs` line 38: `kinds[0].as_str()` — no change needed, `Copy` works via auto-deref

---

### RST-003: Make `K8sItem` fields private, add constructor with all fields

**File:** `src/items.rs` lines 59-87

**Replace the struct and constructor:**
```rust
/// A Kubernetes resource item displayed in the skim TUI.
#[derive(Debug, Clone)]
pub struct K8sItem {
    kind: ResourceKind,
    namespace: String,
    name: String,
    status: String,
    age: String,
    context: String,
}

impl K8sItem {
    /// Create a new K8sItem. Use `context: ""` for single-cluster mode.
    pub fn new(
        kind: ResourceKind,
        namespace: impl Into<String>,
        name: impl Into<String>,
        status: impl Into<String>,
        age: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            namespace: namespace.into(),
            name: name.into(),
            status: status.into(),
            age: age.into(),
            context: context.into(),
        }
    }

    // Accessor methods
    pub fn kind(&self) -> ResourceKind { self.kind }
    pub fn namespace(&self) -> &str { &self.namespace }
    pub fn name(&self) -> &str { &self.name }
    pub fn status(&self) -> &str { &self.status }
    pub fn age(&self) -> &str { &self.age }
    pub fn context(&self) -> &str { &self.context }
}
```

**Then update ALL call sites:**

1. `src/k8s/resources.rs` line 277-279: Replace `K8sItem::new(...); item.context = ...` with single constructor call:
   ```rust
   let item = K8sItem::new(kind.clone(), ns, name, status, age, context);
   ```

2. `src/main.rs` demo_items: Add `""` as last argument to every `K8sItem::new(...)` call.

3. `src/actions.rs` — every `item.name`, `item.namespace`, `item.kind`, `item.context`, `item.status` → `item.name()`, `item.namespace()`, `item.kind()`, `item.context()`, `item.status()`. Note: `item.kind` was `ResourceKind` (a type), now use `item.kind()` which returns `ResourceKind` (Copy).

4. `src/items.rs` — internal uses within `impl K8sItem` and `impl SkimItem for K8sItem` keep using `self.name`, `self.namespace`, etc. (private field access within the impl is fine).

---

### RST-004: Fix `Evicted` duplicate and sync `status_priority` with `status_color`

**File:** `src/items.rs` lines 90-121 and `src/k8s/resources.rs` lines 284-301

**Create a shared status classification. Add to `src/items.rs`:**
```rust
/// Health category for a Kubernetes resource status string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusHealth {
    /// Critical — needs immediate attention (CrashLoopBackOff, Error, Failed, etc.)
    Critical,
    /// Warning — transitional state (Pending, Terminating, Init:X/Y, etc.)
    Warning,
    /// Healthy — normal operation (Running, Active, Complete, etc.)
    Healthy,
    /// Unknown — deleted or unrecognized
    Unknown,
}

impl StatusHealth {
    /// Classify a status string into a health category.
    pub fn classify(status: &str) -> Self {
        match status {
            // Exact critical matches
            "Failed" | "Error" | "OOMKilled" | "NotReady" | "Lost" | "Evicted" | "BackOff" => {
                Self::Critical
            }
            // Prefix-based critical matches
            s if s.starts_with("CrashLoop")
                || s.starts_with("ErrImage")
                || s.starts_with("ImagePull")
                || s.starts_with("Init:Error")
                || s.starts_with("Init:ErrImage")
                || s.starts_with("Init:ImagePull")
                || s.starts_with("Failed(") =>
            {
                Self::Critical
            }
            // Exact warning matches
            "Pending" | "Terminating" | "ContainerCreating" | "Unknown" => Self::Warning,
            // Prefix-based warning matches
            s if s.starts_with("Init:") => Self::Warning,
            // Deleted
            "[DELETED]" => Self::Unknown,
            // Exact healthy matches
            "Running" | "Active" | "Bound" | "Complete" | "Succeeded" | "Ready"
            | "Scheduled" | "ClusterIP" | "NodePort" | "LoadBalancer" => Self::Healthy,
            // Prefix-based healthy
            s if s.starts_with("Active(") => Self::Healthy,
            // Ratio: "3/3" green, "1/3" yellow
            s if s.contains('/') => {
                let parts: Vec<&str> = s.splitn(2, '/').collect();
                if parts.len() == 2 && parts[0] == parts[1] {
                    Self::Healthy
                } else {
                    Self::Warning
                }
            }
            _ => Self::Healthy, // default: assume healthy for unknown statuses
        }
    }

    /// Color for this health category.
    pub fn color(self) -> Color {
        match self {
            Self::Critical => Color::Red,
            Self::Warning => Color::Yellow,
            Self::Healthy => Color::Green,
            Self::Unknown => Color::DarkGray,
        }
    }

    /// Sort priority: 0 = top of list (critical), 1 = middle, 2 = bottom (healthy).
    pub fn priority(self) -> u8 {
        match self {
            Self::Critical => 0,
            Self::Warning | Self::Unknown => 1,
            Self::Healthy => 2,
        }
    }
}
```

**Then replace `status_color()` in `K8sItem`:**
```rust
pub fn status_color(&self) -> Color {
    StatusHealth::classify(&self.status).color()
}
```

**And replace `status_priority()` in `resources.rs`:**
```rust
pub fn status_priority(status: &str) -> u8 {
    StatusHealth::classify(status).priority()
}
```

**This eliminates:**
- The `Evicted` duplicate (ADD-006)
- The `status_priority`/`status_color` desync (ADD-007)
- Both functions now use the same classification logic

---

### RST-005: Remove `async` from `dispatch()` (unused async)

**File:** `src/main.rs` line 250

**Change:**
```rust
async fn dispatch(output: SkimOutput) -> Result<()> {
```
**To:**
```rust
fn dispatch(output: SkimOutput) -> Result<()> {
```

**Then update call sites — remove `.await`:**
- Line 107: `dispatch(output).await?;` → `dispatch(output)?;`
- Line 157: `dispatch(output).await` → `dispatch(output)`

---

### RST-006: Replace `expect()` with `?` in `build_skim_options`

**File:** `src/main.rs` line 244-245

**Change return type and propagate:**
```rust
fn build_skim_options(ctx_label: &str, kind_label: &str, show_ctx_switch: bool) -> Result<SkimOptions> {
    // ... (keep body identical)
    Ok(SkimOptionsBuilder::default()
        // ... all the builder calls ...
        .build()?)  // `?` instead of `.expect()`
}
```

**Update call sites to propagate with `?`:**
- Line 87: `let options = build_skim_options(...)?;`
- Line 149: `let options = build_skim_options(...)?;`

---

### RST-007: Move `use` imports to module top

**File:** `src/main.rs` lines 95, 251

**Remove these lines from inside function bodies:**
```rust
use crossterm::event::{KeyCode, KeyModifiers};
```

**Add once at the top of `main.rs` (after line 18):**
```rust
use crossterm::event::{KeyCode, KeyModifiers};
```

---

### RST-008: Add clippy configuration at crate root

**File:** `src/main.rs` — add at very top (line 1, before `mod` declarations):
```rust
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,  // K8sItem in items.rs is fine
    clippy::too_many_lines,           // watch_resources match is inherently long
)]
```

---

### RST-009: Replace silent `let _ =` with logging (all 8 occurrences)

**Pattern — replace every `let _ = <expr>;` with:**
```rust
if let Err(e) = <expr> {
    eprintln!("[kubefuzz] warning: <description>: {e}");
}
```

**Full list:**

| File:Line | Current | Replacement |
|-----------|---------|-------------|
| main.rs:80 | `let _ = tx_k8s.send(demo_items());` | `if tx_k8s.send(demo_items()).is_err() { eprintln!("[kubefuzz] warning: failed to send demo items"); }` |
| main.rs:173 | `let _ = tx.send(vec![...]);` | `if tx.send(vec![...]).is_err() { eprintln!("[kubefuzz] warning: failed to send context item"); }` |
| resources.rs:155 | `let _ = task.await;` | `if let Err(e) = task.await { eprintln!("[kubefuzz] warning: watcher task panicked: {e}"); }` |

SEC-001 already covers the `actions.rs` and `client.rs` occurrences.

---

## INFRA: Infrastructure Fixes

### INFRA-001: Fix skim dependency

**File:** `Cargo.toml` line 16

**Replace:**
```toml
skim = { path = "../skim", default-features = false }
```
**With (option A — git dep):**
```toml
skim = { git = "https://github.com/syedazeez337/skim", branch = "main", default-features = false }
```
**Or (option B — workspace member):** Copy the skim directory into the repo as `skim/` and use:
```toml
skim = { path = "skim", default-features = false }
```
And add a workspace `Cargo.toml` in the repo root or keep the path dep internal.

---

### INFRA-002: Add CI pipeline

**Create `.github/workflows/ci.yml`:**
```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo install cargo-audit && cargo audit
      - run: cargo build --release
```

---

### INFRA-003: Trim tokio features

**File:** `Cargo.toml` line 26

**Replace:**
```toml
tokio = { version = "1", features = ["full"] }
```
**With:**
```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
```

---

### INFRA-004: Add `tempfile` dependency for tests

**File:** `Cargo.toml` — add at end:
```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

---

> **Continued in [REMEDIATION_PART2.md](REMEDIATION_PART2.md)** — Tests (100% coverage) and Feature Fixes.
