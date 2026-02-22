mod items;

use anyhow::Result;
use items::{K8sItem, ResourceKind};
use skim::prelude::*;
use std::sync::Arc;

fn main() -> Result<()> {
    let options = SkimOptionsBuilder::default()
        .multi(true)
        .preview(String::new()) // empty string enables the preview pane
        .preview_window("right:50%")
        .height("60%")
        .header(
            "KubeFuzz [DEMO]  <tab> multi-select  <enter> select  \
             ctrl-l logs  ctrl-e exec  ctrl-d delete  esc quit",
        )
        .prompt("❯ ")
        .build()?;

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    // Phase 0: hardcoded fake K8s resources to verify TUI works.
    // Phase 1 will replace this with real kube-rs API calls.
    let fake_items: Vec<Arc<dyn SkimItem>> = vec![
        // --- Pods ---
        Arc::new(K8sItem::new(ResourceKind::Pod, "production", "api-server-7d9f8b6c5-xk2lp", "Running", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "production", "worker-6f8b9c4d7-mn3qr", "Running", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "production", "cache-3a5b6c7d8-ij3kl", "CrashLoopBackOff", "1h")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "staging", "frontend-5c7d8e9f0-ab1cd", "Pending", "5m")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "staging", "db-migrator-4b6c7d8e9-ef2gh", "Failed", "10m")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "staging", "worker-7d9e0f1g2-hi3jk", "Running", "1d")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "kube-system", "coredns-5dd5756b68-lm4no", "Running", "7d")),
        Arc::new(K8sItem::new(ResourceKind::Pod, "kube-system", "kube-proxy-pq5rs", "Running", "7d")),
        // --- Services ---
        Arc::new(K8sItem::new(ResourceKind::Service, "production", "api-service", "ClusterIP", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Service, "production", "worker-headless", "ClusterIP", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Service, "staging", "frontend-lb", "LoadBalancer", "5m")),
        // --- Deployments ---
        Arc::new(K8sItem::new(ResourceKind::Deployment, "production", "api-server", "3/3", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Deployment, "production", "worker", "5/5", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Deployment, "staging", "frontend", "0/1", "5m")),
        // --- ConfigMaps & Secrets ---
        Arc::new(K8sItem::new(ResourceKind::ConfigMap, "production", "api-config", "Active", "2d")),
        Arc::new(K8sItem::new(ResourceKind::Secret, "production", "api-tls-cert", "Active", "30d")),
        // --- Namespaces ---
        Arc::new(K8sItem::new(ResourceKind::Namespace, "", "production", "Active", "30d")),
        Arc::new(K8sItem::new(ResourceKind::Namespace, "", "staging", "Active", "10d")),
        Arc::new(K8sItem::new(ResourceKind::Namespace, "", "kube-system", "Active", "7d")),
    ];

    tx.send(fake_items)?;
    drop(tx); // signal end of input — skim knows no more items are coming

    // Skim uses color_eyre::Result internally, so we map to anyhow
    let output = Skim::run_with(options, Some(rx)).map_err(|e| anyhow::anyhow!("{e}"))?;

    if !output.is_abort {
        for item in &output.selected_items {
            if let Some(k8s) = (**item).as_any().downcast_ref::<K8sItem>() {
                println!("{}", k8s.output_str());
            }
        }
    }

    Ok(())
}
