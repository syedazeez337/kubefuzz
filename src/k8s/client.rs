use anyhow::{Context, Result};
use kube::{config::KubeConfigOptions, Client};

/// Build a `kube::Client` for a specific named kubeconfig context.
/// If `kubeconfig` is Some, reads from that file; otherwise uses the default
/// ($KUBECONFIG or ~/.kube/config).
pub async fn build_client_for_context(
    context_name: &str,
    kubeconfig: Option<&str>,
) -> Result<Client> {
    let options = KubeConfigOptions {
        context: Some(context_name.to_string()),
        ..Default::default()
    };
    let config = match kubeconfig {
        Some(path) => {
            let kc = kube::config::Kubeconfig::read_from(path)
                .with_context(|| format!("Failed to read kubeconfig from '{path}'"))?;
            kube::Config::from_custom_kubeconfig(kc, &options)
                .await
                .with_context(|| format!("Failed to load context '{context_name}' from '{path}'"))?
        }
        None => kube::Config::from_kubeconfig(&options)
            .await
            .with_context(|| format!("Failed to load kubeconfig context '{context_name}'"))?,
    };
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

/// Persist the last-used context to `~/.config/kuberift/last_context`.
/// Sets 0o700 on the directory and 0o600 on the file on Unix.
pub fn save_last_context(context: &str) {
    if let Some(dir) = dirs::config_dir() {
        let dir = dir.join("kuberift");
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("[kuberift] warning: cannot create config dir: {e}");
            return;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
        }
        let path = dir.join("last_context");
        if let Err(e) = std::fs::write(&path, context) {
            eprintln!("[kuberift] warning: cannot save context: {e}");
            return;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
    }
}

/// Load the last-used context from `~/.config/kuberift/last_context`.
pub fn load_last_context() -> Option<String> {
    dirs::config_dir()
        .map(|d| d.join("kuberift").join("last_context"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
