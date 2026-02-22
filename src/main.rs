mod actions;
mod cli;
mod items;
mod k8s;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use items::{K8sItem, ResourceKind};
use k8s::{
    client::{build_client, current_context},
    resources::{watch_resources, ALL_KINDS},
};
use skim::prelude::*;
use std::sync::Arc;

use actions::{
    action_delete, action_describe, action_exec, action_logs, action_portforward,
    action_rollout_restart, action_yaml, install_preview_toggle, preview_mode_label,
    PREVIEW_TOGGLE_SCRIPT,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let ctx = current_context();

    // Write the preview-toggle shell script and reset mode to 0 (describe)
    install_preview_toggle();

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
            "KubeFuzz  ctx:{ctx}  res:{kind_label}  preview:{}\n\
             <tab> select  <enter> describe  ctrl-l logs  ctrl-e exec  \
             ctrl-d delete  ctrl-f forward  ctrl-r restart  ctrl-y yaml  \
             ctrl-p preview-mode",
            preview_mode_label()
        ))
        .prompt("❯ ")
        .bind(
            [
                "ctrl-l:accept",
                "ctrl-e:accept",
                "ctrl-d:accept",
                "ctrl-f:accept",
                "ctrl-r:accept",
                "ctrl-y:accept",
                // ctrl-p cycles preview mode in-place (does NOT exit skim)
                &format!("ctrl-p:execute-silent({PREVIEW_TOGGLE_SCRIPT})+refresh-preview"),
            ]
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
                if let Err(e) = watch_resources(client, tx_k8s, &kinds_clone).await {
                    eprintln!("\n[kubefuzz] {e}");
                }
            });
        }
        Err(e) => {
            eprintln!("[kubefuzz] No cluster: {e}. Showing demo data.");
            let _ = tx.send(demo_items());
        }
    }

    drop(tx);

    let output = Skim::run_with(options, Some(rx)).map_err(|e| anyhow::anyhow!("{e}"))?;

    if output.is_abort {
        return Ok(());
    }

    dispatch(output).await
}

// ─── Action dispatch ─────────────────────────────────────────────────────────

async fn dispatch(output: SkimOutput) -> Result<()> {
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
    let ctrl = |c: char| key.code == KeyCode::Char(c) && key.modifiers == KeyModifiers::CONTROL;

    if ctrl('l') {
        action_logs(&items)?;
    } else if ctrl('e') {
        if let Some(item) = items.first() {
            action_exec(item)?;
        }
    } else if ctrl('d') {
        action_delete(&items)?;
    } else if ctrl('f') {
        if let Some(item) = items.first() {
            action_portforward(item)?;
        }
    } else if ctrl('r') {
        action_rollout_restart(&items)?;
    } else if ctrl('y') {
        action_yaml(&items)?;
    } else {
        // Enter — describe selected resources
        action_describe(&items)?;
    }

    Ok(())
}

// ─── Demo data (no cluster) ───────────────────────────────────────────────────

fn demo_items() -> Vec<Arc<dyn SkimItem>> {
    vec![
        Arc::new(K8sItem::new(ResourceKind::Pod,        "production", "api-server-7d9f8b6c5-xk2lp", "CrashLoopBackOff", "1h")),
        Arc::new(K8sItem::new(ResourceKind::Pod,        "staging",    "frontend-5c7d8e9f0-ab1cd",   "Pending",          "5m")),
        Arc::new(K8sItem::new(ResourceKind::Pod,        "production", "worker-6f8b9c4d7-mn3qr",     "Running",          "2d")),
        Arc::new(K8sItem::new(ResourceKind::Deployment, "production", "api-server",                 "2/3",              "2d")),
        Arc::new(K8sItem::new(ResourceKind::Deployment, "staging",    "frontend",                   "0/1",              "5m")),
        Arc::new(K8sItem::new(ResourceKind::Service,    "production", "api-service",                "ClusterIP",        "2d")),
        Arc::new(K8sItem::new(ResourceKind::ConfigMap,  "production", "app-config",                 "ConfigMap",        "2d")),
        Arc::new(K8sItem::new(ResourceKind::Secret,     "production", "api-tls",                    "kubernetes.io/tls","30d")),
        Arc::new(K8sItem::new(ResourceKind::Node,       "",           "kind-control-plane",         "Ready",            "7d")),
        Arc::new(K8sItem::new(ResourceKind::Namespace,  "",           "production",                 "Active",           "30d")),
        Arc::new(K8sItem::new(ResourceKind::Namespace,  "",           "staging",                    "Active",           "10d")),
    ]
}
