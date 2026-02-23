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
    /// The cluster context this resource belongs to (empty in single-cluster mode)
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
            "Unknown" | "[DELETED]" => Color::DarkGray,
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

    /// Machine-parseable output string for piping.
    /// In multi-cluster mode, prefixed with the context: "ctx:kind/ns/name"
    pub fn output_str(&self) -> String {
        let loc = if self.namespace.is_empty() {
            format!("{}/{}", self.kind.as_str(), self.name)
        } else {
            format!("{}/{}/{}", self.kind.as_str(), self.namespace, self.name)
        };
        if self.context.is_empty() {
            loc
        } else {
            format!("{}:{}", self.context, loc)
        }
    }
}

/// Pick a consistent color for a cluster context name based on a hash of the name.
/// Ensures the same context always gets the same color across all items.
fn context_color(ctx: &str) -> Color {
    const PALETTE: &[Color] = &[
        Color::Cyan,
        Color::Magenta,
        Color::Yellow,
        Color::LightGreen,
        Color::LightBlue,
        Color::LightRed,
        Color::LightCyan,
        Color::LightMagenta,
    ];
    let hash: usize = ctx.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize));
    PALETTE[hash % PALETTE.len()]
}

impl SkimItem for K8sItem {
    /// The text skim fuzzy-matches against — plain, no color.
    /// In multi-cluster mode the context name is included so users can search by cluster.
    fn text(&self) -> Cow<'_, str> {
        let ctx_prefix = if self.context.is_empty() {
            String::new()
        } else {
            format!("{}/", self.context)
        };
        let ns_prefix = if self.namespace.is_empty() {
            String::new()
        } else {
            format!("{}/", self.namespace)
        };
        Cow::Owned(format!(
            "{:<8} {}{}{} {} {}",
            self.kind.as_str(),
            ctx_prefix,
            ns_prefix,
            self.name,
            self.status,
            self.age,
        ))
    }

    /// Colored display shown in the skim list.
    /// In multi-cluster mode a context prefix is shown before the namespace/name,
    /// colored distinctly per cluster.
    fn display<'a>(&'a self, _context: DisplayContext) -> Line<'a> {
        let ns_prefix = if self.namespace.is_empty() {
            String::new()
        } else {
            format!("{}/", self.namespace)
        };

        let mut spans = vec![Span::styled(
            format!("{:<8} ", self.kind.as_str()),
            Style::default().fg(self.kind.color()),
        )];

        // Context prefix — only shown in multi-cluster mode
        if !self.context.is_empty() {
            spans.push(Span::styled(
                format!("{}/", self.context),
                Style::default().fg(context_color(&self.context)),
            ));
        }

        spans.push(Span::styled(ns_prefix, Style::default().fg(Color::Cyan)));
        spans.push(Span::styled(
            format!("{:<48} ", self.name),
            Style::default().fg(Color::White),
        ));
        spans.push(Span::styled(
            format!("{:<22} ", self.status),
            Style::default().fg(self.status_color()),
        ));
        spans.push(Span::styled(
            self.age.clone(),
            Style::default().fg(Color::DarkGray),
        ));

        Line::from(spans)
    }

    /// Preview pane content — mode cycles via ctrl-p (describe → yaml → logs).
    /// Passes --context when the item belongs to a non-default cluster.
    /// Skim calls this from a background thread; blocking is fine here.
    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let mode = crate::actions::current_preview_mode();

        // Build the kubectl argument list for the current preview mode.
        // Logs mode uses a different argument structure (no kind prefix).
        let mut args: Vec<&str> = if mode == 2 && matches!(self.kind, ResourceKind::Pod) {
            vec!["logs", "--tail=100", &self.name]
        } else {
            match mode {
                1 => vec!["get", self.kind.as_str(), &self.name, "-o", "yaml"],
                _ => vec!["describe", self.kind.as_str(), &self.name],
            }
        };

        if !self.namespace.is_empty() {
            args.push("-n");
            args.push(&self.namespace);
        }

        // Target the correct cluster in multi-context mode
        if !self.context.is_empty() {
            args.push("--context");
            args.push(&self.context);
        }

        match std::process::Command::new("kubectl").args(&args).output() {
            Ok(out) => {
                let header = match mode {
                    1 => format!("── YAML: {}/{} ──\n", self.kind.as_str(), self.name),
                    2 => format!("── LOGS: {} (last 100) ──\n", self.name),
                    _ => format!("── DESCRIBE: {}/{} ──\n", self.kind.as_str(), self.name),
                };
                let body = if out.status.success() {
                    String::from_utf8_lossy(&out.stdout).to_string()
                } else {
                    format!(
                        "[kubectl error]\n{}",
                        String::from_utf8_lossy(&out.stderr)
                    )
                };
                ItemPreview::AnsiText(format!("{header}{body}"))
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
