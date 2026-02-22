# KubeFuzz

> A fuzzy-first interactive Kubernetes resource navigator built on [skim](https://github.com/skim-rs/skim)

KubeFuzz makes navigating Kubernetes clusters as fast as typing. Fuzzy-search across every resource type (pods, services, deployments, configmaps, secrets…) across all namespaces simultaneously — with a live preview pane showing logs, YAML manifests, and events inline.

---

## Why KubeFuzz?

| Tool | Problem |
|---|---|
| `kubectl` raw | Verbose, requires knowing exact names/namespaces upfront |
| `k9s` | Powerful but steep learning curve; text-heavy; poor multi-cluster UX |
| `kubectx` / `kubens` | Narrow scope — only switches context/namespace |
| Lens / Aptakube | GUI-based, breaks terminal-native workflows |

KubeFuzz is the missing piece: a **keyboard-driven, fuzzy-search-native TUI** for developers and platform engineers who live in the terminal.

---

## Core Features (Planned)

- **Universal fuzzy search** — type anything, match pods/services/deployments/configmaps/secrets/nodes across all namespaces
- **Live preview pane** — logs, YAML manifest, describe output, events for the selected resource
- **Multi-cluster support** — switch contexts via fuzzy search, persist across sessions
- **Multi-select + bulk actions** — select multiple pods → restart/delete/port-forward all at once
- **`kubectl exec` drop-in** — select a pod, pick a container, get a shell instantly
- **Smart defaults** — failed/pending resources bubble to top automatically
- **Composable** — pipe output back to shell, scriptable via exit codes

---

## Tech Stack

| Layer | Choice | Reason |
|---|---|---|
| Language | Rust | Performance, safety, small binary; skim is Rust-native |
| Fuzzy engine | [skim](https://github.com/skim-rs/skim) (library) | Multi-select, preview pane, async streaming — MIT license |
| K8s API | [kube-rs](https://github.com/kube-rs/kube) | Idiomatic async Rust K8s client |
| Async runtime | Tokio | Required by kube-rs; skim supports async streaming |
| TUI rendering | Ratatui (via skim) | skim already uses ratatui internally |
| Config | TOML via `serde` + `dirs` crate | Follow XDG conventions |
| CLI args | `clap` | Standard Rust CLI arg parsing |

---

## Status

**Pre-development** — research and architecture phase complete.

See [`docs/`](./docs/) for full context:
- [`docs/RESEARCH.md`](./docs/RESEARCH.md) — Market research, competitive analysis, opportunity sizing
- [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) — Technical design, module breakdown, skim integration
- [`docs/ROADMAP.md`](./docs/ROADMAP.md) — MVP scope, phased feature plan
- [`docs/SKIM_NOTES.md`](./docs/SKIM_NOTES.md) — How skim works and how we use it as a library

---

## Quick Start (once built)

```bash
# Install
cargo install kubefuzz

# Run — fuzzy search all resources in current context
kf

# Multi-cluster mode
kf --all-contexts

# Filter to a specific resource type
kf pods
kf svc
kf deploy

# Keybindings (default)
# <tab>        multi-select
# <enter>      describe / open preview
# ctrl-l       view live logs
# ctrl-e       exec into container shell
# ctrl-d       delete selected resource(s)
# ctrl-f       port-forward
# ctrl-x       switch cluster context
```

---

## Contributing

See [`docs/CONTRIBUTING.md`](./docs/CONTRIBUTING.md). The project is in early design phase — architectural feedback welcome.

---

## License

MIT — same as skim, the underlying fuzzy engine.
