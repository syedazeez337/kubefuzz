use anyhow::Result;
use k8s_openapi::{
    api::core::v1::Pod,
    jiff::{SpanRound, Timestamp, Unit},
};
use kube::{
    api::{Api, ListParams},
    Client, ResourceExt,
};
use skim::SkimItemSender;
use std::sync::Arc;

use crate::items::{K8sItem, ResourceKind};

/// List all pods across all namespaces and send them to the skim channel.
/// This function is async and designed to be spawned as a tokio task.
pub async fn stream_pods(client: Client, tx: SkimItemSender) -> Result<()> {
    let pods: Api<Pod> = Api::all(client);
    let pod_list = pods
        .list(&ListParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list pods: {e}\nIs the cluster reachable?"))?;

    let items: Vec<Arc<dyn skim::SkimItem>> = pod_list
        .items
        .into_iter()
        .map(|pod| {
            let item = K8sItem::new(
                ResourceKind::Pod,
                pod.namespace().unwrap_or_default(),
                pod.name_any(),
                pod_status(&pod),
                pod_age(&pod),
            );
            Arc::new(item) as Arc<dyn skim::SkimItem>
        })
        .collect();

    if !items.is_empty() {
        // Send all items as one batch — skim displays them as they arrive
        tx.send(items)?;
    }

    Ok(())
}

/// Extract a human-readable status string from a Pod object.
/// Mimics what kubectl shows in the STATUS column.
fn pod_status(pod: &Pod) -> String {
    // Terminating takes priority over everything else
    if pod.metadata.deletion_timestamp.is_some() {
        return "Terminating".to_string();
    }

    let status = match &pod.status {
        Some(s) => s,
        None => return "Unknown".to_string(),
    };

    // Check container statuses for waiting reasons (CrashLoopBackOff, OOMKilled, etc.)
    if let Some(container_statuses) = &status.container_statuses {
        for cs in container_statuses {
            if let Some(state) = &cs.state {
                if let Some(waiting) = &state.waiting {
                    if let Some(reason) = &waiting.reason {
                        if reason != "ContainerCreating" {
                            return reason.clone();
                        }
                    }
                }
                // Completed / error in terminated state
                if let Some(terminated) = &state.terminated {
                    if terminated.exit_code != 0 {
                        return terminated
                            .reason
                            .clone()
                            .unwrap_or_else(|| "Error".to_string());
                    }
                }
            }
        }
    }

    // Check init container statuses
    if let Some(init_statuses) = &status.init_container_statuses {
        for cs in init_statuses {
            if let Some(state) = &cs.state {
                if let Some(waiting) = &state.waiting {
                    if let Some(reason) = &waiting.reason {
                        return format!("Init:{reason}");
                    }
                }
            }
        }
    }

    // Fall back to phase
    status
        .phase
        .clone()
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Format the age of a pod from its creation timestamp.
fn pod_age(pod: &Pod) -> String {
    pod.metadata
        .creation_timestamp
        .as_ref()
        .and_then(|t| {
            Timestamp::now()
                .since(t.0)
                .ok()
                .and_then(|dur| {
                    dur.round(
                        SpanRound::new()
                            .largest(Unit::Day)
                            .days_are_24_hours()
                            .smallest(Unit::Minute),
                    )
                    .ok()
                })
                .map(|dur| match (dur.get_days(), dur.get_hours(), dur.get_minutes()) {
                    (d, _, _) if d > 0 => format!("{d}d"),
                    (_, h, _) if h > 0 => format!("{h}h"),
                    (_, _, m) => format!("{m}m"),
                })
        })
        .unwrap_or_else(|| "?".to_string())
}
