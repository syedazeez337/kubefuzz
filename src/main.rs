mod actions;
mod cli;
mod items;
mod k8s;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use items::{K8sItem, ResourceKind};
use k8s::{
    client::{
        build_client_for_context, current_context, list_contexts, load_last_context,
        save_last_context,
    },
    resources::{watch_resources, ALL_KINDS},
};
use skim::prelude::*;
use std::{borrow::Cow, sync::Arc};

use actions::{
    action_delete, action_describe, action_exec, action_logs, action_portforward,
    action_rollout_restart, action_yaml, install_preview_toggle, preview_mode_label,
    PREVIEW_TOGGLE_SCRIPT,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

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

    if args.all_contexts {
        // ── Multi-cluster mode: load all contexts simultaneously ──────────────
        run_all_contexts(&args, &kinds, &kind_label).await
    } else {
        // ── Single-cluster mode: loop to support ctrl-x context switching ─────
        run_single_context(&args, &kinds, &kind_label).await
    }
}

// ─── Single-cluster mode (with ctrl-x context switching) ─────────────────────

async fn run_single_context(args: &Args, kinds: &[ResourceKind], kind_label: &str) -> Result<()> {
    // Determine the starting context: CLI flag → last saved → kubeconfig current
    let mut active_ctx = args
        .context
        .clone()
        .or_else(load_last_context)
        .unwrap_or_else(current_context);

    loop {
        let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

        // Spawn watcher for the active context
        let ctx_for_watcher = active_ctx.clone();
        let tx_k8s = tx.clone();
        let kinds_clone = kinds.to_vec();
        tokio::spawn(async move {
            match build_client_for_context(&ctx_for_watcher).await {
                Ok(client) => {
                    if let Err(e) =
                        watch_resources(client, tx_k8s, &kinds_clone, "").await
                    {
                        eprintln!("\n[kubefuzz] {e}");
                    }
                }
                Err(e) => {
                    eprintln!("[kubefuzz] No cluster ({e}). Showing demo data.");
                    let _ = tx_k8s.send(demo_items());
                }
            }
        });

        drop(tx);

        let options = build_skim_options(&active_ctx, kind_label, true);
        let output = Skim::run_with(options, Some(rx)).map_err(|e| anyhow::anyhow!("{e}"))?;

        if output.is_abort {
            break;
        }

        // Check if ctrl-x was pressed → open context picker and restart loop
        use crossterm::event::{KeyCode, KeyModifiers};
        let key = output.final_key;
        if key.code == KeyCode::Char('x') && key.modifiers == KeyModifiers::CONTROL {
            if let Some(new_ctx) = pick_context()? {
                active_ctx = new_ctx;
                save_last_context(&active_ctx);
            }
            // Reinstall preview toggle so the new session starts fresh
            install_preview_toggle();
            continue;
        }

        dispatch(output).await?;
        break;
    }

    Ok(())
}

// ─── Multi-cluster mode (--all-contexts) ─────────────────────────────────────

async fn run_all_contexts(_args: &Args, kinds: &[ResourceKind], kind_label: &str) -> Result<()> {
    let contexts = list_contexts();
    if contexts.is_empty() {
        eprintln!("[kubefuzz] No contexts found in kubeconfig.");
        return Ok(());
    }

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    for ctx_name in &contexts {
        let tx_clone = tx.clone();
        let ctx_clone = ctx_name.clone();
        let kinds_clone = kinds.to_vec();

        tokio::spawn(async move {
            match build_client_for_context(&ctx_clone).await {
                Ok(client) => {
                    if let Err(e) =
                        watch_resources(client, tx_clone, &kinds_clone, &ctx_clone).await
                    {
                        eprintln!("[kubefuzz:{}] {e}", ctx_clone);
                    }
                }
                Err(e) => {
                    eprintln!("[kubefuzz] Cannot connect to '{}': {e}", ctx_clone);
                }
            }
        });
    }

    drop(tx);

    let ctx_label = "all-contexts";
    let options = build_skim_options(ctx_label, kind_label, false);
    let output = Skim::run_with(options, Some(rx)).map_err(|e| anyhow::anyhow!("{e}"))?;

    if output.is_abort {
        return Ok(());
    }

    // Ignore ctrl-x in all-contexts mode (no loop needed)
    dispatch(output).await
}

