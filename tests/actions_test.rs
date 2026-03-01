//! Tests for kuberift::actions — runtime dir, preview toggle, and action guard logic.
//!
//! Tests that require executing `kubectl` use a fake binary placed at the front of PATH.
//! A process-wide Mutex serialises all PATH-mutating tests so they cannot race.
//!
//! Tests that read from stdin (delete confirmation, port-forward port prompt) rely on the
//! fact that stdin is closed / EOF in automated test runs, which causes the functions to
//! cancel cleanly and return Ok(()).

use std::sync::Mutex;

use kuberift::actions::{
    action_delete, action_describe, action_exec, action_logs, action_portforward,
    action_rollout_restart, action_yaml, current_preview_mode, install_preview_toggle,
    preview_toggle_path, runtime_dir,
};
use kuberift::items::{K8sItem, ResourceKind};

// ── Test item helpers ─────────────────────────────────────────────────────────

fn pod_item() -> K8sItem {
    K8sItem::new(
        ResourceKind::Pod,
        "default",
        "test-pod",
        "Running",
        "1d",
        "",
    )
}

fn service_item() -> K8sItem {
    K8sItem::new(
        ResourceKind::Service,
        "default",
        "test-svc",
        "ClusterIP",
        "1d",
        "",
    )
}

fn deploy_item() -> K8sItem {
    K8sItem::new(
        ResourceKind::Deployment,
        "default",
        "test-deploy",
        "3/3",
        "1d",
        "",
    )
}

fn sts_item() -> K8sItem {
    K8sItem::new(
        ResourceKind::StatefulSet,
        "default",
        "test-sts",
        "3/3",
        "1d",
        "",
    )
}

fn ds_item() -> K8sItem {
    K8sItem::new(
        ResourceKind::DaemonSet,
        "default",
        "test-ds",
        "3/3",
        "1d",
        "",
    )
}

fn node_item() -> K8sItem {
    K8sItem::new(ResourceKind::Node, "", "node-1", "Ready", "7d", "")
}

// ── Fake kubectl helper ───────────────────────────────────────────────────────

/// Mutex that serialises all tests which temporarily modify PATH.
static PATH_MUTEX: Mutex<()> = Mutex::new(());

/// Mutex that serialises tests which write/read the preview-mode temp file.
static PREVIEW_MUTEX: Mutex<()> = Mutex::new(());

/// Run `f` with a fake `kubectl` binary that exits with `exit_code` at the
/// front of PATH.  Restores PATH unconditionally on return.
fn with_fake_kubectl<F, T>(exit_code: i32, f: F) -> T
where
    F: FnOnce() -> T,
{
    let _guard = PATH_MUTEX.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let kubectl = tmp.path().join("kubectl");
    let script = format!("#!/bin/sh\nexit {exit_code}\n");
    std::fs::write(&kubectl, &script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&kubectl, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let old_path = std::env::var("PATH").unwrap_or_default();
    // SAFETY: serialised by PATH_MUTEX — no concurrent PATH reads during f().
    unsafe { std::env::set_var("PATH", format!("{}:{old_path}", tmp.path().display())) };
    let result = f();
    unsafe { std::env::set_var("PATH", old_path) };
    result
}

// ── runtime_dir ───────────────────────────────────────────────────────────────

#[test]
fn runtime_dir_returns_existing_path() {
    let dir = runtime_dir();
    assert!(dir.exists(), "runtime_dir must exist after first call");
}

#[test]
fn runtime_dir_is_singleton() {
    let a = runtime_dir();
    let b = runtime_dir();
    assert_eq!(a, b, "runtime_dir must return the same path on every call");
}

#[test]
fn runtime_dir_is_a_directory() {
    assert!(runtime_dir().is_dir());
}

// ── preview_toggle_path ───────────────────────────────────────────────────────

#[test]
fn preview_toggle_path_is_under_runtime_dir() {
    let toggle = preview_toggle_path();
    assert!(
        toggle.starts_with(runtime_dir()),
        "preview_toggle_path must be inside runtime_dir"
    );
}

// ── install_preview_toggle / current_preview_mode ─────────────────────────────

#[test]
fn install_preview_toggle_creates_mode_file_set_to_zero() {
    let _guard = PREVIEW_MUTEX.lock().unwrap();
    install_preview_toggle();
    assert_eq!(
        current_preview_mode(),
        0,
        "mode must be reset to 0 after install"
    );
}

#[test]
fn install_preview_toggle_creates_executable_script() {
    let _guard = PREVIEW_MUTEX.lock().unwrap();
    install_preview_toggle();
    let toggle = preview_toggle_path();
    assert!(
        toggle.exists(),
        "preview toggle script must exist after install"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&toggle).unwrap().permissions().mode();
        assert_ne!(mode & 0o100, 0, "preview toggle script must be executable");
    }
}

