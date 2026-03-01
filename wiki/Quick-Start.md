# Quick Start

## 1. Install kf

See [Installation](Installation). The fastest path:

```bash
curl -sL https://github.com/syedazeez337/kuberift/releases/latest/download/kf-x86_64-linux.tar.gz \
  | tar xz && sudo mv kf /usr/local/bin/kf
```

## 2. Launch

```bash
kf
```

kf opens immediately. It connects to your current kubeconfig context and starts streaming all resource types. If no cluster is reachable it switches to demo mode automatically — you can still explore the full UI.

## 3. Navigate and filter

| What to do | How |
|---|---|
| Move the cursor | `↑` / `↓` arrow keys |
| Fuzzy filter in real time | Just start typing |
| Clear the filter | `ctrl-u` |
| Select multiple items | `tab` (toggles) |
| Quit | `esc` |

Unhealthy resources sort to the top automatically — `CrashLoopBackOff`, `Error`, and `ImagePullBackOff` are red and appear first without any filtering.

## 4. Run your first action

Navigate to any pod and press:

- **`enter`** — `kubectl describe` the pod (safe, read-only)
- **`ctrl-l`** — stream the last 200 lines of logs to your terminal
- **`ctrl-p`** — cycle the right-hand preview pane between describe / YAML / logs

## 5. Common launch patterns

```bash
# All resources, current context (default)
kf

# Pods only
kf pods

# Restrict to one namespace
kf -n production

# Use a specific context
kf --context staging

# Watch all clusters at once
kf --all-contexts

# Safe mode — disables delete, exec, port-forward, rollout restart
kf --read-only
```

Next: [Filtering](Filtering) | [Preview Modes](Preview-Modes) | [Actions](Actions)
