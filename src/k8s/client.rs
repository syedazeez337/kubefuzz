use anyhow::{Context, Result};
use kube::{Client, config::KubeConfigOptions};

/// Build a kube::Client for a specific named kubeconfig context.
/// Reads kubeconfig from the default location ($KUBECONFIG or ~/.kube/config).
pub async fn build_client_for_context(context_name: &str) -> Result<Client> {
    let config = kube::Config::from_kubeconfig(&KubeConfigOptions {
        context: Some(context_name.to_string()),
        ..Default::default()
    })
    .await
    .with_context(|| format!("Failed to load kubeconfig context '{context_name}'"))?;

    Client::try_from(config).context("Failed to build Kubernetes client")
}

/// Return the current context name from kubeconfig (for display in the header).
pub fn current_context() -> String {
    kube::config::Kubeconfig::read()
        .ok()
        .and_then(|cfg| cfg.current_context)
        .unwrap_or_else(|| "unknown".to_string())
}

/// Return all context names from kubeconfig, sorted alphabetically.
pub fn list_contexts() -> Vec<String> {
    let mut ctxs: Vec<String> = kube::config::Kubeconfig::read()
        .ok()
        .map(|cfg| cfg.contexts.into_iter().map(|c| c.name).collect())
        .unwrap_or_default();
    ctxs.sort();
    ctxs
}

/// Persist the last-used context to ~/.config/kubefuzz/last_context.
pub fn save_last_context(context: &str) {
    if let Some(dir) = dirs::config_dir() {
        let dir = dir.join("kubefuzz");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("last_context"), context);
    }
}

/// Load the last-used context from ~/.config/kubefuzz/last_context.
pub fn load_last_context() -> Option<String> {
    dirs::config_dir()
        .map(|d| d.join("kubefuzz").join("last_context"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
