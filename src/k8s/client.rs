use anyhow::{Context, Result};
use kube::Client;

/// Build a kube::Client from the default kubeconfig (~/.kube/config or $KUBECONFIG).
/// Returns a descriptive error if no kubeconfig is found or the API is unreachable.
pub async fn build_client() -> Result<Client> {
    Client::try_default()
        .await
        .context("Failed to build Kubernetes client.\n\
                  Is kubectl configured? Check ~/.kube/config or $KUBECONFIG.")
}

/// Return the current context name from kubeconfig (for display in the header).
pub fn current_context() -> String {
    kube::config::Kubeconfig::read()
        .ok()
        .and_then(|cfg| cfg.current_context)
        .unwrap_or_else(|| "unknown".to_string())
}
