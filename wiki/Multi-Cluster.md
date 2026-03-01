# Multi-Cluster

![multicluster demo](https://raw.githubusercontent.com/syedazeez337/kuberift/master/docs/media/multicluster.gif)

kf has two multi-cluster modes: watching all contexts simultaneously and switching interactively.

---

## Watch all clusters — `--all-contexts`

```bash
kf --all-contexts
kf --all-contexts pods
kf --all-contexts -n staging
```

kf connects to every context in your kubeconfig in parallel and streams resources from all of them into a single list. Each item is prefixed with its cluster name:

```
pod   prod-cluster/default/api-server-7d9f     CrashLoopBackOff   2h
pod   staging/default/api-server-5c2a          Running            5m
pod   prod-cluster/kube-system/coredns         Running            7d
```

Each cluster gets a distinct color so items are immediately identifiable at a glance.

All actions — describe, logs, exec, delete, port-forward, rollout restart — work correctly in all-contexts mode. kf automatically passes `--context <cluster>` to every `kubectl` invocation.

---

## Interactive context switching — `ctrl-x`

Press `ctrl-x` while kf is running to open a secondary fuzzy picker showing all contexts in your kubeconfig. Select one and kf restarts the resource stream from that cluster without you having to quit and relaunch.

The selected context is saved to `~/.config/kuberift/last_context` and restored automatically on the next launch.

---

## Explicit context — `--context`

```bash
kf --context my-prod-cluster
kf pods --context staging --namespace payments
```

`--context` overrides both the kubeconfig `current-context` and the saved last context. Useful in scripts or when you always want a specific cluster regardless of what you last used interactively.

---

## Context persistence

kf remembers the last context you switched to via `ctrl-x` and restores it on the next launch. To reset to the kubeconfig default, delete the saved file:

```bash
rm ~/.config/kuberift/last_context
```

---

## Kubeconfig location

```bash
# Default: ~/.kube/config or $KUBECONFIG
kf

# Explicit alternate file
kf --kubeconfig ~/my-other.yaml --context staging
```