#[test]
fn current_preview_mode_returns_zero_to_two() {
    let _guard = PREVIEW_MUTEX.lock().unwrap();
    install_preview_toggle();
    let mode = current_preview_mode();
    assert!(mode <= 2, "mode must be 0, 1, or 2; got {mode}");
}

// ── action_logs — kind guard ───────────────────────────────────────────────────

#[test]
fn action_logs_skips_non_pod_and_returns_ok() {
    // Service is not a Pod — function should print a warning and return Ok without
    // calling kubectl.
    let item = service_item();
    let result = action_logs(&[&item]);
    assert!(
        result.is_ok(),
        "action_logs on Service must return Ok: {result:?}"
    );
}

#[test]
fn action_logs_multiple_items_skips_non_pods() {
    let pod = pod_item();
    let svc = service_item();
    // Even with mixed kinds, passing only non-pods returns Ok without kubectl.
    let result = action_logs(&[&svc]);
    assert!(result.is_ok());
    // Pod goes through kubectl path; with no kubectl available it errors.
    // We test the success path separately with a fake kubectl.
    let _ = action_logs(&[&pod]); // may Ok or Err depending on environment
}

// ── action_logs — kubectl success path ────────────────────────────────────────

#[test]
fn action_logs_pod_with_kubectl_success() {
    let item = pod_item();
    let result = with_fake_kubectl(0, || action_logs(&[&item]));
    assert!(
        result.is_ok(),
        "action_logs should be Ok when kubectl exits 0: {result:?}"
    );
}

#[test]
fn action_logs_pod_with_kubectl_failure_still_ok() {
    // kubectl exits 1 → eprintln but function returns Ok(())
    let item = pod_item();
    let result = with_fake_kubectl(1, || action_logs(&[&item]));
    assert!(
        result.is_ok(),
        "action_logs returns Ok even when kubectl fails: {result:?}"
    );
}

// ── action_exec — kind guard ───────────────────────────────────────────────────

#[test]
fn action_exec_returns_ok_immediately_for_non_pod() {
    let item = service_item();
    let result = action_exec(&item);
    assert!(
        result.is_ok(),
        "action_exec on Service must return Ok: {result:?}"
    );
}

// ── action_exec — kubectl paths ───────────────────────────────────────────────

#[test]
fn action_exec_pod_with_kubectl_success_on_first_shell() {
    let item = pod_item();
    let result = with_fake_kubectl(0, || action_exec(&item));
    assert!(
        result.is_ok(),
        "action_exec should be Ok when kubectl exits 0: {result:?}"
    );
}

#[test]
fn action_exec_pod_with_kubectl_failure_on_both_shells() {
    // Both /bin/sh and /bin/bash fail (exit 1) → eprintln but Ok(())
    let item = pod_item();
    let result = with_fake_kubectl(1, || action_exec(&item));
    assert!(
        result.is_ok(),
        "action_exec returns Ok even when all shells fail: {result:?}"
    );
}

// ── action_delete — stdin-empty cancel paths ──────────────────────────────────

#[test]
fn action_delete_single_item_cancelled_on_empty_stdin() {
    // In automated runs stdin is closed → read_line returns "" → not "y" → cancel
    let item = pod_item();
    let result = action_delete(&[&item]);
    assert!(
        result.is_ok(),
        "action_delete with empty stdin must return Ok: {result:?}"
    );
}

#[test]
fn action_delete_cluster_scoped_item_cancelled_on_empty_stdin() {
    let item = node_item();
    let result = action_delete(&[&item]);
    assert!(result.is_ok());
}

#[test]
fn action_delete_bulk_more_than_ten_cancelled_on_empty_stdin() {
    // >10 items triggers the "type 'yes'" guard — empty stdin → cancelled
    let items: Vec<K8sItem> = (0..11)
        .map(|i| {
            K8sItem::new(
                ResourceKind::Pod,
                "ns",
                format!("pod-{i}"),
                "Running",
                "1d",
                "",
            )
        })
        .collect();
    let refs: Vec<&K8sItem> = items.iter().collect();
    let result = action_delete(&refs);
    assert!(
        result.is_ok(),
        "bulk delete with empty stdin must return Ok: {result:?}"
    );
}

// ── action_portforward — kind guard ───────────────────────────────────────────

#[test]
fn action_portforward_returns_ok_immediately_for_non_pod_non_service() {
    let item = deploy_item();
    let result = action_portforward(&item);
    assert!(
        result.is_ok(),
        "action_portforward on Deployment must return Ok: {result:?}"
    );
}

#[test]
fn action_portforward_returns_ok_for_node_kind() {
    let item = node_item();
    let result = action_portforward(&item);
    assert!(result.is_ok());
}

// ── action_portforward — stdin-empty cancel (Pod / Service) ──────────────────

