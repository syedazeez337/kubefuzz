//! Tests for kubefuzz::k8s::client — context persistence and kubeconfig reading.

use std::sync::Mutex;

use kubefuzz::k8s::client::{current_context, list_contexts, load_last_context, save_last_context};

/// Serialises tests that modify the last_context file so they don't race.
static CTX_MUTEX: Mutex<()> = Mutex::new(());

// ── current_context ───────────────────────────────────────────────────────────

#[test]
fn current_context_returns_non_empty_string() {
    // Without a kubeconfig the function returns "unknown"; with one it returns the context name.
    // Either way the result must be a non-empty string.
    let ctx = current_context();
    assert!(
        !ctx.is_empty(),
        "current_context must never return an empty string"
    );
}

// ── list_contexts ─────────────────────────────────────────────────────────────

#[test]
fn list_contexts_returns_sorted_list_or_empty() {
    let ctxs = list_contexts();
    if ctxs.len() > 1 {
        let mut sorted = ctxs.clone();
        sorted.sort();
        assert_eq!(
            ctxs, sorted,
            "list_contexts must return alphabetically sorted contexts"
        );
    }
}

#[test]
fn list_contexts_does_not_panic_without_kubeconfig() {
    // May return [] if no kubeconfig; must not panic.
    let _ = list_contexts();
}

// ── save_last_context / load_last_context ─────────────────────────────────────

#[test]
fn save_and_load_last_context_round_trip() {
    let _guard = CTX_MUTEX.lock().unwrap();

    // Preserve original so we can restore it afterward.
    let original = load_last_context();

    save_last_context("kubefuzz-test-context-xyz");
    let loaded = load_last_context();
    assert_eq!(
        loaded,
        Some("kubefuzz-test-context-xyz".to_string()),
        "load_last_context must return the value that was saved"
    );

    // Restore original state.
    match original {
        Some(ref ctx) => save_last_context(ctx),
        None => {
            // Delete the file so the state is back to "no saved context".
            if let Some(dir) = dirs::config_dir() {
                let _ = std::fs::remove_file(dir.join("kubefuzz").join("last_context"));
            }
        }
    }
}

#[test]
fn save_last_context_trims_whitespace_on_reload() {
    let _guard = CTX_MUTEX.lock().unwrap();
    let original = load_last_context();

    save_last_context("my-cluster");
    let loaded = load_last_context().unwrap_or_default();
    assert_eq!(
        loaded, "my-cluster",
        "loaded context must not have extra whitespace"
    );

    match original {
        Some(ref ctx) => save_last_context(ctx),
        None => {
            if let Some(dir) = dirs::config_dir() {
                let _ = std::fs::remove_file(dir.join("kubefuzz").join("last_context"));
            }
        }
    }
}

#[test]
fn load_last_context_does_not_panic_when_file_missing() {
    // Just verifies the function completes without panic in any filesystem state.
    let _ = load_last_context();
}

// ── build_client_for_context — error paths ────────────────────────────────────

#[tokio::test]
async fn build_client_invalid_kubeconfig_path_returns_error() {
    use kubefuzz::k8s::client::build_client_for_context;
    let result = build_client_for_context("any-ctx", Some("/nonexistent/kubeconfig.yaml")).await;
    assert!(
        result.is_err(),
        "build_client_for_context with nonexistent kubeconfig must return Err"
    );
}

#[tokio::test]
async fn build_client_nonexistent_context_returns_error() {
    use kubefuzz::k8s::client::build_client_for_context;
    // A context name that cannot exist in any kubeconfig.
    let result = build_client_for_context("kubefuzz-nonexistent-ctx-zzzz", None).await;
    assert!(
        result.is_err(),
        "build_client_for_context with unknown context must return Err"
    );
}
