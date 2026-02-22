use clap::Parser;

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
}

impl Args {
    /// Parse the resource argument into a list of ResourceKind to stream.
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
