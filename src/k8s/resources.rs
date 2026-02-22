use anyhow::Result;
use k8s_openapi::{
    api::{
        apps::v1::{DaemonSet, Deployment, StatefulSet},
        batch::v1::{CronJob, Job},
        core::v1::{ConfigMap, Namespace, Node, PersistentVolumeClaim, Pod, Secret, Service},
        networking::v1::Ingress,
    },
    jiff::{SpanRound, Timestamp, Unit},
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
};
use kube::{
    api::{Api, ListParams},
    Client, Resource, ResourceExt,
};
use serde::de::DeserializeOwned;
use skim::SkimItemSender;
use std::sync::Arc;

use crate::items::{K8sItem, ResourceKind};

/// All resource kinds to stream when no filter is given.
pub const ALL_KINDS: &[ResourceKind] = &[
    ResourceKind::Pod,
    ResourceKind::Deployment,
    ResourceKind::StatefulSet,
    ResourceKind::DaemonSet,
    ResourceKind::Service,
    ResourceKind::Ingress,
    ResourceKind::Job,
    ResourceKind::CronJob,
    ResourceKind::ConfigMap,
    ResourceKind::Secret,
    ResourceKind::PersistentVolumeClaim,
    ResourceKind::Namespace,
    ResourceKind::Node,
];

/// Stream the given resource kinds from the cluster into the skim channel.
/// Each kind is fetched concurrently. Errors per-kind are logged but do not
/// abort the others.
pub async fn stream_resources(
    client: Client,
    tx: SkimItemSender,
    kinds: &[ResourceKind],
) -> Result<()> {
    let mut tasks = Vec::new();

    for kind in kinds {
        let c = client.clone();
        let t = tx.clone();
        let k = kind.clone();

        tasks.push(tokio::spawn(async move {
            let result = match k {
                ResourceKind::Pod => {
                    stream_typed::<Pod, _>(c, t, ResourceKind::Pod, pod_status).await
                }
                ResourceKind::Service => {
                    stream_typed::<Service, _>(c, t, ResourceKind::Service, service_status).await
                }
                ResourceKind::Deployment => {
                    stream_typed::<Deployment, _>(c, t, ResourceKind::Deployment, deploy_status)
                        .await
                }
                ResourceKind::StatefulSet => {
                    stream_typed::<StatefulSet, _>(
                        c,
                        t,
                        ResourceKind::StatefulSet,
                        statefulset_status,
                    )
                    .await
                }
                ResourceKind::DaemonSet => {
                    stream_typed::<DaemonSet, _>(c, t, ResourceKind::DaemonSet, daemonset_status)
                        .await
                }
                ResourceKind::ConfigMap => {
                    stream_typed::<ConfigMap, _>(c, t, ResourceKind::ConfigMap, |_| {
                        "ConfigMap".to_string()
                    })
                    .await
                }
                ResourceKind::Secret => {
                    stream_typed::<Secret, _>(c, t, ResourceKind::Secret, secret_status).await
                }
                ResourceKind::Ingress => {
                    stream_typed::<Ingress, _>(c, t, ResourceKind::Ingress, ingress_status).await
                }
                ResourceKind::Node => {
                    stream_typed::<Node, _>(c, t, ResourceKind::Node, node_status).await
                }
                ResourceKind::Namespace => {
                    stream_typed::<Namespace, _>(c, t, ResourceKind::Namespace, namespace_status)
                        .await
                }
                ResourceKind::PersistentVolumeClaim => {
                    stream_typed::<PersistentVolumeClaim, _>(
                        c,
                        t,
                        ResourceKind::PersistentVolumeClaim,
                        pvc_status,
                    )
                    .await
                }
                ResourceKind::Job => {
                    stream_typed::<Job, _>(c, t, ResourceKind::Job, job_status).await
                }
                ResourceKind::CronJob => {
                    stream_typed::<CronJob, _>(c, t, ResourceKind::CronJob, cronjob_status).await
                }
            };

            if let Err(e) = result {
                // Log per-resource errors without aborting other streams
                eprintln!("\n[kubefuzz] {e}");
            }
        }));
    }

    // Wait for all resource-type fetches to finish
    for task in tasks {
        let _ = task.await;
    }

    Ok(())
}

// ─── Generic typed streamer ──────────────────────────────────────────────────

/// List all resources of type `T` across all namespaces, sort by status
/// priority (failed/pending first), and send to skim.
async fn stream_typed<T, F>(
    client: Client,
    tx: SkimItemSender,
    kind: ResourceKind,
    status_fn: F,
) -> Result<()>
where
    T: Resource<DynamicType = ()> + DeserializeOwned + Clone + Send + Sync + std::fmt::Debug + 'static,
    F: Fn(&T) -> String,
{
    let api: Api<T> = Api::all(client);
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to list {}s: {e}", kind.as_str()))?;

    let mut items: Vec<Arc<dyn skim::SkimItem>> = list
        .items
        .iter()
        .map(|r| {
            let ns = r.meta().namespace.clone().unwrap_or_default();
            let name = r.name_any();
            let status = status_fn(r);
            let age = resource_age(r.meta());
            Arc::new(K8sItem::new(kind.clone(), ns, name, status, age)) as Arc<dyn skim::SkimItem>
        })
        .collect();

    // Sort: unhealthy resources bubble to the top
    items.sort_by_key(|item| {
        (**item)
            .as_any()
            .downcast_ref::<K8sItem>()
            .map(|k| status_priority(&k.status))
            .unwrap_or(99)
    });

    if !items.is_empty() {
        tx.send(items)?;
    }

    Ok(())
}

