use clap::Parser;
use clap_complete::Shell;

use crate::items::ResourceKind;

#[derive(Parser, Debug)]
#[command(
    name = "kf",
    about = "Fuzzy-first interactive Kubernetes resource navigator",
    version
)]
pub struct Args {
    /// Resource type to filter (pods/po, svc, deploy, sts, ds, cm, secret,
    /// ing, node, ns, pvc, job, cronjob). Omit to show ALL resource types.
    pub resource: Option<String>,

    /// Watch resources from all kubeconfig contexts simultaneously.
    /// Resources are prefixed with their cluster context name.
    #[arg(long)]
    pub all_contexts: bool,

    /// Use a specific kubeconfig context instead of the current one.
    /// Overrides the last-used context saved by ctrl-x switching.
    #[arg(long, value_name = "CONTEXT")]
    pub context: Option<String>,

    /// Restrict to a specific namespace. Default: all namespaces.
    /// Cluster-scoped resources (Node, Namespace, PV) ignore this flag.
    #[arg(short = 'n', long, value_name = "NAMESPACE")]
    pub namespace: Option<String>,

    /// Disable all write and exec actions (delete, exec, port-forward, rollout-restart).
    /// Describe, logs, and YAML remain available.
    #[arg(long)]
    pub read_only: bool,

    /// Path to kubeconfig file. Defaults to $KUBECONFIG or ~/.kube/config.
    #[arg(long, value_name = "PATH")]
    pub kubeconfig: Option<String>,

    /// Print shell completions for SHELL to stdout and exit.
    /// Example: `kf --completions bash >> ~/.bash_completion`
    #[arg(long, value_name = "SHELL", hide = true)]
    pub completions: Option<Shell>,

    /// Print the man page to stdout and exit.
    /// Example: `kf --mangen | gzip > /usr/share/man/man1/kf.1.gz`
    #[arg(long, hide = true)]
    pub mangen: bool,
}

impl Args {
    /// Parse the resource argument into a list of `ResourceKind` to stream.
    /// Returns None when the argument is absent (meaning: stream everything).
    pub fn resource_filter(&self) -> Option<Vec<ResourceKind>> {
        let s = self.resource.as_deref()?.to_lowercase();
        let kinds = match s.as_str() {
            "pod" | "pods" | "po" => vec![ResourceKind::Pod],
            "svc" | "service" | "services" => vec![ResourceKind::Service],
            "deploy" | "deployment" | "deployments" => vec![ResourceKind::Deployment],
            "sts" | "statefulset" | "statefulsets" => vec![ResourceKind::StatefulSet],
            "ds" | "daemonset" | "daemonsets" => vec![ResourceKind::DaemonSet],
            "cm" | "configmap" | "configmaps" => vec![ResourceKind::ConfigMap],
            "secret" | "secrets" => vec![ResourceKind::Secret],
            "ing" | "ingress" | "ingresses" => vec![ResourceKind::Ingress],
            "node" | "nodes" | "no" => vec![ResourceKind::Node],
            "ns" | "namespace" | "namespaces" => vec![ResourceKind::Namespace],
            "pvc" | "persistentvolumeclaim" | "persistentvolumeclaims" => {
                vec![ResourceKind::PersistentVolumeClaim]
            }
            "job" | "jobs" => vec![ResourceKind::Job],
            "cj" | "cronjob" | "cronjobs" => vec![ResourceKind::CronJob],
            _ => {
                eprintln!(
                    "[kubefuzz] Unknown resource type '{s}'. Showing all resources.\n\
                     Supported: pods, svc, deploy, sts, ds, cm, secret, ing, node, ns, pvc, job, cronjob"
                );
                return None;
            }
        };
        Some(kinds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args_with_resource(r: &str) -> Args {
        Args {
            resource: Some(r.to_string()),
            all_contexts: false,
            context: None,
            namespace: None,
            read_only: false,
            kubeconfig: None,
            completions: None,
            mangen: false,
        }
    }

    #[test]
    fn filter_pod_aliases() {
        for alias in &["pod", "pods", "po"] {
            let args = args_with_resource(alias);
            let kinds = args.resource_filter().expect("should resolve");
            assert_eq!(
                kinds,
                vec![ResourceKind::Pod],
                "alias '{alias}' should map to Pod"
            );
        }
    }

    #[test]
    fn filter_service_aliases() {
        for alias in &["svc", "service", "services"] {
            let args = args_with_resource(alias);
            let kinds = args.resource_filter().expect("should resolve");
            assert_eq!(kinds, vec![ResourceKind::Service]);
        }
    }

    #[test]
    fn filter_deploy_aliases() {
        for alias in &["deploy", "deployment", "deployments"] {
            let args = args_with_resource(alias);
            let kinds = args.resource_filter().expect("should resolve");
            assert_eq!(kinds, vec![ResourceKind::Deployment]);
        }
    }

    #[test]
    fn filter_all_other_aliases() {
        let cases = vec![
            ("sts", ResourceKind::StatefulSet),
            ("ds", ResourceKind::DaemonSet),
            ("cm", ResourceKind::ConfigMap),
            ("secret", ResourceKind::Secret),
            ("ing", ResourceKind::Ingress),
            ("node", ResourceKind::Node),
            ("ns", ResourceKind::Namespace),
            ("pvc", ResourceKind::PersistentVolumeClaim),
            ("job", ResourceKind::Job),
            ("cj", ResourceKind::CronJob),
        ];
        for (alias, expected) in cases {
            let args = args_with_resource(alias);
            let kinds = args
                .resource_filter()
                .unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
            assert_eq!(kinds, vec![expected], "alias '{alias}' mismatch");
        }
    }

    #[test]
    fn filter_unknown_returns_none() {
        let args = args_with_resource("unknowntype");
        assert!(args.resource_filter().is_none());
    }

    #[test]
    fn filter_case_insensitive() {
        let args = args_with_resource("PODS");
        let kinds = args
            .resource_filter()
            .expect("should resolve case-insensitive");
        assert_eq!(kinds, vec![ResourceKind::Pod]);
    }

    #[test]
    fn filter_none_when_no_resource() {
        let args = Args {
            resource: None,
            all_contexts: false,
            context: None,
            namespace: None,
            read_only: false,
            kubeconfig: None,
            completions: None,
            mangen: false,
        };
        assert!(args.resource_filter().is_none());
    }
}
