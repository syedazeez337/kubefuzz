use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use skim::{DisplayContext, ItemPreview, PreviewContext, SkimItem};
use std::borrow::Cow;

/// The kind of Kubernetes resource this item represents
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceKind {
    Pod,
    Service,
    Deployment,
    StatefulSet,
    DaemonSet,
    ConfigMap,
    Secret,
    Ingress,
    Node,
    Namespace,
    PersistentVolumeClaim,
    Job,
    CronJob,
}

impl ResourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pod => "pod",
            Self::Service => "svc",
            Self::Deployment => "deploy",
            Self::StatefulSet => "sts",
            Self::DaemonSet => "ds",
            Self::ConfigMap => "cm",
            Self::Secret => "secret",
            Self::Ingress => "ing",
            Self::Node => "node",
            Self::Namespace => "ns",
            Self::PersistentVolumeClaim => "pvc",
            Self::Job => "job",
            Self::CronJob => "cronjob",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Pod => Color::Green,
            Self::Service => Color::Blue,
            Self::Deployment | Self::StatefulSet | Self::DaemonSet => Color::Yellow,
            Self::ConfigMap | Self::Secret => Color::Magenta,
            Self::Ingress => Color::Cyan,
            Self::Node | Self::Namespace => Color::White,
            Self::PersistentVolumeClaim => Color::LightMagenta,
            Self::Job | Self::CronJob => Color::LightBlue,
        }
    }
}

/// A Kubernetes resource item displayed in the skim TUI
#[derive(Debug, Clone)]
pub struct K8sItem {
    pub kind: ResourceKind,
    pub namespace: String,
    pub name: String,
    pub status: String,
    pub age: String,
    /// The cluster context this resource belongs to
    pub context: String,
}

impl K8sItem {
    pub fn new(
        kind: ResourceKind,
        namespace: impl Into<String>,
        name: impl Into<String>,
        status: impl Into<String>,
        age: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            namespace: namespace.into(),
            name: name.into(),
            status: status.into(),
            age: age.into(),
            context: String::new(),
        }
    }

    /// Color the status string based on health
    pub fn status_color(&self) -> Color {
        match self.status.as_str() {
            "Running" | "Active" | "Bound" | "Complete" | "Succeeded" | "Ready"
            | "Scheduled" | "ClusterIP" | "NodePort" | "LoadBalancer" => Color::Green,
            "Pending" | "Terminating" | "ContainerCreating" => Color::Yellow,
            "Failed" | "Error" | "OOMKilled" | "NotReady" | "Lost" => Color::Red,
            "Unknown" => Color::DarkGray,
            s if s.starts_with("CrashLoop")
                || s.starts_with("ErrImage")
                || s.starts_with("ImagePull")
                || s.starts_with("Init:Error")
                || s.starts_with("Failed(") =>
            {
                Color::Red
            }
            s if s.starts_with("Init:") => Color::Yellow,
            s if s.starts_with("Active(") => Color::Green,
            s if s.contains('/') => {
                // e.g. "3/3" (ready/desired) — green if equal, yellow if not
                let parts: Vec<&str> = s.splitn(2, '/').collect();
                if parts[0] == parts[1] {
                    Color::Green
                } else {
                    Color::Yellow
                }
            }
            _ => Color::White,
        }
    }

    /// Machine-parseable output string for piping
    pub fn output_str(&self) -> String {
        if self.namespace.is_empty() {
            format!("{}/{}", self.kind.as_str(), self.name)
        } else {
            format!("{}/{}/{}", self.kind.as_str(), self.namespace, self.name)
        }
    }
}

impl SkimItem for K8sItem {
    /// The text skim fuzzy-matches against — plain, no color
    fn text(&self) -> Cow<'_, str> {
        let ns_prefix = if self.namespace.is_empty() {
            String::new()
        } else {
            format!("{}/", self.namespace)
        };
        Cow::Owned(format!(
            "{:<8} {}{} {} {}",
            self.kind.as_str(),
            ns_prefix,
            self.name,
            self.status,
            self.age,
        ))
    }

    /// Colored display shown in the skim list
    fn display<'a>(&'a self, _context: DisplayContext) -> Line<'a> {
        let ns_prefix = if self.namespace.is_empty() {
            String::new()
        } else {
            format!("{}/", self.namespace)
        };

        Line::from(vec![
            Span::styled(
                format!("{:<8} ", self.kind.as_str()),
                Style::default().fg(self.kind.color()),
            ),
            Span::styled(ns_prefix, Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{:<48} ", self.name),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("{:<22} ", self.status),
                Style::default().fg(self.status_color()),
            ),
            Span::styled(self.age.clone(), Style::default().fg(Color::DarkGray)),
        ])
    }

    /// Preview pane content for the hovered item — calls kubectl describe synchronously.
    /// Skim invokes this from a background thread, so blocking is fine here.
    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let mut args = vec!["describe", self.kind.as_str(), &self.name];
        if !self.namespace.is_empty() {
            args.extend_from_slice(&["-n", &self.namespace]);
        }

        match std::process::Command::new("kubectl").args(&args).output() {
            Ok(out) => {
                let text = if out.status.success() {
                    String::from_utf8_lossy(&out.stdout).to_string()
                } else {
                    format!(
                        "[kubectl error]\n{}",
                        String::from_utf8_lossy(&out.stderr)
                    )
                };
                // AnsiText preserves kubectl's color output
                ItemPreview::AnsiText(text)
            }
            Err(e) => ItemPreview::Text(format!(
                "[Error running kubectl]\n{e}\n\nIs kubectl in your PATH?"
            )),
        }
    }

    /// What gets written to stdout when this item is selected
    fn output(&self) -> Cow<'_, str> {
        Cow::Owned(self.output_str())
    }
}
