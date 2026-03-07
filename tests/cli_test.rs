//! Tests for kuberift::cli — Args::resource_filter alias resolution and config merge.

use kuberift::cli::Args;
use kuberift::config::Config;
use kuberift::items::ResourceKind;

// ── Helper ────────────────────────────────────────────────────────────────────

fn args_with(resource: &str) -> Args {
    Args {
        resource: Some(resource.to_string()),
        all_contexts: false,
        context: None,
        namespace: None,
        read_only: false,
        label: None,
        kubeconfig: None,
        completions: None,
        mangen: false,
        no_crds: false,
    }
}

fn no_resource_args() -> Args {
    Args {
        resource: None,
        all_contexts: false,
        context: None,
        namespace: None,
        read_only: false,
        label: None,
        kubeconfig: None,
        completions: None,
        mangen: false,
        no_crds: false,
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
        assert_eq!(
            kinds,
            vec![ResourceKind::Service],
            "alias '{alias}' → Service"
        );
    }
}

// ── Deployment aliases ────────────────────────────────────────────────────────

#[test]
fn filter_deploy_aliases() {
    for alias in &["deploy", "deployment", "deployments"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(
            kinds,
            vec![ResourceKind::Deployment],
            "alias '{alias}' → Deployment"
        );
    }
}

// ── StatefulSet aliases ───────────────────────────────────────────────────────

#[test]
fn filter_statefulset_aliases() {
    for alias in &["sts", "statefulset", "statefulsets"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(
            kinds,
            vec![ResourceKind::StatefulSet],
            "alias '{alias}' → StatefulSet"
        );
    }
}

// ── DaemonSet aliases ─────────────────────────────────────────────────────────

#[test]
fn filter_daemonset_aliases() {
    for alias in &["ds", "daemonset", "daemonsets"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(
            kinds,
            vec![ResourceKind::DaemonSet],
            "alias '{alias}' → DaemonSet"
        );
    }
}

// ── ConfigMap aliases ─────────────────────────────────────────────────────────

#[test]
fn filter_configmap_aliases() {
    for alias in &["cm", "configmap", "configmaps"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(
            kinds,
            vec![ResourceKind::ConfigMap],
            "alias '{alias}' → ConfigMap"
        );
    }
}

// ── Secret aliases ────────────────────────────────────────────────────────────

#[test]
fn filter_secret_aliases() {
    for alias in &["secret", "secrets"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(
            kinds,
            vec![ResourceKind::Secret],
            "alias '{alias}' → Secret"
        );
    }
}

// ── Ingress aliases ───────────────────────────────────────────────────────────

#[test]
fn filter_ingress_aliases() {
    for alias in &["ing", "ingress", "ingresses"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(
            kinds,
            vec![ResourceKind::Ingress],
            "alias '{alias}' → Ingress"
        );
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
        assert_eq!(
            kinds,
            vec![ResourceKind::Namespace],
            "alias '{alias}' → Namespace"
        );
    }
}

// ── PV aliases ────────────────────────────────────────────────────────────────

#[test]
fn filter_pv_aliases() {
    for alias in &["pv", "persistentvolume", "persistentvolumes"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert_eq!(
            kinds,
            vec![ResourceKind::PersistentVolume],
            "alias: {alias}"
        );
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
        assert_eq!(
            kinds,
            vec![ResourceKind::CronJob],
            "alias '{alias}' → CronJob"
        );
    }
}

// ── Unknown alias falls back to None ─────────────────────────────────────────

#[test]
fn filter_unknown_alias_returns_custom() {
    for alias in &["unknowntype", "replicaset", "hpa", "certificates"] {
        let kinds = args_with(alias)
            .resource_filter()
            .unwrap_or_else(|| panic!("alias '{alias}' should resolve to Custom"));
        assert_eq!(
            kinds,
            vec![ResourceKind::Custom(alias.to_lowercase())],
            "unknown alias '{alias}' should map to Custom"
        );
    }
}

// ── Case-insensitive matching ─────────────────────────────────────────────────

#[test]
fn filter_case_insensitive_pod() {
    assert_eq!(
        args_with("PODS")
            .resource_filter()
            .expect("PODS should resolve"),
        vec![ResourceKind::Pod]
    );
}

#[test]
fn filter_case_insensitive_deploy() {
    assert_eq!(
        args_with("DEPLOY")
            .resource_filter()
            .expect("DEPLOY should resolve"),
        vec![ResourceKind::Deployment]
    );
}

#[test]
fn filter_case_insensitive_mixed_case() {
    assert_eq!(
        args_with("PoD")
            .resource_filter()
            .expect("PoD should resolve"),
        vec![ResourceKind::Pod]
    );
    assert_eq!(
        args_with("Svc")
            .resource_filter()
            .expect("Svc should resolve"),
        vec![ResourceKind::Service]
    );
}

// ── Config merge tests ──────────────────────────────────────────────────────

#[test]
fn merge_config_fills_empty_args() {
    let config = kuberift::config::parse_config(
        r#"
        [general]
        default_namespace = "production"
        default_context = "staging"
        default_resource = "pods"
        read_only = true
        "#,
        std::path::Path::new("test.toml"),
    );
    let mut args = no_resource_args();
    args.merge_with_config(&config);
    assert_eq!(args.namespace.as_deref(), Some("production"));
    assert_eq!(args.context.as_deref(), Some("staging"));
    assert_eq!(args.resource.as_deref(), Some("pods"));
    assert!(args.read_only);
}

#[test]
fn merge_config_cli_overrides_config() {
    let config = kuberift::config::parse_config(
        r#"
        [general]
        default_namespace = "production"
        default_context = "staging"
        "#,
        std::path::Path::new("test.toml"),
    );
    let mut args = Args {
        namespace: Some("kube-system".to_string()),
        context: Some("my-cluster".to_string()),
        ..no_resource_args()
    };
    args.merge_with_config(&config);
    // CLI values should win
    assert_eq!(args.namespace.as_deref(), Some("kube-system"));
    assert_eq!(args.context.as_deref(), Some("my-cluster"));
}

#[test]
fn merge_empty_config_changes_nothing() {
    let config = Config::default();
    let mut args = no_resource_args();
    args.merge_with_config(&config);
    assert!(args.namespace.is_none());
    assert!(args.context.is_none());
    assert!(args.resource.is_none());
    assert!(!args.read_only);
}
