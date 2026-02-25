use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use skim::{DisplayContext, ItemPreview, PreviewContext, SkimItem};
use std::borrow::Cow;

// ─── Name truncation helper ───────────────────────────────────────────────────

/// Truncate `name` to at most `max_chars` bytes, appending "…" if truncated.
/// Always splits on a valid UTF-8 char boundary to avoid panics on multi-byte names.
fn truncate_name(name: &str, max_chars: usize) -> Cow<'_, str> {
    if name.len() <= max_chars {
        return Cow::Borrowed(name);
    }
    let mut end = max_chars;
    while !name.is_char_boundary(end) && end > 0 {
        end -= 1;
    }
    Cow::Owned(format!("{}…", &name[..end]))
}

// ─── Status health classification ────────────────────────────────────────────

/// Health category for a Kubernetes resource status string.
/// Single source of truth for both color and sort-priority decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusHealth {
    /// Needs immediate attention: `CrashLoopBackOff`, Error, Failed, Evicted, etc.
    Critical,
    /// Transitional state: Pending, Terminating, Init:X/Y, etc.
    Warning,
    /// Normal operation: Running, Active, Bound, Complete, etc.
    Healthy,
    /// Deleted or unrecognized.
    Unknown,
}

impl StatusHealth {
    /// Classify a status string into a health category.
    pub fn classify(status: &str) -> Self {
        match status {
            // ── Exact critical matches ────────────────────────────────────────
            "Failed" | "Error" | "OOMKilled" | "NotReady" | "Lost" | "Evicted" | "BackOff" => {
                Self::Critical
            }
            // ── Prefix-based critical matches ─────────────────────────────────
            s if s.starts_with("CrashLoop")
                || s.starts_with("ErrImage")
                || s.starts_with("ImagePull")
                || s.starts_with("Init:Error")
                || s.starts_with("Init:ErrImage")
                || s.starts_with("Init:ImagePull")
                || s.starts_with("Failed(") =>
            {
                Self::Critical
            }
            // ── Exact warning matches ─────────────────────────────────────────
            "Pending" | "Terminating" | "ContainerCreating" | "Unknown" => Self::Warning,
            // ── Prefix-based warning matches ──────────────────────────────────
            s if s.starts_with("Init:") => Self::Warning,
            // ── Deleted ───────────────────────────────────────────────────────
            "[DELETED]" => Self::Unknown,
            // ── Exact healthy matches ─────────────────────────────────────────
            "Running" | "Active" | "Bound" | "Complete" | "Succeeded" | "Ready" | "Scheduled"
            | "ClusterIP" | "NodePort" | "LoadBalancer" => Self::Healthy,
            // ── Prefix-based healthy ──────────────────────────────────────────
            s if s.starts_with("Active(") => Self::Healthy,
            // ── Ratio: "3/3" healthy, "1/3" warning ──────────────────────────
            s if s.contains('/') => {
                let parts: Vec<&str> = s.splitn(2, '/').collect();
                if parts.len() == 2 && parts[0] == parts[1] {
                    Self::Healthy
                } else {
                    Self::Warning
                }
            }
            // ── Unknown statuses default to healthy ───────────────────────────
            _ => Self::Healthy,
        }
    }

    /// Terminal color for this health category.
    pub fn color(self) -> Color {
        match self {
            Self::Critical => Color::Red,
            Self::Warning => Color::Yellow,
            Self::Healthy => Color::Green,
            Self::Unknown => Color::DarkGray,
        }
    }

    /// Sort priority: 0 = top of list (critical), 1 = middle, 2 = bottom (healthy).
    /// Skim renders higher-indexed items at the top, so lower priority = sent last.
    pub fn priority(self) -> u8 {
        match self {
            Self::Critical => 0,
            Self::Warning | Self::Unknown => 1,
            Self::Healthy => 2,
        }
    }
}

