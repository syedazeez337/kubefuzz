//! Tests for kubefuzz::k8s::resources — all status extractors, resource_age, status_priority, ALL_KINDS.

use k8s_openapi::{
    api::{
        apps::v1::{
            DaemonSet, DaemonSetStatus, Deployment, DeploymentSpec, DeploymentStatus, StatefulSet,
            StatefulSetStatus,
        },
        batch::v1::{CronJob, CronJobStatus, Job, JobStatus},
        core::v1::{
            ContainerState, ContainerStateTerminated, ContainerStateWaiting, ContainerStatus,
            Namespace, NamespaceStatus, Node, NodeCondition, NodeStatus, ObjectReference,
            PersistentVolume, PersistentVolumeClaim, PersistentVolumeClaimStatus,
            PersistentVolumeStatus, Pod, PodStatus, Secret, Service, ServiceSpec,
        },
        networking::v1::{
            Ingress, IngressLoadBalancerIngress, IngressLoadBalancerStatus, IngressStatus,
        },
    },
    apimachinery::pkg::apis::meta::v1::{ObjectMeta, Time},
};
use kubefuzz::items::ResourceKind;
use kubefuzz::k8s::resources::{
    cronjob_status, daemonset_status, deploy_status, ingress_status, job_status, namespace_status,
    node_status, pod_status, pv_status, pvc_status, resource_age, secret_status, service_status,
    statefulset_status, status_priority, ALL_KINDS,
};

// ── ALL_KINDS ─────────────────────────────────────────────────────────────────

#[test]
fn all_kinds_has_thirteen_entries() {
    assert_eq!(ALL_KINDS.len(), 14);
}

#[test]
fn all_kinds_contains_every_resource_variant() {
    assert!(ALL_KINDS.contains(&ResourceKind::Pod));
    assert!(ALL_KINDS.contains(&ResourceKind::Deployment));
    assert!(ALL_KINDS.contains(&ResourceKind::StatefulSet));
    assert!(ALL_KINDS.contains(&ResourceKind::DaemonSet));
    assert!(ALL_KINDS.contains(&ResourceKind::Service));
    assert!(ALL_KINDS.contains(&ResourceKind::Ingress));
    assert!(ALL_KINDS.contains(&ResourceKind::Job));
    assert!(ALL_KINDS.contains(&ResourceKind::CronJob));
    assert!(ALL_KINDS.contains(&ResourceKind::ConfigMap));
    assert!(ALL_KINDS.contains(&ResourceKind::Secret));
    assert!(ALL_KINDS.contains(&ResourceKind::PersistentVolume));
    assert!(ALL_KINDS.contains(&ResourceKind::PersistentVolumeClaim));
    assert!(ALL_KINDS.contains(&ResourceKind::Namespace));
    assert!(ALL_KINDS.contains(&ResourceKind::Node));
}

// ── status_priority ───────────────────────────────────────────────────────────

#[test]
fn priority_critical_statuses_are_zero() {
    let critical = [
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
    ];
    for s in &critical {
        assert_eq!(
            status_priority(s),
            0,
            "'{s}' should be priority 0 (critical)"
        );
    }
}

#[test]
fn priority_warning_and_deleted_statuses_are_one() {
    let warning = [
        "[DELETED]",
        "Pending",
        "ContainerCreating",
        "Terminating",
        "Init:0/1",
    ];
    for s in &warning {
        assert_eq!(status_priority(s), 1, "'{s}' should be priority 1");
    }
}

#[test]
fn priority_healthy_statuses_are_two() {
    let healthy = [
        "Running",
        "Active",
        "ClusterIP",
        "Complete",
        "Succeeded",
        "3/3",
    ];
    for s in &healthy {
        assert_eq!(
            status_priority(s),
            2,
            "'{s}' should be priority 2 (healthy)"
        );
    }
}

// ── resource_age ──────────────────────────────────────────────────────────────

#[test]
fn resource_age_no_timestamp_returns_question_mark() {
    assert_eq!(resource_age(&ObjectMeta::default()), "?");
}