// ─── Status priority (lower = shown first) ───────────────────────────────────

pub fn status_priority(status: &str) -> u8 {
    match status {
        s if s.starts_with("CrashLoop")
            || s.starts_with("ErrImage")
            || s.starts_with("ImagePull")
            || s == "Error"
            || s == "Failed"
            || s == "OOMKilled"
            || s == "NotReady"
            || s.starts_with("Failed(") =>
        {
            0
        }
        "Pending" | "ContainerCreating" | "Terminating" | "Unknown" => 1,
        s if s.starts_with("Init:") => 1,
        _ => 2,
    }
}

// ─── Per-resource status extractors ──────────────────────────────────────────

fn pod_status(pod: &Pod) -> String {
    if pod.metadata.deletion_timestamp.is_some() {
        return "Terminating".to_string();
    }
    let status = match &pod.status {
        Some(s) => s,
        None => return "Unknown".to_string(),
    };
    // Container-level waiting reasons (CrashLoopBackOff, etc.)
    if let Some(css) = &status.container_statuses {
        for cs in css {
            if let Some(state) = &cs.state {
                if let Some(waiting) = &state.waiting {
                    if let Some(reason) = &waiting.reason {
                        if reason != "ContainerCreating" {
                            return reason.clone();
                        }
                    }
                }
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
    // Init container waiting
    if let Some(ics) = &status.init_container_statuses {
        for cs in ics {
            if let Some(state) = &cs.state {
                if let Some(waiting) = &state.waiting {
                    if let Some(reason) = &waiting.reason {
                        return format!("Init:{reason}");
                    }
                }
            }
        }
    }
    status.phase.clone().unwrap_or_else(|| "Unknown".to_string())
}

fn service_status(svc: &Service) -> String {
    svc.spec
        .as_ref()
        .and_then(|s| s.type_.as_deref())
        .unwrap_or("ClusterIP")
        .to_string()
}

fn deploy_status(d: &Deployment) -> String {
    let ready = d.status.as_ref().and_then(|s| s.ready_replicas).unwrap_or(0);
    let desired = d.spec.as_ref().and_then(|s| s.replicas).unwrap_or(1);
    format!("{ready}/{desired}")
}

fn statefulset_status(sts: &StatefulSet) -> String {
    let ready = sts
        .status
        .as_ref()
        .and_then(|s| s.ready_replicas)
        .unwrap_or(0);
    let total = sts.status.as_ref().map(|s| s.replicas).unwrap_or(0);
    format!("{ready}/{total}")
}

fn daemonset_status(ds: &DaemonSet) -> String {
    let ready = ds.status.as_ref().map(|s| s.number_ready).unwrap_or(0);
    let desired = ds
        .status
        .as_ref()
        .map(|s| s.desired_number_scheduled)
        .unwrap_or(0);
    format!("{ready}/{desired}")
}

fn secret_status(s: &Secret) -> String {
    s.type_.clone().unwrap_or_else(|| "Opaque".to_string())
}

fn ingress_status(ing: &Ingress) -> String {
    ing.status
        .as_ref()
        .and_then(|s| s.load_balancer.as_ref())
        .and_then(|lb| lb.ingress.as_ref())
        .and_then(|v| v.first())
        .and_then(|i| i.ip.as_deref().or(i.hostname.as_deref()))
        .unwrap_or("<pending>")
        .to_string()
}

fn node_status(node: &Node) -> String {
    node.status
        .as_ref()
        .and_then(|s| s.conditions.as_ref())
        .and_then(|conds| conds.iter().find(|c| c.type_ == "Ready"))
        .map(|c| {
            if c.status == "True" {
                "Ready".to_string()
            } else {
                "NotReady".to_string()
            }
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn namespace_status(ns: &Namespace) -> String {
    ns.status
        .as_ref()
        .and_then(|s| s.phase.as_deref())
        .unwrap_or("Active")
        .to_string()
}

fn pvc_status(pvc: &PersistentVolumeClaim) -> String {
    pvc.status
        .as_ref()
        .and_then(|s| s.phase.as_deref())
        .unwrap_or("Unknown")
        .to_string()
}

fn job_status(job: &Job) -> String {
    let s = job.status.as_ref();
    if s.and_then(|s| s.completion_time.as_ref()).is_some() {
        return "Complete".to_string();
    }
    if let Some(failed) = s.and_then(|s| s.failed) {
        if failed > 0 {
            return format!("Failed({failed})");
        }
    }
    let active = s.and_then(|s| s.active).unwrap_or(0);
    if active > 0 {
        return format!("Active({active})");
    }
    "Unknown".to_string()
}

fn cronjob_status(cj: &CronJob) -> String {
    let active = cj
        .status
        .as_ref()
        .and_then(|s| s.active.as_ref())
        .map(|a| a.len())
        .unwrap_or(0);
    if active > 0 {
        format!("Active({active})")
    } else {
        "Scheduled".to_string()
    }
}

// ─── Age helper ───────────────────────────────────────────────────────────────

pub fn resource_age(meta: &ObjectMeta) -> String {
    meta.creation_timestamp
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
