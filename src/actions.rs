//! Post-selection action handlers — every kubectl operation lives here.

use anyhow::Result;
use std::io::{self, Write};
use std::process::Command;

use crate::items::{K8sItem, ResourceKind};

// ─── Secure runtime directory ─────────────────────────────────────────────────

use std::path::PathBuf;
use std::sync::OnceLock;

/// Returns a secure, per-process runtime directory for temp files.
/// Uses `XDG_RUNTIME_DIR` (Linux) or a PID-scoped subdirectory of the system
/// temp dir as fallback. Permissions are set to 0o700 on Unix.
pub fn runtime_dir() -> &'static PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let base = dirs::runtime_dir()
            .or_else(|| std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from))
            .unwrap_or_else(std::env::temp_dir);
        let dir = base.join(format!("kuberift-{}", std::process::id()));
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("[kuberift] warning: cannot create runtime dir: {e}");
        }
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

pub fn preview_toggle_path() -> PathBuf {
    runtime_dir().join("preview-toggle")
}

// ─── Preview mode (shared with items.rs via temp file) ────────────────────────

/// Install the preview-toggle shell script and reset the mode to 0 (describe).
/// Called once at startup before skim opens.
pub fn install_preview_toggle() {
    let mode_path = preview_mode_path();
    let toggle_path = preview_toggle_path();
    let script = format!(
        "#!/bin/sh\n\
         n=$(cat \"{mode}\" 2>/dev/null || echo 0)\n\
         printf $(( (n + 1) % 3 )) > \"{mode}\"\n",
        mode = mode_path.display()
    );
    if let Err(e) = std::fs::write(&toggle_path, &script) {
        eprintln!("[kuberift] warning: cannot write preview toggle script: {e}");
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&toggle_path, std::fs::Permissions::from_mode(0o700));
    }
    if let Err(e) = std::fs::write(&mode_path, "0") {
        eprintln!("[kuberift] warning: cannot write preview mode file: {e}");
    }
}

/// Read the current preview mode (0 = describe, 1 = yaml, 2 = logs).
pub fn current_preview_mode() -> u8 {
    std::fs::read_to_string(preview_mode_path())
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(0)
        % 3
}

// ─── kubectl command builder ──────────────────────────────────────────────────

/// Build a `kubectl` command pre-loaded with `--context <ctx>` when the item
/// belongs to a non-default cluster (multi-cluster mode).
fn kubectl(item: &K8sItem) -> Command {
    let mut cmd = Command::new("kubectl");
    if !item.context().is_empty() {
        cmd.args(["--context", item.context()]);
    }
    cmd
}

// ─── Logs ─────────────────────────────────────────────────────────────────────

pub fn action_logs(items: &[&K8sItem]) -> Result<()> {
    for item in items {
        if !matches!(item.kind(), ResourceKind::Pod) {
            eprintln!(
                "[kuberift] logs only available for pods (got {})",
                item.kind().as_str()
            );
            continue;
        }
        println!("\n─── logs: {}/{} ───", item.namespace(), item.name());
        let mut args = vec!["logs", "--tail=200"];
        if !item.namespace().is_empty() {
            args.extend_from_slice(&["-n", item.namespace()]);
        }
        args.extend_from_slice(&["--", item.name()]);
        let status = kubectl(item).args(&args).status()?;
        if !status.success() {
            eprintln!("[kuberift] kubectl logs exited with {status}");
        }
    }
    Ok(())
}

// ─── Exec ─────────────────────────────────────────────────────────────────────

pub fn action_exec(item: &K8sItem) -> Result<()> {
    if !matches!(item.kind(), ResourceKind::Pod) {
        eprintln!("[kuberift] exec only available for pods");
        return Ok(());
    }
    println!("Dropping into shell: {}/{}", item.namespace(), item.name());
    for shell in &["/bin/sh", "/bin/bash"] {
        let mut args = vec!["exec", "-it", item.name()];
        if !item.namespace().is_empty() {
            args.extend_from_slice(&["-n", item.namespace()]);
        }
        args.extend_from_slice(&["--", shell]);
        let status = kubectl(item).args(&args).status()?;
        if status.success() {
            return Ok(());
        }
    }
    eprintln!("[kuberift] exec failed for {}", item.name());
    Ok(())
}

// ─── Delete ───────────────────────────────────────────────────────────────────