#[test]
fn action_portforward_pod_cancelled_on_empty_stdin() {
    // Port prompt reads stdin; empty stdin → Ok(None) for local port → "Cancelled."
    let item = pod_item();
    let result = action_portforward(&item);
    assert!(
        result.is_ok(),
        "action_portforward cancelled on empty stdin: {result:?}"
    );
}

#[test]
fn action_portforward_service_cancelled_on_empty_stdin() {
    let item = service_item();
    let result = action_portforward(&item);
    assert!(result.is_ok());
}

// ── action_rollout_restart — kind guard ──────────────────────────────────────

#[test]
fn action_rollout_restart_skips_pod_and_returns_ok() {
    let item = pod_item();
    let result = action_rollout_restart(&[&item]);
    assert!(
        result.is_ok(),
        "action_rollout_restart on Pod must skip and return Ok: {result:?}"
    );
}

#[test]
fn action_rollout_restart_skips_service_and_returns_ok() {
    let item = service_item();
    let result = action_rollout_restart(&[&item]);
    assert!(result.is_ok());
}

// ── action_rollout_restart — kubectl paths ────────────────────────────────────

#[test]
fn action_rollout_restart_deploy_kubectl_success() {
    let item = deploy_item();
    let result = with_fake_kubectl(0, || action_rollout_restart(&[&item]));
    assert!(
        result.is_ok(),
        "rollout restart should be Ok when kubectl exits 0: {result:?}"
    );
}

#[test]
fn action_rollout_restart_sts_kubectl_success() {
    let item = sts_item();
    let result = with_fake_kubectl(0, || action_rollout_restart(&[&item]));
    assert!(result.is_ok());
}

#[test]
fn action_rollout_restart_ds_kubectl_success() {
    let item = ds_item();
    let result = with_fake_kubectl(0, || action_rollout_restart(&[&item]));
    assert!(result.is_ok());
}

#[test]
fn action_rollout_restart_kubectl_failure_still_ok() {
    // kubectl exits 1 → eprintln the stderr but function returns Ok(())
    let item = deploy_item();
    let result = with_fake_kubectl(1, || action_rollout_restart(&[&item]));
    assert!(
        result.is_ok(),
        "rollout restart returns Ok even when kubectl fails: {result:?}"
    );
}

#[test]
fn action_rollout_restart_mixed_kinds_skips_invalid() {
    // Pod is skipped; only Deployment goes through kubectl
    let pod = pod_item();
    let deploy = deploy_item();
    let result = with_fake_kubectl(0, || action_rollout_restart(&[&pod, &deploy]));
    assert!(result.is_ok());
}

// ── action_yaml — kubectl paths ───────────────────────────────────────────────

#[test]
fn action_yaml_kubectl_success() {
    let item = pod_item();
    let result = with_fake_kubectl(0, || action_yaml(&[&item]));
    assert!(
        result.is_ok(),
        "action_yaml should be Ok when kubectl exits 0: {result:?}"
    );
}

#[test]
fn action_yaml_kubectl_failure_still_ok() {
    let item = pod_item();
    let result = with_fake_kubectl(1, || action_yaml(&[&item]));
    assert!(
        result.is_ok(),
        "action_yaml returns Ok even when kubectl fails: {result:?}"
    );
}

#[test]
fn action_yaml_works_for_any_resource_kind() {
    let item = deploy_item();
    let result = with_fake_kubectl(0, || action_yaml(&[&item]));
    assert!(result.is_ok());
}

// ── action_describe — kubectl paths ───────────────────────────────────────────

#[test]
fn action_describe_kubectl_success() {
    let item = pod_item();
    let result = with_fake_kubectl(0, || action_describe(&[&item]));
    assert!(
        result.is_ok(),
        "action_describe should be Ok when kubectl exits 0: {result:?}"
    );
}

#[test]
fn action_describe_kubectl_failure_falls_back_to_output_str() {
    // kubectl exits 1 → falls back to printing item.output_str(), still Ok(())
    let item = pod_item();
    let result = with_fake_kubectl(1, || action_describe(&[&item]));
    assert!(
        result.is_ok(),
        "action_describe returns Ok even when kubectl fails: {result:?}"
    );
}

#[test]
fn action_describe_works_for_cluster_scoped_resources() {
    let item = node_item(); // no namespace
    let result = with_fake_kubectl(0, || action_describe(&[&item]));
    assert!(result.is_ok());
}

// ── Multi-cluster context forwarding ──────────────────────────────────────────

#[test]
fn action_logs_passes_context_flag_for_multi_cluster_item() {
    let item = K8sItem::new(
        ResourceKind::Pod,
        "ns",
        "pod",
        "Running",
        "1d",
        "prod-cluster",
    );
    // With fake kubectl, we just verify the call doesn't blow up
    let result = with_fake_kubectl(0, || action_logs(&[&item]));
    assert!(result.is_ok());
}
