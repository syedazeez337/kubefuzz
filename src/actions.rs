//! Post-selection action handlers — every kubectl operation lives here.

use anyhow::Result;
use std::io::{self, Write};

use crate::items::{K8sItem, ResourceKind};

// ─── Preview mode (shared with items.rs via temp file) ────────────────────────

const PREVIEW_MODE_FILE: &str = "/tmp/kubefuzz-preview-mode";
pub const PREVIEW_TOGGLE_SCRIPT: &str = "/tmp/kubefuzz-preview-toggle";

/// Install the preview-toggle shell script and reset the mode to 0 (describe).
/// Called once at startup before skim opens.
pub fn install_preview_toggle() {
    let script = format!(
        "#!/bin/sh\n\
         n=$(cat {PREVIEW_MODE_FILE} 2>/dev/null || echo 0)\n\
         printf $(( (n + 1) % 3 )) > {PREVIEW_MODE_FILE}\n"
    );
    let _ = std::fs::write(PREVIEW_TOGGLE_SCRIPT, script);
    let _ = std::process::Command::new("chmod")
        .args(["+x", PREVIEW_TOGGLE_SCRIPT])
        .status();
    // Reset to describe mode whenever kubefuzz starts fresh
    let _ = std::fs::write(PREVIEW_MODE_FILE, "0");
}

/// Read the current preview mode (0 = describe, 1 = yaml, 2 = logs).
pub fn current_preview_mode() -> u8 {
    std::fs::read_to_string(PREVIEW_MODE_FILE)
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(0)
        % 3
}

/// Human-readable label for the current preview mode (shown in the header).
pub fn preview_mode_label() -> &'static str {
    match current_preview_mode() {
        0 => "describe",
        1 => "yaml",
        2 => "logs",
        _ => "describe",
    }
}

// ─── Logs ─────────────────────────────────────────────────────────────────────

pub fn action_logs(items: &[&K8sItem]) -> Result<()> {
    for item in items {
        if !matches!(item.kind, ResourceKind::Pod) {
            eprintln!(
                "[kubefuzz] logs only available for pods (got {})",
                item.kind.as_str()
            );
            continue;
        }
        println!(
            "\n─── logs: {}/{} ───",
            item.namespace, item.name
        );
        let mut args = vec!["logs", "--tail=200", &item.name];
        if !item.namespace.is_empty() {
            args.extend_from_slice(&["-n", &item.namespace]);
        }
        let status = std::process::Command::new("kubectl").args(&args).status()?;
        if !status.success() {
            eprintln!("[kubefuzz] kubectl logs exited with {status}");
        }
    }
    Ok(())
}

// ─── Exec ─────────────────────────────────────────────────────────────────────

pub fn action_exec(item: &K8sItem) -> Result<()> {
    if !matches!(item.kind, ResourceKind::Pod) {
        eprintln!("[kubefuzz] exec only available for pods");
        return Ok(());
    }
    println!("Dropping into shell: {}/{}", item.namespace, item.name);
    for shell in &["/bin/sh", "/bin/bash"] {
        let mut args = vec!["exec", "-it", &item.name];
        if !item.namespace.is_empty() {
            args.extend_from_slice(&["-n", &item.namespace]);
        }
        args.extend_from_slice(&["--", shell]);
        let status = std::process::Command::new("kubectl").args(&args).status()?;
        if status.success() {
            return Ok(());
        }
    }
    eprintln!("[kubefuzz] exec failed for {}", item.name);
    Ok(())
}

// ─── Delete ───────────────────────────────────────────────────────────────────

