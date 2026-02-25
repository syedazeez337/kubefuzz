# Installation

## Requirements

- `kubectl` in your `$PATH` — used at runtime for all actions (describe, logs, exec, delete, etc.)
- A kubeconfig at `~/.kube/config` or `$KUBECONFIG` — optional; kf falls back to demo mode if absent

---

## Option 1 — Pre-built binary (recommended)

Download the binary for your platform from the [latest release](https://github.com/syedazeez337/kubefuzz/releases/latest), extract, and place `kf` somewhere on your `$PATH`.

| Platform | File |
|---|---|
| Linux x86\_64 | `kf-x86_64-linux.tar.gz` |
| Linux arm64 | `kf-aarch64-linux.tar.gz` |
| macOS Intel | `kf-x86_64-macos.tar.gz` |
| macOS Apple Silicon | `kf-aarch64-macos.tar.gz` |

Each archive contains the `kf` binary, shell completions (`bash`, `zsh`, `fish`), and a man page.

```bash
# Example: Linux x86_64
curl -sL https://github.com/syedazeez337/kubefuzz/releases/latest/download/kf-x86_64-linux.tar.gz \
  | tar xz
sudo mv kf /usr/local/bin/kf
```

---

## Option 2 — Homebrew (macOS / Linux)

```bash
brew install --formula https://raw.githubusercontent.com/syedazeez337/kubefuzz/master/contrib/kf.rb
```

Or via the tap (once a `homebrew-kubefuzz` tap repo exists):

```bash
brew tap syedazeez337/kubefuzz https://github.com/syedazeez337/kubefuzz
brew install kf
```

---

## Option 3 — AUR (Arch Linux)

```bash
# Using an AUR helper
yay -S kubefuzz

# Manually from contrib/PKGBUILD
git clone https://github.com/syedazeez337/kubefuzz.git
cd kubefuzz
makepkg -si contrib/PKGBUILD
```

---

## Option 4 — Build from source

```bash
# Requires Rust stable toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

git clone https://github.com/syedazeez337/kubefuzz.git
cd kubefuzz
cargo build --release
sudo mv target/release/kf /usr/local/bin/kf
```

---

## Option 5 — Windows (WSL)

Install the Linux x86\_64 binary inside your WSL environment. All kf features work normally in WSL2 terminal windows — kubectl, shell completions, and the TUI all behave identically to native Linux.

```bash
# Inside WSL
curl -sL https://github.com/syedazeez337/kubefuzz/releases/latest/download/kf-x86_64-linux.tar.gz \
  | tar xz
sudo mv kf /usr/local/bin/kf
```

---

## Shell completions

Completions are included in every release tarball. To install manually:

```bash
# bash — add to ~/.bashrc or drop in /etc/bash_completion.d/
kf --completions bash >> ~/.bash_completion

# zsh — place in a directory on $fpath
kf --completions zsh > ~/.zsh/completions/_kf

# fish
kf --completions fish > ~/.config/fish/completions/kf.fish
```

---

## Man page

```bash
kf --mangen | sudo tee /usr/share/man/man1/kf.1
# then: man kf
```

---

## Verify

```bash
kf --version   # kf 0.1.0
kf --help
```
