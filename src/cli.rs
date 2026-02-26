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
    /// ing, node, ns, pv, pvc, job, cronjob). Omit to show ALL resource types.
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

    /// Filter resources by a Kubernetes label selector.
    /// Accepts any expression valid for kubectl --selector
    /// (e.g. `app=backend`, `env in (prod,staging)`, `!canary`).
    #[arg(short = 'l', long, value_name = "SELECTOR")]
    pub label: Option<String>,

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
            "pv" | "persistentvolume" | "persistentvolumes" => {
                vec![ResourceKind::PersistentVolume]
            }
            "pvc" | "persistentvolumeclaim" | "persistentvolumeclaims" => {
                vec![ResourceKind::PersistentVolumeClaim]
            }
            "job" | "jobs" => vec![ResourceKind::Job],
            "cj" | "cronjob" | "cronjobs" => vec![ResourceKind::CronJob],
            _ => {
                eprintln!(
                    "[kubefuzz] Unknown resource type '{s}'. Showing all resources.\n\
                     Supported: pods, svc, deploy, sts, ds, cm, secret, ing, node, ns, pv, pvc, job, cronjob"
                );
                return None;
            }
        };
        Some(kinds)
    }
}
