//! Tests for kubefuzz::cli — Args::resource_filter alias resolution.

use kubefuzz::cli::Args;
use kubefuzz::items::ResourceKind;

// ── Helper ────────────────────────────────────────────────────────────────────

fn args_with(resource: &str) -> Args {
    Args {
        resource: Some(resource.to_string()),
        all_contexts: false,
        context: None,
        namespace: None,
        read_only: false,
        kubeconfig: None,
        completions: None,
        mangen: false,
    }
}

fn no_resource_args() -> Args {
    Args {
        resource: None,
        all_contexts: false,
        context: None,
        namespace: None,
        read_only: false,
        kubeconfig: None,
        completions: None,
        mangen: false,
    }
}

// ── None when no resource argument ───────────────────────────────────────────

#[test]
fn filter_none_when_resource_arg_absent() {
    assert!(no_resource_args().resource_filter().is_none());
}

// ── Pod aliases ───────────────────────────────────────────────────────────────

#[test]
fn filter_pod_aliases() {
    for alias in &["pod", "pods", "po"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::Pod], "alias '{alias}' → Pod");
    }
}

// ── Service aliases ───────────────────────────────────────────────────────────

#[test]
fn filter_service_aliases() {
    for alias in &["svc", "service", "services"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::Service], "alias '{alias}' → Service");
    }
}

// ── Deployment aliases ────────────────────────────────────────────────────────

#[test]
fn filter_deploy_aliases() {
    for alias in &["deploy", "deployment", "deployments"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::Deployment], "alias '{alias}' → Deployment");
    }
}

// ── StatefulSet aliases ───────────────────────────────────────────────────────

#[test]
fn filter_statefulset_aliases() {
    for alias in &["sts", "statefulset", "statefulsets"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::StatefulSet], "alias '{alias}' → StatefulSet");
    }
}

// ── DaemonSet aliases ─────────────────────────────────────────────────────────

#[test]
fn filter_daemonset_aliases() {
    for alias in &["ds", "daemonset", "daemonsets"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::DaemonSet], "alias '{alias}' → DaemonSet");
    }
}

// ── ConfigMap aliases ─────────────────────────────────────────────────────────

#[test]
fn filter_configmap_aliases() {
    for alias in &["cm", "configmap", "configmaps"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::ConfigMap], "alias '{alias}' → ConfigMap");
    }
}

// ── Secret aliases ────────────────────────────────────────────────────────────

#[test]
fn filter_secret_aliases() {
    for alias in &["secret", "secrets"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::Secret], "alias '{alias}' → Secret");
    }
}

// ── Ingress aliases ───────────────────────────────────────────────────────────

#[test]
fn filter_ingress_aliases() {
    for alias in &["ing", "ingress", "ingresses"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::Ingress], "alias '{alias}' → Ingress");
    }
}

// ── Node aliases ──────────────────────────────────────────────────────────────

#[test]
fn filter_node_aliases() {
    for alias in &["node", "nodes", "no"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::Node], "alias '{alias}' → Node");
    }
}

// ── Namespace aliases ─────────────────────────────────────────────────────────

#[test]
fn filter_namespace_aliases() {
    for alias in &["ns", "namespace", "namespaces"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::Namespace], "alias '{alias}' → Namespace");
    }
}

// ── PVC aliases ───────────────────────────────────────────────────────────────

#[test]
fn filter_pvc_aliases() {
    for alias in &["pvc", "persistentvolumeclaim", "persistentvolumeclaims"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(
            kinds,
            vec![ResourceKind::PersistentVolumeClaim],
            "alias '{alias}' → PersistentVolumeClaim"
        );
    }
}

// ── Job aliases ───────────────────────────────────────────────────────────────

#[test]
fn filter_job_aliases() {
    for alias in &["job", "jobs"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::Job], "alias '{alias}' → Job");
    }
}

// ── CronJob aliases ───────────────────────────────────────────────────────────

#[test]
fn filter_cronjob_aliases() {
    for alias in &["cj", "cronjob", "cronjobs"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(kinds, vec![ResourceKind::CronJob], "alias '{alias}' → CronJob");
    }
}

// ── Unknown alias falls back to None ─────────────────────────────────────────

#[test]
fn filter_unknown_alias_returns_none() {
    assert!(args_with("unknowntype").resource_filter().is_none());
    assert!(args_with("replicaset").resource_filter().is_none());
    assert!(args_with("hpa").resource_filter().is_none());
}

// ── Case-insensitive matching ─────────────────────────────────────────────────

#[test]
fn filter_case_insensitive_pod() {
    assert_eq!(
        args_with("PODS").resource_filter().expect("PODS should resolve"),
        vec![ResourceKind::Pod]
    );
}

#[test]
fn filter_case_insensitive_deploy() {
    assert_eq!(
        args_with("DEPLOY").resource_filter().expect("DEPLOY should resolve"),
        vec![ResourceKind::Deployment]
    );
}

#[test]
fn filter_case_insensitive_mixed_case() {
    assert_eq!(
        args_with("PoD").resource_filter().expect("PoD should resolve"),
        vec![ResourceKind::Pod]
    );
    assert_eq!(
        args_with("Svc").resource_filter().expect("Svc should resolve"),
        vec![ResourceKind::Service]
    );
}