// ─── Context picker (ctrl-x) ──────────────────────────────────────────────────

/// Runs a secondary skim session showing all kubeconfig contexts.
/// Returns the selected context name, or None if the user cancelled.
fn pick_context() -> Result<Option<String>> {
    let contexts = list_contexts();
    if contexts.is_empty() {
        eprintln!("[kubefuzz] No contexts found in kubeconfig.");
        return Ok(None);
    }

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
    for ctx in &contexts {
        let _ = tx.send(vec![Arc::new(ContextItem(ctx.clone())) as Arc<dyn SkimItem>]);
    }
    drop(tx);

    let options = SkimOptionsBuilder::default()
        .header("Select context  (Esc to cancel)")
        .prompt("context ❯ ")
        .height("40%")
        .build()?;

    let output = Skim::run_with(options, Some(rx)).map_err(|e| anyhow::anyhow!("{e}"))?;

    if output.is_abort || output.selected_items.is_empty() {
        return Ok(None);
    }

    Ok(Some(output.selected_items[0].output().to_string()))
}

/// Minimal SkimItem wrapper for a kubeconfig context name string.
struct ContextItem(String);

impl SkimItem for ContextItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.0)
    }
    fn output(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.0)
    }
}

// ─── Shared skim options builder ──────────────────────────────────────────────

fn build_skim_options(ctx_label: &str, kind_label: &str, show_ctx_switch: bool) -> SkimOptions {
    let ctx_hint = if show_ctx_switch {
        "  ctrl-x switch-ctx"
    } else {
        ""
    };

    SkimOptionsBuilder::default()
        .multi(true)
        .preview(String::new())
        .preview_window("right:50%")
        .height("60%")
        .header(format!(
            "KubeFuzz  ctx:{ctx_label}  res:{kind_label}  preview:{}\n\
             <tab> select  <enter> describe  ctrl-l logs  ctrl-e exec  \
             ctrl-d delete  ctrl-f forward  ctrl-r restart  ctrl-y yaml  \
             ctrl-p preview-mode{ctx_hint}",
            preview_mode_label()
        ))
        .prompt("❯ ")
        .bind(
            {
                let mut binds = vec![
                    "ctrl-l:accept".to_string(),
                    "ctrl-e:accept".to_string(),
                    "ctrl-d:accept".to_string(),
                    "ctrl-f:accept".to_string(),
                    "ctrl-r:accept".to_string(),
                    "ctrl-y:accept".to_string(),
                    // ctrl-p cycles preview mode in-place (does NOT exit skim)
                    format!("ctrl-p:execute({PREVIEW_TOGGLE_SCRIPT})+refresh-preview"),
                ];
                if show_ctx_switch {
                    binds.push("ctrl-x:accept".to_string());
                }
                binds
            },
        )
        .build()
        .expect("SkimOptionsBuilder failed")
}

// ─── Action dispatch ─────────────────────────────────────────────────────────

async fn dispatch(output: SkimOutput) -> Result<()> {
    use crossterm::event::{KeyCode, KeyModifiers};

    // selected_items is Vec<Arc<MatchedItem>>; MatchedItem.item is Arc<dyn SkimItem>.
    //
    // IMPORTANT: do NOT call matched.item.as_any() — Arc<dyn SkimItem> is itself 'static
    // so AsAny is implemented directly on it, returning TypeId::of::<Arc<dyn SkimItem>>(),
    // which never matches K8sItem. Instead, deref through the Arc to get &dyn SkimItem and
    // call as_any() through the vtable, which dispatches to K8sItem::as_any() and returns
    // the correct TypeId.
    let items: Vec<&K8sItem> = output
        .selected_items
        .iter()
        .filter_map(|matched| {
            let inner: &dyn SkimItem = &*matched.item;
            inner.as_any().downcast_ref::<K8sItem>()
        })
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
