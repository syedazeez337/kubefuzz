# KubeFuzz — Wiki

> Fuzzy-first interactive Kubernetes resource navigator

`kf` lets you fuzzy-search every resource across every namespace in your cluster from a single terminal window. Select one or many, then describe, exec into a shell, tail logs, delete, port-forward, restart, or dump YAML — all without typing a single `kubectl` command.

![demo](https://raw.githubusercontent.com/syedazeez337/kubefuzz/master/docs/demo.gif)

---

## Pages

| Page | What you'll learn |
|---|---|
| [Installation](Installation) | Cargo, pre-built binaries, Homebrew, AUR, shell completions |
| [Quick Start](Quick-Start) | Launch kf and run your first action in under two minutes |
| [Filtering](Filtering) | Filter by resource type, namespace, and fuzzy query |
| [Preview Modes](Preview-Modes) | Cycle the right-hand pane between describe, YAML, and logs |
| [Actions](Actions) | logs, exec, delete, port-forward, rollout restart, yaml |
| [Multi-Cluster](Multi-Cluster) | Watch all clusters at once; switch context interactively |
| [Read-Only Mode](Read-Only-Mode) | Lock kf to safe-read operations on production clusters |
| [Keybindings](Keybindings) | Full keybinding reference |
| [Demo Gallery](Demo-Gallery) | Animated GIFs for every major feature |

---

## Feature highlights

- **Unhealthy-first sort** — `CrashLoopBackOff`, `Error`, `ImagePullBackOff` pods surface to the top automatically; no searching required.
- **Live watch** — resources appear, update, and disappear in real time as the cluster changes.
- **Preview pane** — inline `describe` / YAML / logs, cycled with `ctrl-p`, without leaving the list.
- **Multi-select** — `tab` to select any number of resources, then act on all of them at once.
- **Multi-cluster** — stream all contexts simultaneously with `--all-contexts`, or switch interactively with `ctrl-x`.
- **Demo mode** — works without a cluster; shows sample resources so you can explore the UI anywhere.

---

## Quick reference

```
kf                          # all resource types, current context
kf pods                     # pods only
kf -n production            # namespace filter
kf --all-contexts           # every cluster at once
kf --context staging        # specific context
kf --read-only              # disable all write/exec actions
```

