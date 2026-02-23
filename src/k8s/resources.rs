use anyhow::Result;
use futures::StreamExt;
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
    api::Api,
    runtime::{watcher, WatchStreamExt},
    Client, Resource, ResourceExt,
};
use serde::de::DeserializeOwned;
use skim::SkimItemSender;
use std::{fmt::Debug, pin::pin, sync::Arc};

use crate::items::{K8sItem, ResourceKind};

/// All resource kinds to watch when no filter is given.
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

/// Watch the given resource kinds from the cluster, streaming live updates into skim.
/// `context` is a display label attached to every item (empty string in single-cluster mode).
/// Initial items are sorted (unhealthy first) and sent as a batch at InitDone.
/// Subsequent Apply/Delete events are streamed in real-time.
/// Automatically reconnects on watch failures via default_backoff.
pub async fn watch_resources(
    client: Client,
    tx: SkimItemSender,
    kinds: &[ResourceKind],
    context: &str,
) -> Result<()> {
    let mut tasks = Vec::new();

    for kind in kinds {
        let c = client.clone();
        let t = tx.clone();
        let k = kind.clone();
        let ctx = context.to_string();

        tasks.push(tokio::spawn(async move {
            let result = match k {
                ResourceKind::Pod => {
                    watch_typed::<Pod, _>(c, t, ResourceKind::Pod, pod_status, ctx).await
                }
                ResourceKind::Service => {
                    watch_typed::<Service, _>(c, t, ResourceKind::Service, service_status, ctx)
                        .await
                }
                ResourceKind::Deployment => {
                    watch_typed::<Deployment, _>(
                        c,
                        t,
                        ResourceKind::Deployment,
                        deploy_status,
                        ctx,
                    )
                    .await
                }
                ResourceKind::StatefulSet => {
                    watch_typed::<StatefulSet, _>(
                        c,
                        t,
                        ResourceKind::StatefulSet,
                        statefulset_status,
                        ctx,
                    )
                    .await
                }
                ResourceKind::DaemonSet => {
                    watch_typed::<DaemonSet, _>(
                        c,
                        t,
                        ResourceKind::DaemonSet,
                        daemonset_status,
                        ctx,
                    )
                    .await
                }
                ResourceKind::ConfigMap => {
                    watch_typed::<ConfigMap, _>(
                        c,
                        t,
                        ResourceKind::ConfigMap,
                        |_| "ConfigMap".to_string(),
                        ctx,
                    )
                    .await
                }
                ResourceKind::Secret => {
                    watch_typed::<Secret, _>(c, t, ResourceKind::Secret, secret_status, ctx).await
                }
                ResourceKind::Ingress => {
                    watch_typed::<Ingress, _>(c, t, ResourceKind::Ingress, ingress_status, ctx)
                        .await
                }
                ResourceKind::Node => {
                    watch_typed::<Node, _>(c, t, ResourceKind::Node, node_status, ctx).await
                }
                ResourceKind::Namespace => {
                    watch_typed::<Namespace, _>(
                        c,
                        t,
                        ResourceKind::Namespace,
                        namespace_status,
                        ctx,
                    )
                    .await
                }
                ResourceKind::PersistentVolumeClaim => {
                    watch_typed::<PersistentVolumeClaim, _>(
                        c,
                        t,
                        ResourceKind::PersistentVolumeClaim,
                        pvc_status,
                        ctx,
                    )
                    .await
                }
                ResourceKind::Job => {
                    watch_typed::<Job, _>(c, t, ResourceKind::Job, job_status, ctx).await
                }
                ResourceKind::CronJob => {
                    watch_typed::<CronJob, _>(c, t, ResourceKind::CronJob, cronjob_status, ctx)
                        .await
                }
            };

            if let Err(e) = result {
                eprintln!("\n[kubefuzz] {e}");
            }
        }));
    }

    for task in tasks {
        let _ = task.await;
    }

    Ok(())
}

// ─── Generic typed watcher ───────────────────────────────────────────────────