pub fn action_delete(items: &[&K8sItem]) -> Result<()> {
    let count = items.len();
    let noun = if count == 1 { "resource" } else { "resources" };

    // Show what will be deleted
    for item in items {
        let loc = if item.namespace.is_empty() {
            "(cluster-scoped)".to_string()
        } else {
            format!("ns/{}", item.namespace)
        };
        println!("  • {}/{} [{}]", item.kind.as_str(), item.name, loc);
    }

    print!("\nDelete {count} {noun}? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Cancelled.");
        return Ok(());
    }

    for item in items {
        let mut args = vec!["delete", item.kind.as_str(), &item.name];
        if !item.namespace.is_empty() {
            args.extend_from_slice(&["-n", &item.namespace]);
        }
        let out = std::process::Command::new("kubectl").args(&args).output()?;
        if out.status.success() {
            println!("✓ deleted {}/{}", item.kind.as_str(), item.name);
        } else {
            eprintln!(
                "✗ delete failed {}/{}: {}",
                item.kind.as_str(),
                item.name,
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
    }
    Ok(())
}

// ─── Port-forward ─────────────────────────────────────────────────────────────

pub fn action_portforward(item: &K8sItem) -> Result<()> {
    if !matches!(item.kind, ResourceKind::Pod | ResourceKind::Service) {
        eprintln!(
            "[kubefuzz] port-forward only works with pods and services (got {})",
            item.kind.as_str()
        );
        return Ok(());
    }

    print!("Local port: ");
    io::stdout().flush()?;
    let mut local = String::new();
    io::stdin().read_line(&mut local)?;
    let local = local.trim().to_string();
    if local.is_empty() {
        println!("Cancelled.");
        return Ok(());
    }

    print!("Remote port [{}]: ", local);
    io::stdout().flush()?;
    let mut remote = String::new();
    io::stdin().read_line(&mut remote)?;
    let remote = remote.trim().to_string();
    let remote = if remote.is_empty() { local.clone() } else { remote };

    let target = format!("{}/{}", item.kind.as_str(), item.name);
    let ports = format!("{local}:{remote}");

    let mut args = vec!["port-forward", &target, &ports];
    if !item.namespace.is_empty() {
        args.extend_from_slice(&["-n", &item.namespace]);
    }

    println!(
        "Forwarding localhost:{local} → {target} port {remote}  (Ctrl-C to stop)"
    );
    let status = std::process::Command::new("kubectl").args(&args).status()?;
    if !status.success() {
        eprintln!("[kubefuzz] port-forward exited with {status}");
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
        if !RESTARTABLE.contains(&item.kind) {
            eprintln!(
                "[kubefuzz] rollout restart only works with deploy/sts/ds (got {})",
                item.kind.as_str()
            );
            continue;
        }

        let target = format!("{}/{}", item.kind.as_str(), item.name);
        let mut restart_args = vec!["rollout", "restart", &target];
        if !item.namespace.is_empty() {
            restart_args.extend_from_slice(&["-n", &item.namespace]);
        }

        let out = std::process::Command::new("kubectl")
            .args(&restart_args)
            .output()?;
        if out.status.success() {
            println!("↺ restarting {target}");
            // Watch rollout status
            let mut status_args = vec!["rollout", "status", &target];
            if !item.namespace.is_empty() {
                status_args.extend_from_slice(&["-n", &item.namespace]);
            }
            std::process::Command::new("kubectl")
                .args(&status_args)
                .status()?;
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
        let mut args = vec!["get", item.kind.as_str(), &item.name, "-o", "yaml"];
        if !item.namespace.is_empty() {
            args.extend_from_slice(&["-n", &item.namespace]);
        }
        let out = std::process::Command::new("kubectl").args(&args).output()?;
        if out.status.success() {
            print!("{}", String::from_utf8_lossy(&out.stdout));
        } else {
            eprintln!(
                "[kubefuzz] kubectl get yaml failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
    }
    Ok(())
}

// ─── Describe (default Enter) ─────────────────────────────────────────────────

pub fn action_describe(items: &[&K8sItem]) -> Result<()> {
    for item in items {
        let mut args = vec!["describe", item.kind.as_str(), &item.name];
        if !item.namespace.is_empty() {
            args.extend_from_slice(&["-n", &item.namespace]);
        }
        let out = std::process::Command::new("kubectl").args(&args).output()?;
        if out.status.success() {
            print!("{}", String::from_utf8_lossy(&out.stdout));
        } else {
            // Fallback: just print the resource identifier
            println!("{}", item.output_str());
        }
    }
    Ok(())
}
