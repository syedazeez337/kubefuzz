# KubeRift — Remediation Blueprint (Part 2: Tests, Features, Bug Fixes)

> **STATUS: COMPLETED** — All items in this document were implemented in commits `ad6f0d6`, `f00caba`, and `bd3de1a` (2026-02-23 to 2026-02-25). This file is kept as a historical record.

> **Continued from [REMEDIATION_PART1.md](REMEDIATION_PART1.md)**

---

## Table of Contents — Part 2

4. [TEST: 100% Test Coverage Implementation](#test-100-test-coverage-implementation)
5. [FEAT: Feature Completeness Fixes](#feat-feature-completeness-fixes)
6. [BUG: Logic Bug Fixes](#bug-logic-bug-fixes)
7. [DOC: Documentation Accuracy Fixes](#doc-documentation-accuracy-fixes)

---

## TEST: 100% Test Coverage Implementation

### TEST-001: Create test module in `src/items.rs`

**Append to the bottom of `src/items.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ── ResourceKind::as_str ──────────────────────────────────────────────────

    #[test]
    fn kind_as_str_all_variants() {
        assert_eq!(ResourceKind::Pod.as_str(), "pod");
        assert_eq!(ResourceKind::Service.as_str(), "svc");
        assert_eq!(ResourceKind::Deployment.as_str(), "deploy");
        assert_eq!(ResourceKind::StatefulSet.as_str(), "sts");
        assert_eq!(ResourceKind::DaemonSet.as_str(), "ds");
        assert_eq!(ResourceKind::ConfigMap.as_str(), "cm");
        assert_eq!(ResourceKind::Secret.as_str(), "secret");
        assert_eq!(ResourceKind::Ingress.as_str(), "ing");
        assert_eq!(ResourceKind::Node.as_str(), "node");
        assert_eq!(ResourceKind::Namespace.as_str(), "ns");
        assert_eq!(ResourceKind::PersistentVolumeClaim.as_str(), "pvc");
        assert_eq!(ResourceKind::Job.as_str(), "job");
        assert_eq!(ResourceKind::CronJob.as_str(), "cronjob");
    }

    // ── ResourceKind::color ───────────────────────────────────────────────────

    #[test]
    fn kind_color_pod_is_green() {
        assert_eq!(ResourceKind::Pod.color(), Color::Green);
    }

    #[test]
    fn kind_color_service_is_blue() {
        assert_eq!(ResourceKind::Service.color(), Color::Blue);
    }

    #[test]
    fn kind_color_workloads_are_yellow() {
        assert_eq!(ResourceKind::Deployment.color(), Color::Yellow);
        assert_eq!(ResourceKind::StatefulSet.color(), Color::Yellow);
        assert_eq!(ResourceKind::DaemonSet.color(), Color::Yellow);
    }

    // ── StatusHealth::classify (after RST-004 is applied) ─────────────────────

    #[test]
    fn status_health_critical_exact() {
        for s in &["Failed", "Error", "OOMKilled", "NotReady", "Lost", "Evicted", "BackOff"] {
            assert_eq!(StatusHealth::classify(s), StatusHealth::Critical, "status '{s}' should be Critical");
        }
    }

    #[test]
    fn status_health_critical_prefix() {
        for s in &["CrashLoopBackOff", "ErrImagePull", "ImagePullBackOff",
                    "Init:ErrImagePull", "Init:Error", "Init:ImagePullBackOff", "Failed(3)"] {
            assert_eq!(StatusHealth::classify(s), StatusHealth::Critical, "status '{s}' should be Critical");
        }
    }

    #[test]
    fn status_health_warning() {
        for s in &["Pending", "Terminating", "ContainerCreating", "Unknown", "Init:0/1", "Init:2/3"] {
            assert_eq!(StatusHealth::classify(s), StatusHealth::Warning, "status '{s}' should be Warning");
        }
    }

    #[test]
    fn status_health_deleted() {
        assert_eq!(StatusHealth::classify("[DELETED]"), StatusHealth::Unknown);
    }

    #[test]
    fn status_health_healthy_exact() {
        for s in &["Running", "Active", "Bound", "Complete", "Succeeded", "Ready",
                    "Scheduled", "ClusterIP", "NodePort", "LoadBalancer"] {
            assert_eq!(StatusHealth::classify(s), StatusHealth::Healthy, "status '{s}' should be Healthy");
        }
    }

    #[test]
    fn status_health_ratio_equal_is_healthy() {
        assert_eq!(StatusHealth::classify("3/3"), StatusHealth::Healthy);
        assert_eq!(StatusHealth::classify("1/1"), StatusHealth::Healthy);
    }

    #[test]
    fn status_health_ratio_unequal_is_warning() {
        assert_eq!(StatusHealth::classify("0/3"), StatusHealth::Warning);
        assert_eq!(StatusHealth::classify("2/3"), StatusHealth::Warning);
    }

    #[test]
    fn status_health_active_prefix_is_healthy() {
        assert_eq!(StatusHealth::classify("Active(2)"), StatusHealth::Healthy);
    }

    #[test]
    fn status_health_unknown_string_defaults_healthy() {
        assert_eq!(StatusHealth::classify("SomeNewStatus"), StatusHealth::Healthy);
    }

    // ── status_color ──────────────────────────────────────────────────────────

    #[test]
    fn status_color_running_is_green() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "p", "Running", "1d", "");
        assert_eq!(item.status_color(), Color::Green);
    }

    #[test]
    fn status_color_crashloop_is_red() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "p", "CrashLoopBackOff", "1d", "");
        assert_eq!(item.status_color(), Color::Red);
    }

    #[test]
    fn status_color_pending_is_yellow() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "p", "Pending", "1d", "");
        assert_eq!(item.status_color(), Color::Yellow);
    }

    #[test]
    fn status_color_deleted_is_gray() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "p", "[DELETED]", "1d", "");
        assert_eq!(item.status_color(), Color::DarkGray);
    }

    // ── output_str ────────────────────────────────────────────────────────────

    #[test]
    fn output_str_with_namespace() {
        let item = K8sItem::new(ResourceKind::Pod, "default", "nginx", "Running", "1d", "");
        assert_eq!(item.output_str(), "pod/default/nginx");
    }

    #[test]
    fn output_str_without_namespace() {
        let item = K8sItem::new(ResourceKind::Node, "", "node-1", "Ready", "7d", "");
        assert_eq!(item.output_str(), "node/node-1");
    }

    #[test]
    fn output_str_with_context() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "p", "Running", "1d", "prod");
        assert_eq!(item.output_str(), "prod:pod/ns/p");
    }

    #[test]
    fn output_str_no_namespace_with_context() {
        let item = K8sItem::new(ResourceKind::Namespace, "", "default", "Active", "30d", "prod");
        assert_eq!(item.output_str(), "prod:ns/default");
    }

    // ── context_color ─────────────────────────────────────────────────────────

    #[test]
    fn context_color_deterministic() {
        let c1 = context_color("prod");
        let c2 = context_color("prod");
        assert_eq!(c1, c2, "same input must give same color");
    }

    #[test]
    fn context_color_different_inputs_may_differ() {
        // Not guaranteed to differ, but at least should not panic
        let _ = context_color("prod");
        let _ = context_color("staging");
        let _ = context_color("");
    }

    // ── text() — SkimItem ─────────────────────────────────────────────────────

    #[test]
    fn text_contains_kind_and_name() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "nginx", "Running", "1d", "");
        let t = item.text();
        assert!(t.contains("pod"), "text should contain kind");
        assert!(t.contains("nginx"), "text should contain name");
        assert!(t.contains("ns/"), "text should contain namespace prefix");
    }

    #[test]
    fn text_truncates_long_names() {
        let long_name = "a".repeat(50);
        let item = K8sItem::new(ResourceKind::Pod, "ns", &long_name, "Running", "1d", "");
        let t = item.text();
        assert!(t.contains("…"), "long names should be truncated with ellipsis");
        assert!(!t.contains(&long_name), "full long name should not appear");
    }

    #[test]
    fn text_includes_context_prefix() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "p", "Running", "1d", "prod-cluster");
        let t = item.text();
        assert!(t.contains("prod-cluster/"), "multi-cluster text should include context");
    }

    // ── truncate_name (after RST-001 is applied) ──────────────────────────────

    #[test]
    fn truncate_short_name_unchanged() {
        assert_eq!(truncate_name("nginx", 31).as_ref(), "nginx");
    }

    #[test]
    fn truncate_exact_boundary() {
        let name = "a".repeat(31);
        assert_eq!(truncate_name(&name, 31).as_ref(), &name);
    }

    #[test]
    fn truncate_long_name_gets_ellipsis() {
        let name = "a".repeat(40);
        let result = truncate_name(&name, 31);
        assert!(result.contains("…"));
        assert!(result.len() <= 31 + "…".len());
    }
}
```

---

### TEST-002: Create test module in `src/k8s/resources.rs`

**Append to the bottom of `src/k8s/resources.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::{
        ContainerState, ContainerStateWaiting, ContainerStateTerminated,
        ContainerStatus, PodStatus,
    };

    fn make_pod(status: Option<PodStatus>, deletion: bool) -> Pod {
        let mut pod = Pod::default();
        pod.metadata.name = Some("test-pod".to_string());
        pod.status = status;
        if deletion {
            pod.metadata.deletion_timestamp = Some(k8s_openapi::apimachinery::pkg::apis::meta::v1::Time(
                k8s_openapi::jiff::Timestamp::now(),
            ));
        }
        pod
    }

    #[test]
    fn pod_status_terminating() {
        let pod = make_pod(None, true);
        assert_eq!(pod_status(&pod), "Terminating");
    }

    #[test]
    fn pod_status_no_status_is_unknown() {
        let pod = make_pod(None, false);
        assert_eq!(pod_status(&pod), "Unknown");
    }

    #[test]
    fn pod_status_running() {
        let pod = make_pod(Some(PodStatus {
            phase: Some("Running".to_string()),
            ..Default::default()
        }), false);
        assert_eq!(pod_status(&pod), "Running");
    }

    #[test]
    fn pod_status_crashloop() {
        let pod = make_pod(Some(PodStatus {
            container_statuses: Some(vec![ContainerStatus {
                state: Some(ContainerState {
                    waiting: Some(ContainerStateWaiting {
                        reason: Some("CrashLoopBackOff".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            ..Default::default()
        }), false);
        assert_eq!(pod_status(&pod), "CrashLoopBackOff");
    }

    #[test]
    fn pod_status_init_progress() {
        let pod = make_pod(Some(PodStatus {
            init_container_statuses: Some(vec![
                ContainerStatus {
                    state: Some(ContainerState {
                        // running, not terminated
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }), false);
        assert_eq!(pod_status(&pod), "Init:0/1");
    }

    #[test]
    fn pod_status_terminated_nonzero_exit() {
        let pod = make_pod(Some(PodStatus {
            container_statuses: Some(vec![ContainerStatus {
                state: Some(ContainerState {
                    terminated: Some(ContainerStateTerminated {
                        exit_code: 1,
                        reason: Some("OOMKilled".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            ..Default::default()
        }), false);
        assert_eq!(pod_status(&pod), "OOMKilled");
    }

    // ── Service status ────────────────────────────────────────────────────────

    #[test]
    fn service_status_default_clusterip() {
        let svc = Service::default();
        assert_eq!(service_status(&svc), "ClusterIP");
    }

    // ── Deploy status ─────────────────────────────────────────────────────────

    #[test]
    fn deploy_status_ready() {
        let mut d = Deployment::default();
        d.spec = Some(k8s_openapi::api::apps::v1::DeploymentSpec {
            replicas: Some(3),
            ..Default::default()
        });
        d.status = Some(k8s_openapi::api::apps::v1::DeploymentStatus {
            ready_replicas: Some(3),
            ..Default::default()
        });
        assert_eq!(deploy_status(&d), "3/3");
    }

    #[test]
    fn deploy_status_degraded() {
        let mut d = Deployment::default();
        d.spec = Some(k8s_openapi::api::apps::v1::DeploymentSpec {
            replicas: Some(3),
            ..Default::default()
        });
        // No ready_replicas → 0
        assert_eq!(deploy_status(&d), "0/3");
    }

    // ── Status priority ───────────────────────────────────────────────────────

    #[test]
    fn priority_critical_statuses() {
        assert_eq!(status_priority("CrashLoopBackOff"), 0);
        assert_eq!(status_priority("ImagePullBackOff"), 0);
        assert_eq!(status_priority("Error"), 0);
        assert_eq!(status_priority("Failed"), 0);
        assert_eq!(status_priority("OOMKilled"), 0);
        assert_eq!(status_priority("NotReady"), 0);
        assert_eq!(status_priority("Failed(3)"), 0);
    }

    #[test]
    fn priority_warning_statuses() {
        assert_eq!(status_priority("[DELETED]"), 1);
        assert_eq!(status_priority("Pending"), 1);
        assert_eq!(status_priority("ContainerCreating"), 1);
        assert_eq!(status_priority("Init:0/1"), 1);
    }

    #[test]
    fn priority_healthy_statuses() {
        assert_eq!(status_priority("Running"), 2);
        assert_eq!(status_priority("Active"), 2);
        assert_eq!(status_priority("ClusterIP"), 2);
        assert_eq!(status_priority("Complete"), 2);
    }

    #[test]
    fn resource_age_no_timestamp() {
        let meta = ObjectMeta::default();
        assert_eq!(resource_age(&meta), "?");
    }
}
```

---

### TEST-003: Create test module in `src/cli.rs`

**Append to the bottom of `src/cli.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn args_with_resource(r: &str) -> Args {
        Args {
            resource: Some(r.to_string()),
            all_contexts: false,
            context: None,
        }
    }

    #[test]
    fn filter_pod_aliases() {
        for alias in &["pod", "pods", "po"] {
            let args = args_with_resource(alias);
            let kinds = args.resource_filter().expect("should resolve");
            assert_eq!(kinds, vec![ResourceKind::Pod], "alias '{alias}' should map to Pod");
        }
    }

    #[test]
    fn filter_service_aliases() {
        for alias in &["svc", "service", "services"] {
            let args = args_with_resource(alias);
            let kinds = args.resource_filter().expect("should resolve");
            assert_eq!(kinds, vec![ResourceKind::Service]);
        }
    }

    #[test]
    fn filter_deploy_aliases() {
        for alias in &["deploy", "deployment", "deployments"] {
            let args = args_with_resource(alias);
            let kinds = args.resource_filter().expect("should resolve");
            assert_eq!(kinds, vec![ResourceKind::Deployment]);
        }
    }

    #[test]
    fn filter_all_other_aliases() {
        let cases = vec![
            ("sts", ResourceKind::StatefulSet),
            ("ds", ResourceKind::DaemonSet),
            ("cm", ResourceKind::ConfigMap),
            ("secret", ResourceKind::Secret),
            ("ing", ResourceKind::Ingress),
            ("node", ResourceKind::Node),
            ("ns", ResourceKind::Namespace),
            ("pvc", ResourceKind::PersistentVolumeClaim),
            ("job", ResourceKind::Job),
            ("cj", ResourceKind::CronJob),
        ];
        for (alias, expected) in cases {
            let args = args_with_resource(alias);
            let kinds = args.resource_filter().expect(&format!("alias '{alias}' should resolve"));
            assert_eq!(kinds, vec![expected], "alias '{alias}' mismatch");
        }
    }

    #[test]
    fn filter_unknown_returns_none() {
        let args = args_with_resource("unknowntype");
        assert!(args.resource_filter().is_none());
    }

    #[test]
    fn filter_case_insensitive() {
        let args = args_with_resource("PODS");
        let kinds = args.resource_filter().expect("should resolve case-insensitive");
        assert_eq!(kinds, vec![ResourceKind::Pod]);
    }

    #[test]
    fn filter_none_when_no_resource() {
        let args = Args { resource: None, all_contexts: false, context: None };
        assert!(args.resource_filter().is_none());
    }
}
```

---

### TEST-004: Create integration test file

**Create `tests/cli_integration.rs` (new file):**

```rust
//! Integration tests for the `kf` binary.
//! These test CLI arg parsing and help output without needing a cluster.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_flag() {
    Command::cargo_bin("kf")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Fuzzy-first interactive Kubernetes resource navigator"))
        .stdout(predicate::str::contains("--all-contexts"))
        .stdout(predicate::str::contains("--context"));
}

#[test]
fn version_flag() {
    Command::cargo_bin("kf")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("kf"));
}
```

---

## FEAT: Feature Completeness Fixes

### FEAT-001: Add `--namespace` flag

**File:** `src/cli.rs` — add to the `Args` struct:

```rust
/// Restrict to a specific namespace. Default: all namespaces.
#[arg(short = 'n', long, value_name = "NAMESPACE")]
pub namespace: Option<String>,
```

**File:** `src/k8s/resources.rs` — update `watch_typed` to accept namespace:
- Change `Api::<T>::all(client)` to:
  ```rust
  let api: Api<T> = match namespace {
      Some(ns) => Api::namespaced(client, ns),
      None => Api::all(client),
  };
  ```
- Thread the namespace parameter through `watch_resources` → `watch_typed`

**File:** `src/main.rs` — pass `args.namespace.as_deref()` to `watch_resources`.

---

### FEAT-002: Add `--read-only` flag

**File:** `src/cli.rs` — add:
```rust
/// Disable all write/exec actions (describe, logs, yaml only).
#[arg(long)]
pub read_only: bool,
```

**File:** `src/main.rs` `dispatch()` — wrap destructive actions:
```rust
if ctrl('d') {
    if read_only { eprintln!("[kuberift] delete disabled in read-only mode"); }
    else { action_delete(&items)?; }
} else if ctrl('e') {
    if read_only { eprintln!("[kuberift] exec disabled in read-only mode"); }
    else if let Some(item) = items.first() { action_exec(item)?; }
} else if ctrl('r') {
    if read_only { eprintln!("[kuberift] restart disabled in read-only mode"); }
    else { action_rollout_restart(&items)?; }
}
// ctrl-l (logs), ctrl-y (yaml), enter (describe) remain available
```

**Thread `read_only` through `run_single_context` → `dispatch`.**

---

### FEAT-003: Add `--kubeconfig` flag

**File:** `src/cli.rs` — add:
```rust
/// Path to kubeconfig file. Default: $KUBECONFIG or ~/.kube/config.
#[arg(long, value_name = "PATH")]
pub kubeconfig: Option<String>,
```

**File:** `src/k8s/client.rs` — update `build_client_for_context`:
```rust
pub async fn build_client_for_context(context_name: &str, kubeconfig: Option<&str>) -> Result<Client> {
    let options = KubeConfigOptions {
        context: Some(context_name.to_string()),
        ..Default::default()
    };
    let config = match kubeconfig {
        Some(path) => {
            let kc = kube::config::Kubeconfig::read_from(path)?;
            kube::Config::from_custom_kubeconfig(kc, &options).await?
        }
        None => kube::Config::from_kubeconfig(&options).await?,
    };
    Client::try_from(config).context("Failed to build Kubernetes client")
}
```

---

## BUG: Logic Bug Fixes

### BUG-001: Fix stale preview mode label in header

**File:** `src/main.rs` line 206-246

The header is built once with `preview_mode_label()`. It never updates when ctrl-p is pressed.

**This is a limitation of skim's API** — the header is set at startup. The partial fix is to acknowledge this in the header text:

```rust
.header(format!(
    "KubeRift  ctx:{ctx_label}  res:{kind_label}\n\
     <tab> select  <enter> describe  ctrl-l logs  ctrl-e exec  \
     ctrl-d delete  ctrl-f forward  ctrl-r restart  ctrl-y yaml  \
     ctrl-p cycle-preview{ctx_hint}",
))
```

Remove the `preview:{label}` from the header since it can't update dynamically.

---

### BUG-002: Don't exit after single action — re-enter skim loop

**File:** `src/main.rs` lines 107-108

**Change:**
```rust
dispatch(output).await?;
break;
```
**To:**
```rust
dispatch(output)?;
// Re-enter the loop to show skim again after the action completes
install_preview_toggle();
continue;
```

This makes the navigator truly interactive — users perform an action and return to the fuzzy list.

---

### BUG-003: Fix `Terminating` not being in `status_priority` warning tier

**Already fixed by RST-004** (`StatusHealth::classify` unification). No additional work needed if RST-004 is implemented. Verify the test for `"Terminating"` returns `StatusHealth::Warning`.

---

## DOC: Documentation Accuracy Fixes

### DOC-001: Rewrite ARCHITECTURE.md to match actual code

**File:** `docs/ARCHITECTURE.md`

**Delete the entire current content and replace with a document that accurately describes:**

1. **Actual directory structure** (no `config.rs`, no `preview.rs`, no `watcher.rs`, no `tests/`)
2. **Actual `Cargo.toml` dependencies** (no `thiserror`, no `arboard`, no `owo-colors`)
3. **Actual `ResourceKind` variants** (no `ReplicaSet`)
4. **Actual `K8sItem` fields** (no `raw: serde_json::Value`, no `ResourceStatus` enum)
5. **Actual CLI flags** (no `--namespace`, `--read-only`, `--preview` — unless FEAT-001/002 are implemented first)
6. **Actual `Action` set** (no `Edit`, no `CopyName`, no `SwitchContext` as an action variant)

**Key rule: the architecture doc must be generated FROM the code, not the other way around. Diff every claim against the actual source before writing.**

---

### DOC-002: Fix README install instructions

**File:** `README.md` lines 35-42

**Add the skim clone step:**
```markdown
## Installation

```bash
# Clone both repos (kuberift depends on a patched skim)
git clone https://github.com/syedazeez337/skim.git
git clone https://github.com/syedazeez337/kuberift.git
cd kuberift
cargo build --release
```

Or, if INFRA-001 is fixed (git dependency), the current instructions are fine.

---

## Execution Checklist

Apply in this order to avoid conflicts:

- [ ] **RST-004** — Create `StatusHealth` enum (changes `items.rs` and `resources.rs`)
- [ ] **RST-001** — Add `truncate_name` helper (changes `items.rs`)
- [ ] **RST-002** — Add derives to `ResourceKind` (changes `items.rs`, ripple to `resources.rs`)
- [ ] **RST-003** — Make `K8sItem` fields private (changes `items.rs`, `actions.rs`, `resources.rs`, `main.rs`)
- [ ] **SEC-001** — Replace `/tmp` paths (changes `actions.rs`, `main.rs`)
- [ ] **SEC-002** — Add `--` separators (changes `actions.rs`, `items.rs`)
- [ ] **SEC-003** — Validate port numbers (changes `actions.rs`)
- [ ] **SEC-004** — Bulk delete warning (changes `actions.rs`)
- [ ] **SEC-005** — Restrict file permissions (changes `client.rs`)
- [ ] **RST-005** — Remove async from dispatch (changes `main.rs`)
- [ ] **RST-006** — Remove expect (changes `main.rs`)
- [ ] **RST-007** — Move imports (changes `main.rs`)
- [ ] **RST-008** — Add clippy config (changes `main.rs`)
- [ ] **RST-009** — Replace `let _ =` with logging (changes `main.rs`, `resources.rs`)
- [ ] **BUG-001** — Fix stale header (changes `main.rs`)
- [ ] **BUG-002** — Re-enter loop after action (changes `main.rs`)
- [ ] **FEAT-001** — Add `--namespace` (changes `cli.rs`, `resources.rs`, `main.rs`)
- [ ] **FEAT-002** — Add `--read-only` (changes `cli.rs`, `main.rs`)
- [ ] **FEAT-003** — Add `--kubeconfig` (changes `cli.rs`, `client.rs`, `main.rs`)
- [ ] **INFRA-001** — Fix skim dependency (changes `Cargo.toml`)
- [ ] **INFRA-002** — Add CI pipeline (new file `.github/workflows/ci.yml`)
- [ ] **INFRA-003** — Trim tokio features (changes `Cargo.toml`)
- [ ] **INFRA-004** — Add dev-dependencies (changes `Cargo.toml`)
- [ ] **TEST-001** — Tests for items.rs (changes `src/items.rs`)
- [ ] **TEST-002** — Tests for resources.rs (changes `src/k8s/resources.rs`)
- [ ] **TEST-003** — Tests for cli.rs (changes `src/cli.rs`)
- [ ] **TEST-004** — Integration tests (new file `tests/cli_integration.rs`)
- [ ] **DOC-001** — Rewrite ARCHITECTURE.md
- [ ] **DOC-002** — Fix README install instructions
- [ ] Run `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`
- [ ] Verify all tests pass with `cargo test -- --nocapture`