/// Watch all resources of type `T` across all namespaces.
///
/// Lifecycle:
/// - `Init`      → new watch cycle starting; clear the init buffer.
/// - `InitApply` → existing object; buffer it.
/// - `InitDone`  → sort the buffer by health priority and send the whole batch.
/// - `Apply`     → live add/modify; send immediately.
/// - `Delete`    → live deletion; send with `[DELETED]` status so it's visible.
///
/// The watcher reconnects automatically on failures via `default_backoff()`.
/// The loop exits cleanly when skim closes the channel (send returns Err).
async fn watch_typed<T, F>(
    client: Client,
    tx: SkimItemSender,
    kind: ResourceKind,
    status_fn: F,
    context: String,
) -> Result<()>
where
    T: Resource<DynamicType = ()> + DeserializeOwned + Clone + Send + Sync + Debug + 'static,
    F: Fn(&T) -> String,
{
    let api: Api<T> = Api::all(client);
    let mut stream = pin!(watcher(api, watcher::Config::default()).default_backoff());

    // Buffer for initial items so we can sort before the first render.
    // Stored as K8sItem (concrete type) so we can sort without downcast.
    let mut init_batch: Vec<K8sItem> = Vec::new();
    let mut in_init = true;

    while let Some(event) = stream.next().await {
        match event {
            // ── Init cycle start ──────────────────────────────────────────────
            Ok(watcher::Event::Init) => {
                init_batch.clear();
                in_init = true;
            }

            // ── Existing object during initial list ───────────────────────────
            Ok(watcher::Event::InitApply(r)) => {
                let item = make_item(&r, &kind, &status_fn, false, &context);
                if in_init {
                    init_batch.push(item);
                } else {
                    // Shouldn't occur, but handle gracefully.
                    if tx.send(vec![Arc::new(item) as Arc<dyn skim::SkimItem>]).is_err() {
                        break;
                    }
                }
            }

            // ── Initial list complete — sort & send ───────────────────────────
            Ok(watcher::Event::InitDone) => {
                // Sort by priority descending: healthy (2) first, unhealthy (0) last.
                // Skim renders higher-indexed items at the TOP of the list (lower indices
                // appear near the prompt at the bottom). Sending unhealthy items LAST
                // gives them the highest indices so they surface to the top of the display.
                init_batch.sort_by_key(|item| std::cmp::Reverse(status_priority(&item.status)));
                let sorted: Vec<Arc<dyn skim::SkimItem>> = init_batch
                    .drain(..)
                    .map(|item| Arc::new(item) as Arc<dyn skim::SkimItem>)
                    .collect();
                if !sorted.is_empty() && tx.send(sorted).is_err() {
                    break;
                }
                in_init = false;
            }

            // ── Live add / update ─────────────────────────────────────────────
            Ok(watcher::Event::Apply(r)) => {
                let item = make_item(&r, &kind, &status_fn, false, &context);
                if tx.send(vec![Arc::new(item) as Arc<dyn skim::SkimItem>]).is_err() {
                    break;
                }
            }

            // ── Live deletion ─────────────────────────────────────────────────
            Ok(watcher::Event::Delete(r)) => {
                let item = make_item(&r, &kind, &status_fn, true, &context);
                if tx.send(vec![Arc::new(item) as Arc<dyn skim::SkimItem>]).is_err() {
                    break;
                }
            }

            // ── Watch error — default_backoff handles retry ───────────────────
            Err(e) => {
                eprintln!("[kubefuzz] watch error ({}): {e}", kind.as_str());
            }
        }
    }

    Ok(())
}

// ─── Item constructor helper ─────────────────────────────────────────────────

fn make_item<T>(
    r: &T,
    kind: &ResourceKind,
    status_fn: &impl Fn(&T) -> String,
    deleted: bool,
    context: &str,
) -> K8sItem
where
    T: Resource<DynamicType = ()>,
{
    let ns = r.meta().namespace.clone().unwrap_or_default();
    let name = r.name_any();
    let status = if deleted {
        "[DELETED]".to_string()
    } else {
        status_fn(r)
    };
    let age = resource_age(r.meta());
    let mut item = K8sItem::new(kind.clone(), ns, name, status, age);
    item.context = context.to_string();
    item
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
        "[DELETED]" => 1,
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
                        // "PodInitializing" = main container waiting for init containers —
                        // skip it and let the init container block below compute Init:X/Y
                        if reason != "ContainerCreating" && reason != "PodInitializing" {
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
    // Init container status — check waiting reason first, then running progress
    if let Some(ics) = &status.init_container_statuses {
        // Explicit waiting reason (e.g. Init:ErrImagePull)
        for cs in ics {
            if let Some(state) = &cs.state {
                if let Some(waiting) = &state.waiting {
                    if let Some(reason) = &waiting.reason {
                        return format!("Init:{reason}");
                    }
                }
            }
        }
        // One or more init containers are running — show X/Y progress like kubectl
        let total = ics.len();
        let done = ics
            .iter()
            .filter(|cs| {
                cs.state
                    .as_ref()
                    .and_then(|s| s.terminated.as_ref())
                    .map(|t| t.exit_code == 0)
                    .unwrap_or(false)
            })
            .count();
        if done < total {
            return format!("Init:{done}/{total}");
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