pub fn action_delete(items: &[&K8sItem]) -> Result<()> {
    let count = items.len();
    let noun = if count == 1 { "resource" } else { "resources" };

    // Show what will be deleted
    for item in items {
        let loc = if item.namespace().is_empty() {
            "(cluster-scoped)".to_string()
        } else {
            format!("ns/{}", item.namespace())
        };
        let ctx_suffix = if item.context().is_empty() {
            String::new()
        } else {
            format!(" @{}", item.context())
        };
        println!(
            "  • {}/{} [{}]{}",
            item.kind().as_str(),
            item.name(),
            loc,
            ctx_suffix
        );
    }

    if count > 10 {
        eprintln!("[kuberift] ⚠ WARNING: You are about to delete {count} resources.");
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

    for item in items {
        let mut args = vec!["delete", item.kind().as_str()];
        if !item.namespace().is_empty() {
            args.extend_from_slice(&["-n", item.namespace()]);
        }
        args.extend_from_slice(&["--", item.name()]);
        let out = kubectl(item).args(&args).output()?;
        if out.status.success() {
            println!("✓ deleted {}/{}", item.kind().as_str(), item.name());
        } else {
            eprintln!(
                "✗ delete failed {}/{}: {}",
                item.kind().as_str(),
                item.name(),
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
    }
    Ok(())
}

// ─── Port-forward ─────────────────────────────────────────────────────────────

fn read_port(prompt: &str, default: Option<u16>) -> Result<Option<u16>> {
    match default {
        Some(d) => print!("{prompt} [{d}]: "),
        None => print!("{prompt}: "),
    }
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if input.is_empty() {
        return Ok(default);
    }
    let port: u16 = input
        .parse()
        .map_err(|_| anyhow::anyhow!("'{input}' is not a valid port number (1–65535)"))?;
    if port == 0 {
        anyhow::bail!("Port 0 is not valid");
    }
    if port < 1024 {
        eprintln!("[kuberift] warning: port {port} is privileged (may require root/admin)");
    }
    Ok(Some(port))
}

pub fn action_portforward(item: &K8sItem) -> Result<()> {
    if !matches!(item.kind(), ResourceKind::Pod | ResourceKind::Service) {
        eprintln!(
            "[kuberift] port-forward only works with pods and services (got {})",
            item.kind().as_str()
        );
        return Ok(());
    }

    let Some(local) = read_port("Local port", None)? else {
        println!("Cancelled.");
        return Ok(());
    };
    let remote = read_port("Remote port", Some(local))?.unwrap_or(local);

    let local = local.to_string();
    let remote = remote.to_string();

    let target = format!("{}/{}", item.kind().as_str(), item.name());
    let ports = format!("{local}:{remote}");

    let mut args = vec!["port-forward", &target, &ports];
    if !item.namespace().is_empty() {
        args.extend_from_slice(&["-n", item.namespace()]);
    }

    println!("Forwarding localhost:{local} → {target} port {remote}  (Ctrl-C to stop)");
    let status = kubectl(item).args(&args).status()?;
    if !status.success() {
        eprintln!("[kuberift] port-forward exited with {status}");
    }
    Ok(())
}

// ─── Rollout restart ──────────────────────────────────────────────────────────

pub fn action_rollout_restart(items: &[&K8sItem]) -> Result<()> {
    const RESTARTABLE: &[ResourceKind] = &[
        ResourceKind::Deployment,
        ResourceKind::StatefulSet,
        ResourceKind::DaemonSet,
    ];

    for item in items {
        if !RESTARTABLE.contains(&item.kind()) {
            eprintln!(
                "[kuberift] rollout restart only works with deploy/sts/ds (got {})",
                item.kind().as_str()
            );
            continue;
        }

        let target = format!("{}/{}", item.kind().as_str(), item.name());
        let mut restart_args = vec!["rollout", "restart", &target];
        if !item.namespace().is_empty() {
            restart_args.extend_from_slice(&["-n", item.namespace()]);
        }

        let out = kubectl(item).args(&restart_args).output()?;
        if out.status.success() {
            println!("↺ restarting {target}");
            let mut status_args = vec!["rollout", "status", &target];
            if !item.namespace().is_empty() {
                status_args.extend_from_slice(&["-n", item.namespace()]);
            }
            kubectl(item).args(&status_args).status()?;
        } else {
            eprintln!(
                "✗ rollout restart failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
    }
    Ok(())
}

// ─── Print YAML ───────────────────────────────────────────────────────────────

pub fn action_yaml(items: &[&K8sItem]) -> Result<()> {
    for item in items {
        let mut args = vec!["get", item.kind().as_str(), "-o", "yaml"];
        if !item.namespace().is_empty() {
            args.extend_from_slice(&["-n", item.namespace()]);
        }
        args.extend_from_slice(&["--", item.name()]);
        let out = kubectl(item).args(&args).output()?;
        if out.status.success() {
            print!("{}", String::from_utf8_lossy(&out.stdout));
        } else {
            eprintln!(
                "[kuberift] kubectl get yaml failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
    }
    Ok(())
}

// ─── Describe (default Enter) ─────────────────────────────────────────────────

pub fn action_describe(items: &[&K8sItem]) -> Result<()> {
    for item in items {
        let mut args = vec!["describe", item.kind().as_str()];
        if !item.namespace().is_empty() {
            args.extend_from_slice(&["-n", item.namespace()]);
        }
        args.extend_from_slice(&["--", item.name()]);
        let out = kubectl(item).args(&args).output()?;
        if out.status.success() {
            print!("{}", String::from_utf8_lossy(&out.stdout));
        } else {
            println!("{}", item.output_str());
        }
    }
    Ok(())
}