#[test]
fn resource_age_old_timestamp_returns_days() {
    // 2020-01-01 is always many days ago
    let time: Time = serde_json::from_str(r#""2020-01-01T00:00:00Z""#).unwrap();
    let meta = ObjectMeta {
        creation_timestamp: Some(time),
        ..Default::default()
    };
    let age = resource_age(&meta);
    assert!(age.ends_with('d'), "expected days suffix, got: {age}");
}

// ── pod_status ────────────────────────────────────────────────────────────────

#[test]
fn pod_status_no_status_returns_unknown() {
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
fn pod_status_phase_pending() {
    let pod = Pod {
        status: Some(PodStatus {
            phase: Some("Pending".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pod_status(&pod), "Pending");
}

#[test]
fn pod_status_terminating_takes_priority() {
    // deletion_timestamp set → "Terminating" regardless of phase
    let pod = Pod {
        metadata: ObjectMeta {
            deletion_timestamp: Some(serde_json::from_str(r#""2024-01-01T00:00:00Z""#).unwrap()),
            ..Default::default()
        },
        status: Some(PodStatus {
            phase: Some("Running".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pod_status(&pod), "Terminating");
}

#[test]
fn pod_status_crashloop_from_container_waiting_reason() {
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
fn pod_status_oomkilled_from_terminated_nonzero_exit() {
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
fn pod_status_terminated_nonzero_without_reason_returns_error() {
    let pod = Pod {
        status: Some(PodStatus {
            container_statuses: Some(vec![ContainerStatus {
                state: Some(ContainerState {
                    terminated: Some(ContainerStateTerminated {
                        exit_code: 1,
                        reason: None,
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
    assert_eq!(pod_status(&pod), "Error");
}

#[test]
fn pod_status_terminated_exit_zero_falls_through_to_phase() {
    // Exit code 0 is not an error — should fall through to phase
    let pod = Pod {
        status: Some(PodStatus {
            container_statuses: Some(vec![ContainerStatus {
                state: Some(ContainerState {
                    terminated: Some(ContainerStateTerminated {
                        exit_code: 0,
                        reason: Some("Completed".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            phase: Some("Succeeded".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pod_status(&pod), "Succeeded");
}

#[test]
fn pod_status_container_creating_reason_falls_through_to_phase() {
    // "ContainerCreating" is explicitly skipped so init logic can take over
    let pod = Pod {
        status: Some(PodStatus {
            container_statuses: Some(vec![ContainerStatus {
                state: Some(ContainerState {
                    waiting: Some(ContainerStateWaiting {
                        reason: Some("ContainerCreating".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            phase: Some("Pending".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pod_status(&pod), "Pending");
}

#[test]
fn pod_status_pod_initializing_reason_falls_through_to_init_containers() {
    // "PodInitializing" is skipped; init:0/1 is computed from init containers
    let pod = Pod {
        status: Some(PodStatus {
            container_statuses: Some(vec![ContainerStatus {
                state: Some(ContainerState {
                    waiting: Some(ContainerStateWaiting {
                        reason: Some("PodInitializing".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            init_container_statuses: Some(vec![ContainerStatus {
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

#[test]
fn pod_status_init_container_waiting_reason() {
    let pod = Pod {
        status: Some(PodStatus {
            init_container_statuses: Some(vec![ContainerStatus {
                state: Some(ContainerState {
                    waiting: Some(ContainerStateWaiting {
                        reason: Some("ErrImagePull".to_string()),
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
    assert_eq!(pod_status(&pod), "Init:ErrImagePull");
}

#[test]
fn pod_status_init_progress_none_done() {
    let pod = Pod {
        status: Some(PodStatus {
            init_container_statuses: Some(vec![ContainerStatus {
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

#[test]
fn pod_status_init_progress_partial() {
    // 1 done (exit_code=0), 2 total
    let done = ContainerStatus {
        state: Some(ContainerState {
            terminated: Some(ContainerStateTerminated {
                exit_code: 0,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    let running = ContainerStatus {
        state: Some(ContainerState {
            ..Default::default()
        }),
        ..Default::default()
    };
    let pod = Pod {
        status: Some(PodStatus {
            init_container_statuses: Some(vec![done, running]),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pod_status(&pod), "Init:1/2");
}

// ── service_status ────────────────────────────────────────────────────────────

#[test]
fn service_status_defaults_to_clusterip() {
    assert_eq!(service_status(&Service::default()), "ClusterIP");
}

#[test]
fn service_status_nodeport() {
    let svc = Service {
        spec: Some(ServiceSpec {
            type_: Some("NodePort".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(service_status(&svc), "NodePort");
}

#[test]
fn service_status_loadbalancer() {
    let svc = Service {
        spec: Some(ServiceSpec {
            type_: Some("LoadBalancer".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(service_status(&svc), "LoadBalancer");
}

#[test]
fn service_status_externalname() {
    let svc = Service {
        spec: Some(ServiceSpec {
            type_: Some("ExternalName".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(service_status(&svc), "ExternalName");
}

// ── deploy_status ─────────────────────────────────────────────────────────────

#[test]
fn deploy_status_defaults_zero_of_one() {
    // No spec replicas → desired=1; no status ready → ready=0
    assert_eq!(deploy_status(&Deployment::default()), "0/1");
}

#[test]
fn deploy_status_fully_ready() {
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

#[test]
fn deploy_status_zero_replicas() {
    let d = Deployment {
        spec: Some(DeploymentSpec {
            replicas: Some(0),
            ..Default::default()
        }),
        status: Some(DeploymentStatus {
            ready_replicas: Some(0),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(deploy_status(&d), "0/0");
}

// ── statefulset_status ────────────────────────────────────────────────────────

#[test]
fn statefulset_status_defaults_zero_of_zero() {
    assert_eq!(statefulset_status(&StatefulSet::default()), "0/0");
}

#[test]
fn statefulset_status_fully_ready() {
    let sts = StatefulSet {
        status: Some(StatefulSetStatus {
            ready_replicas: Some(3),
            replicas: 3,
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(statefulset_status(&sts), "3/3");
}

#[test]
fn statefulset_status_degraded() {
    let sts = StatefulSet {
        status: Some(StatefulSetStatus {
            ready_replicas: Some(1),
            replicas: 3,
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(statefulset_status(&sts), "1/3");
}

// ── daemonset_status ──────────────────────────────────────────────────────────

#[test]
fn daemonset_status_defaults_zero_of_zero() {
    assert_eq!(daemonset_status(&DaemonSet::default()), "0/0");
}

#[test]
fn daemonset_status_fully_scheduled() {
    let ds = DaemonSet {
        status: Some(DaemonSetStatus {
            number_ready: 3,
            desired_number_scheduled: 3,
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(daemonset_status(&ds), "3/3");
}

#[test]
fn daemonset_status_partially_scheduled() {
    let ds = DaemonSet {
        status: Some(DaemonSetStatus {
            number_ready: 2,
            desired_number_scheduled: 5,
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(daemonset_status(&ds), "2/5");
}

// ── secret_status ─────────────────────────────────────────────────────────────

#[test]
fn secret_status_defaults_to_opaque() {
    assert_eq!(secret_status(&Secret::default()), "Opaque");
}

#[test]
fn secret_status_tls_type() {
    let s = Secret {
        type_: Some("kubernetes.io/tls".to_string()),
        ..Default::default()
    };
    assert_eq!(secret_status(&s), "kubernetes.io/tls");
}

#[test]
fn secret_status_service_account_type() {
    let s = Secret {
        type_: Some("kubernetes.io/service-account-token".to_string()),
        ..Default::default()
    };
    assert_eq!(secret_status(&s), "kubernetes.io/service-account-token");
}

// ── ingress_status ────────────────────────────────────────────────────────────

#[test]
fn ingress_status_defaults_to_pending() {
    assert_eq!(ingress_status(&Ingress::default()), "<pending>");
}

#[test]
fn ingress_status_with_ip() {
    let ing = Ingress {
        status: Some(IngressStatus {
            load_balancer: Some(IngressLoadBalancerStatus {
                ingress: Some(vec![IngressLoadBalancerIngress {
                    ip: Some("10.0.0.1".to_string()),
                    hostname: None,
                    ..Default::default()
                }]),
            }),
        }),
        ..Default::default()
    };
    assert_eq!(ingress_status(&ing), "10.0.0.1");
}

#[test]
fn ingress_status_hostname_when_no_ip() {
    let ing = Ingress {
        status: Some(IngressStatus {
            load_balancer: Some(IngressLoadBalancerStatus {
                ingress: Some(vec![IngressLoadBalancerIngress {
                    ip: None,
                    hostname: Some("my-lb.example.com".to_string()),
                    ..Default::default()
                }]),
            }),
        }),
        ..Default::default()
    };
    assert_eq!(ingress_status(&ing), "my-lb.example.com");
}

#[test]
fn ingress_status_ip_preferred_over_hostname() {
    let ing = Ingress {
        status: Some(IngressStatus {
            load_balancer: Some(IngressLoadBalancerStatus {
                ingress: Some(vec![IngressLoadBalancerIngress {
                    ip: Some("1.2.3.4".to_string()),
                    hostname: Some("lb.example.com".to_string()),
                    ..Default::default()
                }]),
            }),
        }),
        ..Default::default()
    };
    assert_eq!(ingress_status(&ing), "1.2.3.4");
}

#[test]
fn ingress_status_empty_ingress_list_is_pending() {
    let ing = Ingress {
        status: Some(IngressStatus {
            load_balancer: Some(IngressLoadBalancerStatus {
                ingress: Some(vec![]),
            }),
        }),
        ..Default::default()
    };
    assert_eq!(ingress_status(&ing), "<pending>");
}

// ── node_status ───────────────────────────────────────────────────────────────

#[test]
fn node_status_no_conditions_returns_unknown() {
    assert_eq!(node_status(&Node::default()), "Unknown");
}

#[test]
fn node_status_ready_true() {
    let node = Node {
        status: Some(NodeStatus {
            conditions: Some(vec![NodeCondition {
                type_: "Ready".to_string(),
                status: "True".to_string(),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(node_status(&node), "Ready");
}

#[test]
fn node_status_ready_false_returns_not_ready() {
    let node = Node {
        status: Some(NodeStatus {
            conditions: Some(vec![NodeCondition {
                type_: "Ready".to_string(),
                status: "False".to_string(),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(node_status(&node), "NotReady");
}

#[test]
fn node_status_no_ready_condition_in_list_returns_unknown() {
    let node = Node {
        status: Some(NodeStatus {
            conditions: Some(vec![NodeCondition {
                type_: "MemoryPressure".to_string(),
                status: "False".to_string(),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(node_status(&node), "Unknown");
}

// ── namespace_status ──────────────────────────────────────────────────────────

#[test]
fn namespace_status_defaults_to_active() {
    assert_eq!(namespace_status(&Namespace::default()), "Active");
}

#[test]
fn namespace_status_terminating() {
    let ns = Namespace {
        status: Some(NamespaceStatus {
            phase: Some("Terminating".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(namespace_status(&ns), "Terminating");
}

#[test]
fn namespace_status_active_explicit() {
    let ns = Namespace {
        status: Some(NamespaceStatus {
            phase: Some("Active".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(namespace_status(&ns), "Active");
}

// ── pv_status ─────────────────────────────────────────────────────────────────

#[test]
fn pv_status_defaults_to_unknown() {
    assert_eq!(pv_status(&PersistentVolume::default()), "Unknown");
}

#[test]
fn pv_status_available() {
    let pv = PersistentVolume {
        status: Some(PersistentVolumeStatus {
            phase: Some("Available".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pv_status(&pv), "Available");
}

#[test]
fn pv_status_bound() {
    let pv = PersistentVolume {
        status: Some(PersistentVolumeStatus {
            phase: Some("Bound".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pv_status(&pv), "Bound");
}

#[test]
fn pv_status_released() {
    let pv = PersistentVolume {
        status: Some(PersistentVolumeStatus {
            phase: Some("Released".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pv_status(&pv), "Released");
}

#[test]
fn pv_status_failed() {
    let pv = PersistentVolume {
        status: Some(PersistentVolumeStatus {
            phase: Some("Failed".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pv_status(&pv), "Failed");
}

// ── pvc_status ────────────────────────────────────────────────────────────────

#[test]
fn pvc_status_defaults_to_unknown() {
    assert_eq!(pvc_status(&PersistentVolumeClaim::default()), "Unknown");
}

#[test]
fn pvc_status_bound() {
    let pvc = PersistentVolumeClaim {
        status: Some(PersistentVolumeClaimStatus {
            phase: Some("Bound".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pvc_status(&pvc), "Bound");
}

#[test]
fn pvc_status_pending() {
    let pvc = PersistentVolumeClaim {
        status: Some(PersistentVolumeClaimStatus {
            phase: Some("Pending".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pvc_status(&pvc), "Pending");
}

#[test]
fn pvc_status_lost() {
    let pvc = PersistentVolumeClaim {
        status: Some(PersistentVolumeClaimStatus {
            phase: Some("Lost".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(pvc_status(&pvc), "Lost");
}

// ── job_status ────────────────────────────────────────────────────────────────

#[test]
fn job_status_no_status_returns_unknown() {
    assert_eq!(job_status(&Job::default()), "Unknown");
}

#[test]
fn job_status_complete_when_completion_time_set() {
    let job = Job {
        status: Some(JobStatus {
            completion_time: Some(serde_json::from_str(r#""2024-01-01T00:00:00Z""#).unwrap()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(job_status(&job), "Complete");
}

#[test]
fn job_status_failed_with_nonzero_failures() {
    let job = Job {
        status: Some(JobStatus {
            failed: Some(2),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(job_status(&job), "Failed(2)");
}

#[test]
fn job_status_active_when_running() {
    let job = Job {
        status: Some(JobStatus {
            active: Some(3),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(job_status(&job), "Active(3)");
}

#[test]
fn job_status_zero_failures_falls_through_to_active_check() {
    // failed=0 is not >0, so it skips the failed branch
    let job = Job {
        status: Some(JobStatus {
            failed: Some(0),
            active: Some(1),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(job_status(&job), "Active(1)");
}

// ── cronjob_status ────────────────────────────────────────────────────────────

#[test]
fn cronjob_status_defaults_to_scheduled() {
    assert_eq!(cronjob_status(&CronJob::default()), "Scheduled");
}

#[test]
fn cronjob_status_active_when_jobs_running() {
    let cj = CronJob {
        status: Some(CronJobStatus {
            active: Some(vec![ObjectReference::default(), ObjectReference::default()]),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(cronjob_status(&cj), "Active(2)");
}

#[test]
fn cronjob_status_empty_active_list_returns_scheduled() {
    let cj = CronJob {
        status: Some(CronJobStatus {
            active: Some(vec![]),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(cronjob_status(&cj), "Scheduled");
}
