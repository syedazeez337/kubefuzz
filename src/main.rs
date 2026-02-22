mod items;
mod k8s;

use anyhow::Result;
use items::{K8sItem, ResourceKind};
use k8s::{client::build_client, client::current_context, pods::stream_pods};
use skim::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = current_context();

    let options = SkimOptionsBuilder::default()
        .multi(true)
        .preview(String::new())
        .preview_window("right:50%")
        .height("60%")
        .header(format!(
            "KubeFuzz  ctx:{ctx}  \
             <tab> multi-select  <enter> describe  \
             ctrl-l logs  ctrl-e exec  ctrl-d delete  esc quit"
        ))
        .prompt("❯ ")
        // ctrl-l/e/d cause skim to accept the selection so we can handle the action
        .bind(
            ["ctrl-l:accept", "ctrl-e:accept", "ctrl-d:accept"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        )
        .build()?;

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    // Try to connect to the cluster. On failure, fall back to demo mode.
    match build_client().await {
        Ok(client) => {
            // Spawn async task to fetch pods — sends items to skim as they arrive
            let tx_k8s = tx.clone();
            tokio::spawn(async move {
                if let Err(e) = stream_pods(client, tx_k8s).await {
                    eprintln!("\n[kubefuzz] {e}");
                }
            });
        }
        Err(e) => {
            eprintln!("[kubefuzz] No cluster: {e}. Showing demo data.");
            // Fall back to fake data so the TUI still opens
            let demo = demo_items();
            let _ = tx.send(demo);
        }
    }

    drop(tx); // signal end-of-stream so skim knows input is done

    // Skim blocks here — it uses tokio::task::block_in_place internally
    let output = Skim::run_with(options, Some(rx)).map_err(|e| anyhow::anyhow!("{e}"))?;

    if output.is_abort {
        return Ok(());
    }

    handle_action(output).await
}

/// Dispatch the selected items + key binding to the right action.
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

    // Detect which key ended the session
    let key = output.final_key;
    let is_ctrl = |c: char| {
        key.code == KeyCode::Char(c) && key.modifiers == KeyModifiers::CONTROL
    };

    if is_ctrl('l') {
        // ctrl-l → stream logs for selected pod(s)
        for item in items {
            if matches!(item.kind, ResourceKind::Pod) {
                action_logs(item)?;
            } else {
                eprintln!("[kubefuzz] logs only available for pods");
            }
        }
    } else if is_ctrl('e') {
        // ctrl-e → exec into first selected pod
        if let Some(item) = items.first() {
            if matches!(item.kind, ResourceKind::Pod) {
                action_exec(item)?;
            }
        }
    } else if is_ctrl('d') {
        // ctrl-d → delete selected resources
        for item in &items {
            action_delete(item)?;
        }
    } else {
        // Enter / default → print selected resource identifiers to stdout
        for item in items {
            println!("{}", item.output_str());
        }
    }

    Ok(())
}

/// Stream logs for a pod to stdout (replaces the terminal after skim exits).
fn action_logs(item: &K8sItem) -> Result<()> {
    let mut args = vec!["logs", "--tail=100", &item.name];
    if !item.namespace.is_empty() {
        args.extend_from_slice(&["-n", &item.namespace]);
    }
    let status = std::process::Command::new("kubectl")
        .args(&args)
        .status()?;
    if !status.success() {
        eprintln!("[kubefuzz] kubectl logs exited with status {status}");
    }
    Ok(())
}

/// Open an interactive shell in a pod container.
fn action_exec(item: &K8sItem) -> Result<()> {
    let mut args = vec!["exec", "-it", &item.name];
    if !item.namespace.is_empty() {
        args.extend_from_slice(&["-n", &item.namespace]);
    }
    args.extend_from_slice(&["--", "/bin/sh"]);
    let status = std::process::Command::new("kubectl")
        .args(&args)
        .status()?;
    if !status.success() {
        // Try bash if sh fails
        let mut args2 = vec!["exec", "-it", &item.name];
        if !item.namespace.is_empty() {
            args2.extend_from_slice(&["-n", &item.namespace]);
        }
        args2.extend_from_slice(&["--", "/bin/bash"]);
        std::process::Command::new("kubectl").args(&args2).status()?;
    }
    Ok(())
}

/// Delete a resource with a confirmation prompt.
fn action_delete(item: &K8sItem) -> Result<()> {
    use std::io::{self, Write};
    print!(
        "Delete {}/{} in namespace '{}'? [y/N] ",
        item.kind.as_str(),
        item.name,
        item.namespace
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        let mut args = vec!["delete", item.kind.as_str(), &item.name];
        if !item.namespace.is_empty() {
            args.extend_from_slice(&["-n", &item.namespace]);
        }
        let output = std::process::Command::new("kubectl").args(&args).output()?;
        if output.status.success() {
            println!("Deleted {}/{}", item.kind.as_str(), item.name);
        } else {
            eprintln!(
                "[kubefuzz] delete failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    } else {
        println!("Cancelled.");
    }
    Ok(())
}

/// Hardcoded demo items shown when no cluster is available.
fn demo_items() -> Vec<Arc<dyn SkimItem>> {
    vec![
        Arc::new(K8sItem::new(ResourceKind::Pod, "production", "api-server-7d9f8b6c5-xk2lp", "Running", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "production", "worker-6f8b9c4d7-mn3qr", "Running", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "production", "cache-3a5b6c7d8-ij3kl", "CrashLoopBackOff", "1h")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "staging", "frontend-5c7d8e9f0-ab1cd", "Pending", "5m")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "staging", "db-migrator-4b6c7d8e9-ef2gh", "Failed", "10m")),
        Arc::new(K8sItem::new(ResourceKind::Service, "production", "api-service", "ClusterIP", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Deployment, "production", "api-server", "3/3", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Deployment, "staging", "frontend", "0/1", "5m")),
        Arc::new(K8sItem::new(ResourceKind::Namespace, "", "production", "Active", "30d")),
        Arc::new(K8sItem::new(ResourceKind::Namespace, "", "staging", "Active", "10d")),
    ]
}