/// The kind of Kubernetes resource this item represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
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
    pub fn as_str(self) -> &'static str {
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

    pub fn color(self) -> Color {
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

impl std::fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A Kubernetes resource item displayed in the skim TUI.
#[derive(Debug, Clone)]
pub struct K8sItem {
    kind: ResourceKind,
    namespace: String,
    name: String,
    status: String,
    age: String,
    /// The cluster context this resource belongs to (empty in single-cluster mode).
    context: String,
}

impl K8sItem {
    pub fn new(
        kind: ResourceKind,
        namespace: impl Into<String>,
        name: impl Into<String>,
        status: impl Into<String>,
        age: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            namespace: namespace.into(),
            name: name.into(),
            status: status.into(),
            age: age.into(),
            context: context.into(),
        }
    }

    pub fn kind(&self) -> ResourceKind {
        self.kind
    }
    pub fn namespace(&self) -> &str {
        &self.namespace
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn status(&self) -> &str {
        &self.status
    }
    pub fn context(&self) -> &str {
        &self.context
    }

    /// Color the status string based on health — delegates to `StatusHealth`.
    pub fn status_color(&self) -> Color {
        StatusHealth::classify(&self.status).color()
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
    let hash: usize = ctx
        .bytes()
        .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
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
        let name_truncated = truncate_name(&self.name, 31);
        Cow::Owned(format!(
            "{:<8} {}{}{} {} {}",
            self.kind.as_str(),
            ctx_prefix,
            ns_prefix,
            name_truncated,
            self.status,
            self.age,
        ))
    }

    /// Colored display shown in the skim list.
    /// In multi-cluster mode a context prefix is shown before the namespace/name,
    /// colored distinctly per cluster.
    fn display(&self, _context: DisplayContext) -> Line<'_> {
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
        let name_col = {
            let t = truncate_name(&self.name, 31);
            if t.len() < 32 {
                format!("{t:<32} ")
            } else {
                format!("{t} ")
            }
        };
        spans.push(Span::styled(name_col, Style::default().fg(Color::White)));
        spans.push(Span::styled(
            format!("{:<17} ", self.status),
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
            vec!["logs", "--tail=100", "--", &self.name]
        } else {
            match mode {
                1 => vec!["get", self.kind.as_str(), "--", &self.name, "-o", "yaml"],
                _ => vec!["describe", self.kind.as_str(), "--", &self.name],
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
                    format!("[kubectl error]\n{}", String::from_utf8_lossy(&out.stderr))
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── ResourceKind::as_str ──────────────────────────────────────────────────

    #[test]
    fn kind_as_str_all_variants() {
        assert_eq!(ResourceKind::Pod.as_str(), "pod");
        assert_eq!(ResourceKind::Service.as_str(), "svc");
        assert_eq!(ResourceKind::Deployment.as_str(), "deploy");
        assert_eq!(ResourceKind::StatefulSet.as_str(), "sts");
        assert_eq!(ResourceKind::DaemonSet.as_str(), "ds");
        assert_eq!(ResourceKind::ConfigMap.as_str(), "cm");
        assert_eq!(ResourceKind::Secret.as_str(), "secret");
        assert_eq!(ResourceKind::Ingress.as_str(), "ing");
        assert_eq!(ResourceKind::Node.as_str(), "node");
        assert_eq!(ResourceKind::Namespace.as_str(), "ns");
        assert_eq!(ResourceKind::PersistentVolumeClaim.as_str(), "pvc");
        assert_eq!(ResourceKind::Job.as_str(), "job");
        assert_eq!(ResourceKind::CronJob.as_str(), "cronjob");
    }

    #[test]
    fn kind_display_matches_as_str() {
        assert_eq!(format!("{}", ResourceKind::Pod), "pod");
        assert_eq!(format!("{}", ResourceKind::Deployment), "deploy");
    }

    // ── StatusHealth::classify ────────────────────────────────────────────────

    #[test]
    fn status_health_critical_exact() {
        for s in &[
            "Failed",
            "Error",
            "OOMKilled",
            "NotReady",
            "Lost",
            "Evicted",
            "BackOff",
        ] {
            assert_eq!(
                StatusHealth::classify(s),
                StatusHealth::Critical,
                "status '{s}' should be Critical"
            );
        }
    }

    #[test]
    fn status_health_critical_prefix() {
        for s in &[
            "CrashLoopBackOff",
            "ErrImagePull",
            "ImagePullBackOff",
            "Init:ErrImagePull",
            "Init:Error",
            "Init:ImagePullBackOff",
            "Failed(3)",
        ] {
            assert_eq!(
                StatusHealth::classify(s),
                StatusHealth::Critical,
                "status '{s}' should be Critical"
            );
        }
    }

    #[test]
    fn status_health_warning() {
        for s in &[
            "Pending",
            "Terminating",
            "ContainerCreating",
            "Unknown",
            "Init:0/1",
            "Init:2/3",
        ] {
            assert_eq!(
                StatusHealth::classify(s),
                StatusHealth::Warning,
                "status '{s}' should be Warning"
            );
        }
    }

    #[test]
    fn status_health_deleted_is_unknown() {
        assert_eq!(StatusHealth::classify("[DELETED]"), StatusHealth::Unknown);
    }

    #[test]
    fn status_health_healthy_exact() {
        for s in &[
            "Running",
            "Active",
            "Bound",
            "Complete",
            "Succeeded",
            "Ready",
            "Scheduled",
            "ClusterIP",
            "NodePort",
            "LoadBalancer",
        ] {
            assert_eq!(
                StatusHealth::classify(s),
                StatusHealth::Healthy,
                "status '{s}' should be Healthy"
            );
        }
    }

    #[test]
    fn status_health_ratio_equal_is_healthy() {
        assert_eq!(StatusHealth::classify("3/3"), StatusHealth::Healthy);
        assert_eq!(StatusHealth::classify("1/1"), StatusHealth::Healthy);
    }

    #[test]
    fn status_health_ratio_unequal_is_warning() {
        assert_eq!(StatusHealth::classify("0/3"), StatusHealth::Warning);
        assert_eq!(StatusHealth::classify("2/3"), StatusHealth::Warning);
    }

    #[test]
    fn status_health_active_prefix_is_healthy() {
        assert_eq!(StatusHealth::classify("Active(2)"), StatusHealth::Healthy);
    }

    #[test]
    fn status_health_unknown_string_defaults_healthy() {
        assert_eq!(
            StatusHealth::classify("SomeUnknownStatus"),
            StatusHealth::Healthy
        );
    }

    // ── BUG-003: Terminating must remain in warning tier ──────────────────────

    #[test]
    fn terminating_is_warning_not_critical() {
        assert_eq!(StatusHealth::classify("Terminating"), StatusHealth::Warning);
        assert_eq!(StatusHealth::Warning.priority(), 1);
        assert_eq!(StatusHealth::Warning.color(), Color::Yellow);
    }

    // ── status_color ─────────────────────────────────────────────────────────

    #[test]
    fn status_color_running_is_green() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "p", "Running", "1d", "");
        assert_eq!(item.status_color(), Color::Green);
    }

    #[test]
    fn status_color_crashloop_is_red() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "p", "CrashLoopBackOff", "1d", "");
        assert_eq!(item.status_color(), Color::Red);
    }

    #[test]
    fn status_color_pending_is_yellow() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "p", "Pending", "1d", "");
        assert_eq!(item.status_color(), Color::Yellow);
    }

    #[test]
    fn status_color_deleted_is_gray() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "p", "[DELETED]", "1d", "");
        assert_eq!(item.status_color(), Color::DarkGray);
    }

    // ── output_str ───────────────────────────────────────────────────────────

    #[test]
    fn output_str_with_namespace() {
        let item = K8sItem::new(ResourceKind::Pod, "default", "nginx", "Running", "1d", "");
        assert_eq!(item.output_str(), "pod/default/nginx");
    }

    #[test]
    fn output_str_no_namespace() {
        let item = K8sItem::new(ResourceKind::Node, "", "node-1", "Ready", "7d", "");
        assert_eq!(item.output_str(), "node/node-1");
    }

    #[test]
    fn output_str_with_context() {
        let item = K8sItem::new(ResourceKind::Pod, "ns", "p", "Running", "1d", "prod");
        assert_eq!(item.output_str(), "prod:pod/ns/p");
    }

    #[test]
    fn output_str_no_namespace_with_context() {
        let item = K8sItem::new(
            ResourceKind::Namespace,
            "",
            "default",
            "Active",
            "30d",
            "prod",
        );
        assert_eq!(item.output_str(), "prod:ns/default");
    }

    // ── truncate_name ─────────────────────────────────────────────────────────

    #[test]
    fn truncate_short_name_unchanged() {
        assert_eq!(truncate_name("nginx", 31).as_ref(), "nginx");
    }

    #[test]
    fn truncate_exact_boundary_unchanged() {
        let name = "a".repeat(31);
        assert_eq!(truncate_name(&name, 31).as_ref(), name.as_str());
    }

    #[test]
    fn truncate_long_name_gets_ellipsis() {
        let name = "a".repeat(40);
        let result = truncate_name(&name, 31);
        assert!(result.contains('…'));
        assert!(result.len() <= 31 + '…'.len_utf8());
    }

    #[test]
    fn truncate_handles_multibyte_utf8() {
        // "é" is 2 bytes; raw &name[..31] would panic if byte 31 lands mid-char
        let name = format!("{}é{}", "a".repeat(30), "suffix");
        // Should not panic and should produce valid UTF-8
        let result = truncate_name(&name, 31);
        assert!(std::str::from_utf8(result.as_bytes()).is_ok());
    }

    // ── context_color ─────────────────────────────────────────────────────────

    #[test]
    fn context_color_is_deterministic() {
        assert_eq!(context_color("prod"), context_color("prod"));
    }

    #[test]
    fn context_color_does_not_panic_on_empty() {
        let _ = context_color("");
    }
}
