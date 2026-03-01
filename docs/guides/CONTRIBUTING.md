# Contributing to KubeRift

KubeRift is in early development. The best way to contribute right now is architectural feedback, testing with real clusters, and picking up roadmap items.

---

## Development Setup

```bash
# 1. Clone
git clone https://github.com/syedazeez337/kuberift.git
cd kuberift

# 2. Install Rust (stable toolchain)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# 3. Linux system deps (required for skim/ncurses)
# Ubuntu/Debian:
sudo apt install libncurses5-dev libncursesw5-dev pkg-config

# Fedora/RHEL:
sudo dnf install ncurses-devel pkgconf

# Arch Linux:
sudo pacman -S ncurses pkgconf

# 4. Build
cargo build

# 5. Run (needs a kubeconfig; falls back to demo mode automatically if none found)
cargo run

# 6. Tests
cargo test
```

---

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` — no warnings allowed
- Keep functions small and focused
- Prefer `anyhow::Result` for error propagation
- Use `thiserror` for library-facing error types

---

## Commit Convention

```
feat: add port-forward action
fix: handle pods with no namespace
docs: update skim integration notes
refactor: split items.rs into items/ module
test: add integration test for multi-select
chore: update skim to 0.16
```

---

## Current Priority

See `docs/guides/ROADMAP.md`. Phases 0–5 are complete. We are working on **Phase 6** (distribution and polish).

The most valuable contributions right now: pre-built release binaries, shell completions, and a demo GIF.
