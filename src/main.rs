mod cli;
mod items;
mod k8s;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use items::{K8sItem, ResourceKind};
use k8s::{
    client::{build_client, current_context},
    resources::{stream_resources, ALL_KINDS},
};
use skim::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let ctx = current_context();

    // Determine which resource kinds to stream
    let kinds: Vec<ResourceKind> = args
        .resource_filter()
        .unwrap_or_else(|| ALL_KINDS.to_vec());

    let kind_label = if kinds.len() == 1 {
        kinds[0].as_str().to_string()
    } else {
        "all".to_string()
    };

    let options = SkimOptionsBuilder::default()
        .multi(true)
        .preview(String::new())
        .preview_window("right:50%")
        .height("60%")
        .header(format!(
            "KubeFuzz  ctx:{ctx}  res:{kind_label}  \
             <tab> multi-select  <enter> describe  \
             ctrl-l logs  ctrl-e exec  ctrl-d delete  esc quit"
        ))
        .prompt("❯ ")
        .bind(
            ["ctrl-l:accept", "ctrl-e:accept", "ctrl-d:accept"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        )
        .build()?;

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    match build_client().await {
        Ok(client) => {
            let tx_k8s = tx.clone();
            let kinds_clone = kinds.clone();
            tokio::spawn(async move {
                if let Err(e) = stream_resources(client, tx_k8s, &kinds_clone).await {
                    eprintln!("\n[kubefuzz] {e}");
                }
                // tx_k8s drops here — signals end of stream
            });
        }
        Err(e) => {
            eprintln!("[kubefuzz] No cluster: {e}. Showing demo data.");
            let _ = tx.send(demo_items());
        }
    }

    drop(tx); // our copy drops; skim sees EOF when the spawned task's copy also drops

    let output = Skim::run_with(options, Some(rx)).map_err(|e| anyhow::anyhow!("{e}"))?;

    if output.is_abort {
        return Ok(());
    }

    handle_action(output).await
}

// ─── Action dispatch ─────────────────────────────────────────────────────────

async fn handle_action(output: SkimOutput) -> Result<()> {
    use crossterm::event::{KeyCode, KeyModifiers};

    let items: Vec<&K8sItem> = output
        .selected_items
        .iter()
        .filter_map(|i| (**i).as_any().downcast_ref::<K8sItem>())
        .collect();

    if items.is_empty() {
        return Ok(());
    }

    let key = output.final_key;
    let is_ctrl = |c: char| {
        key.code == KeyCode::Char(c) && key.modifiers == KeyModifiers::CONTROL
    };

    if is_ctrl('l') {
        for item in items {
            if matches!(item.kind, ResourceKind::Pod) {
                action_logs(item)?;
            } else {
                eprintln!("[kubefuzz] ctrl-l logs only available for pods, got {}", item.kind.as_str());
            }
        }
    } else if is_ctrl('e') {
        if let Some(item) = items.first() {
            if matches!(item.kind, ResourceKind::Pod) {
                action_exec(item)?;
            } else {
                eprintln!("[kubefuzz] ctrl-e exec only available for pods");
            }
        }
    } else if is_ctrl('d') {
        for item in &items {
            action_delete(item)?;
        }
    } else {
        // Enter / default — print selected resource identifiers
        for item in items {
            println!("{}", item.output_str());
        }
    }

    Ok(())
}

// ─── kubectl wrappers ─────────────────────────────────────────────────────────

fn action_logs(item: &K8sItem) -> Result<()> {
    let mut args = vec!["logs", "--tail=100", &item.name];
    if !item.namespace.is_empty() {
        args.extend_from_slice(&["-n", &item.namespace]);
    }
    let status = std::process::Command::new("kubectl").args(&args).status()?;
    if !status.success() {
        eprintln!("[kubefuzz] kubectl logs exited with {status}");
    }
    Ok(())
}

fn action_exec(item: &K8sItem) -> Result<()> {
    // Try /bin/sh, fall back to /bin/bash
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
    Ok(())
}

fn action_delete(item: &K8sItem) -> Result<()> {
    use std::io::{self, Write};
    print!(
        "Delete {}/{} in '{}' ? [y/N] ",
        item.kind.as_str(),
        item.name,
        if item.namespace.is_empty() { "cluster" } else { &item.namespace }
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        let mut args = vec!["delete", item.kind.as_str(), &item.name];
        if !item.namespace.is_empty() {
            args.extend_from_slice(&["-n", &item.namespace]);
        }
        let out = std::process::Command::new("kubectl").args(&args).output()?;
        if out.status.success() {
            println!("Deleted {}/{}", item.kind.as_str(), item.name);
        } else {
            eprintln!(
                "[kubefuzz] delete failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    } else {
        println!("Cancelled.");
    }
    Ok(())
}

// ─── Demo data (no cluster) ───────────────────────────────────────────────────

fn demo_items() -> Vec<Arc<dyn SkimItem>> {
    vec![
        Arc::new(K8sItem::new(ResourceKind::Pod, "production", "api-server-7d9f8b6c5-xk2lp", "CrashLoopBackOff", "1h")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "staging", "frontend-5c7d8e9f0-ab1cd", "Pending", "5m")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "production", "worker-6f8b9c4d7-mn3qr", "Running", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Deployment, "production", "api-server", "2/3", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Deployment, "staging", "frontend", "0/1", "5m")),
        Arc::new(K8sItem::new(ResourceKind::Service, "production", "api-service", "ClusterIP", "2d")),
        Arc::new(K8sItem::new(ResourceKind::ConfigMap, "production", "app-config", "ConfigMap", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Secret, "production", "api-tls", "kubernetes.io/tls", "30d")),
        Arc::new(K8sItem::new(ResourceKind::Node, "", "kind-control-plane", "Ready", "7d")),
        Arc::new(K8sItem::new(ResourceKind::Namespace, "", "production", "Active", "30d")),
        Arc::new(K8sItem::new(ResourceKind::Namespace, "", "staging", "Active", "10d")),
    ]
}
