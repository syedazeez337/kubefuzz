use anyhow::Result;
use futures::StreamExt;
use k8s_openapi::{
    api::{
        apps::v1::{DaemonSet, Deployment, StatefulSet},
        batch::v1::{CronJob, Job},
        core::v1::{ConfigMap, Namespace, Node, PersistentVolumeClaim, Pod, Secret, Service},
        networking::v1::Ingress,
    },
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
    jiff::{SpanRound, Timestamp, Unit},
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
/// Initial items are sorted (unhealthy first) and sent as a batch at `InitDone`.
/// Subsequent Apply/Delete events are streamed in real-time.
/// Automatically reconnects on watch failures via `default_backoff`.
pub async fn watch_resources(
    client: Client,
    tx: SkimItemSender,
    kinds: &[ResourceKind],
    context: &str,
    namespace: Option<&str>,
) -> Result<()> {
    let mut tasks = Vec::new();

    for kind in kinds {
        let c = client.clone();
        let t = tx.clone();
        let k = *kind;
        let ctx = context.to_string();
        let ns = namespace.map(str::to_string);

        tasks.push(tokio::spawn(async move {
            let result = match k {
                ResourceKind::Pod => {
                    watch_typed::<Pod, _>(c, t, ResourceKind::Pod, pod_status, ctx, ns).await
                }
                ResourceKind::Service => {
                    watch_typed::<Service, _>(c, t, ResourceKind::Service, service_status, ctx, ns)
                        .await
                }
                ResourceKind::Deployment => {
                    watch_typed::<Deployment, _>(
                        c,
                        t,
                        ResourceKind::Deployment,
                        deploy_status,
                        ctx,
                        ns,
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
                        ns,
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
                        ns,
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
                        ns,
                    )
                    .await
                }
                ResourceKind::Secret => {
                    watch_typed::<Secret, _>(c, t, ResourceKind::Secret, secret_status, ctx, ns)
                        .await
                }
                ResourceKind::Ingress => {
                    watch_typed::<Ingress, _>(c, t, ResourceKind::Ingress, ingress_status, ctx, ns)
                        .await
                }
                // Cluster-scoped resources always use Api::all regardless of --namespace
                ResourceKind::Node => {
                    watch_typed::<Node, _>(c, t, ResourceKind::Node, node_status, ctx, None).await
                }
                ResourceKind::Namespace => {
                    watch_typed::<Namespace, _>(
                        c,
                        t,
                        ResourceKind::Namespace,
                        namespace_status,
                        ctx,
                        None,
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
                        ns,
                    )
                    .await
                }
                ResourceKind::Job => {
                    watch_typed::<Job, _>(c, t, ResourceKind::Job, job_status, ctx, ns).await
                }
                ResourceKind::CronJob => {
                    watch_typed::<CronJob, _>(c, t, ResourceKind::CronJob, cronjob_status, ctx, ns)
                        .await
                }
            };

            if let Err(e) = result {
                eprintln!("\n[kubefuzz] {e}");
            }
        }));
    }

    for task in tasks {
        if let Err(e) = task.await {
            eprintln!("[kubefuzz] warning: watcher task panicked: {e}");
        }
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
    namespace: Option<String>,
) -> Result<()>
where
    T: Resource<DynamicType = ()> + DeserializeOwned + Clone + Send + Sync + Debug + 'static,
    F: Fn(&T) -> String,
{
    let api: Api<T> = Api::all(client);
    let watcher_config = match namespace.as_deref() {
        Some(ns) => watcher::Config::default().fields(&format!("metadata.namespace={ns}")),
        None => watcher::Config::default(),
    };
    let mut stream = pin!(watcher(api, watcher_config).default_backoff());

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
                let item = make_item(&r, kind, &status_fn, false, &context);
                if in_init {
                    init_batch.push(item);
                } else {
                    // Shouldn't occur, but handle gracefully.
                    if tx
                        .send(vec![Arc::new(item) as Arc<dyn skim::SkimItem>])
                        .is_err()
                    {
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
                init_batch.sort_by_key(|item| std::cmp::Reverse(status_priority(item.status())));
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
                let item = make_item(&r, kind, &status_fn, false, &context);
                if tx
                    .send(vec![Arc::new(item) as Arc<dyn skim::SkimItem>])
                    .is_err()
                {
                    break;
                }
            }

            // ── Live deletion ─────────────────────────────────────────────────
            Ok(watcher::Event::Delete(r)) => {
                let item = make_item(&r, kind, &status_fn, true, &context);
                if tx
                    .send(vec![Arc::new(item) as Arc<dyn skim::SkimItem>])
                    .is_err()
                {
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
    kind: ResourceKind,
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
    K8sItem::new(kind, ns, name, status, age, context)
}

// ─── Status priority (lower = shown first) ───────────────────────────────────

pub fn status_priority(status: &str) -> u8 {
    crate::items::StatusHealth::classify(status).priority()
}

// ─── Per-resource status extractors ──────────────────────────────────────────

fn pod_status(pod: &Pod) -> String {
    if pod.metadata.deletion_timestamp.is_some() {
        return "Terminating".to_string();
    }
    let Some(status) = &pod.status else {
        return "Unknown".to_string();
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
                    .is_some_and(|t| t.exit_code == 0)
            })
            .count();
        if done < total {
            return format!("Init:{done}/{total}");
        }
    }
    status
        .phase
        .clone()
        .unwrap_or_else(|| "Unknown".to_string())
}

fn service_status(svc: &Service) -> String {
    svc.spec
        .as_ref()
        .and_then(|s| s.type_.as_deref())
        .unwrap_or("ClusterIP")
        .to_string()
}

fn deploy_status(d: &Deployment) -> String {
    let ready = d
        .status
        .as_ref()
        .and_then(|s| s.ready_replicas)
        .unwrap_or(0);
    let desired = d.spec.as_ref().and_then(|s| s.replicas).unwrap_or(1);
    format!("{ready}/{desired}")
}

fn statefulset_status(sts: &StatefulSet) -> String {
    let ready = sts
        .status
        .as_ref()
        .and_then(|s| s.ready_replicas)
        .unwrap_or(0);
    let total = sts.status.as_ref().map_or(0, |s| s.replicas);
    format!("{ready}/{total}")
}

fn daemonset_status(ds: &DaemonSet) -> String {
    let ready = ds.status.as_ref().map_or(0, |s| s.number_ready);
    let desired = ds.status.as_ref().map_or(0, |s| s.desired_number_scheduled);
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
        .map_or_else(
            || "Unknown".to_string(),
            |c| {
                if c.status == "True" {
                    "Ready".to_string()
                } else {
                    "NotReady".to_string()
                }
            },
        )
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
        .map_or(0, Vec::len);
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
                .map(
                    |dur| match (dur.get_days(), dur.get_hours(), dur.get_minutes()) {
                        (d, _, _) if d > 0 => format!("{d}d"),
                        (_, h, _) if h > 0 => format!("{h}h"),
                        (_, _, m) => format!("{m}m"),
                    },
                )
        })
        .unwrap_or_else(|| "?".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::api::core::v1::{
        ContainerState, ContainerStateTerminated, ContainerStateWaiting, ContainerStatus, PodStatus,
    };

    // ── status_priority ───────────────────────────────────────────────────────

    #[test]
    fn priority_critical_statuses() {
        for s in &[
            "CrashLoopBackOff",
            "ImagePullBackOff",
            "ErrImagePull",
            "Error",
            "Failed",
            "OOMKilled",
            "NotReady",
            "Failed(3)",
            "Evicted",
            "BackOff",
        ] {
            assert_eq!(
                status_priority(s),
                0,
                "'{s}' should be priority 0 (critical)"
            );
        }
    }

    #[test]
    fn priority_warning_statuses() {
        for s in &[
            "[DELETED]",
            "Pending",
            "ContainerCreating",
            "Terminating",
            "Init:0/1",
        ] {
            assert_eq!(
                status_priority(s),
                1,
                "'{s}' should be priority 1 (warning)"
            );
        }
    }

    #[test]
    fn priority_healthy_statuses() {
        for s in &["Running", "Active", "ClusterIP", "Complete", "Succeeded"] {
            assert_eq!(
                status_priority(s),
                2,
                "'{s}' should be priority 2 (healthy)"
            );
        }
    }

    // ── resource_age ─────────────────────────────────────────────────────────

    #[test]
    fn resource_age_no_timestamp_returns_question_mark() {
        use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
        assert_eq!(resource_age(&ObjectMeta::default()), "?");
    }

    // ── pod_status ────────────────────────────────────────────────────────────

    #[test]
    fn pod_status_no_status_is_unknown() {
        assert_eq!(pod_status(&Pod::default()), "Unknown");
    }

    #[test]
    fn pod_status_phase_running() {
        let pod = Pod {
            status: Some(PodStatus {
                phase: Some("Running".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(pod_status(&pod), "Running");
    }

    #[test]
    fn pod_status_crashloop() {
        let pod = Pod {
            status: Some(PodStatus {
                container_statuses: Some(vec![ContainerStatus {
                    state: Some(ContainerState {
                        waiting: Some(ContainerStateWaiting {
                            reason: Some("CrashLoopBackOff".to_string()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(pod_status(&pod), "CrashLoopBackOff");
    }

    #[test]
    fn pod_status_oomkilled() {
        let pod = Pod {
            status: Some(PodStatus {
                container_statuses: Some(vec![ContainerStatus {
                    state: Some(ContainerState {
                        terminated: Some(ContainerStateTerminated {
                            exit_code: 137,
                            reason: Some("OOMKilled".to_string()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(pod_status(&pod), "OOMKilled");
    }

    #[test]
    fn pod_status_init_progress() {
        let pod = Pod {
            status: Some(PodStatus {
                init_container_statuses: Some(vec![ContainerStatus {
                    // running but not terminated — counts as 0 done out of 1
                    state: Some(ContainerState {
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(pod_status(&pod), "Init:0/1");
    }

    // ── deploy_status ─────────────────────────────────────────────────────────

    #[test]
    fn deploy_status_fully_ready() {
        use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec, DeploymentStatus};
        let d = Deployment {
            spec: Some(DeploymentSpec {
                replicas: Some(3),
                ..Default::default()
            }),
            status: Some(DeploymentStatus {
                ready_replicas: Some(3),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(deploy_status(&d), "3/3");
    }

    #[test]
    fn deploy_status_degraded() {
        use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec, DeploymentStatus};
        let d = Deployment {
            spec: Some(DeploymentSpec {
                replicas: Some(3),
                ..Default::default()
            }),
            status: Some(DeploymentStatus {
                ready_replicas: Some(1),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(deploy_status(&d), "1/3");
    }
}
