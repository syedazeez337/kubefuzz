# Actions

kf runs all actions by invoking `kubectl` directly — it passes the correct `--context`, `-n`, and resource arguments automatically.

![actions demo](https://raw.githubusercontent.com/syedazeez337/kubefuzz/master/docs/actions.gif)

---

## Describe — `enter`

Runs `kubectl describe <kind> <name>` and prints the output to your terminal. Works on all resource types. Supports multi-select: press `tab` to mark several resources, then `enter` to describe them all in sequence.

---

## Stream logs — `ctrl-l`

Runs `kubectl logs --tail=200 <pod>` and streams the output to your terminal. Pods only.

kf relaunches the TUI automatically after logs finish so you can immediately select another resource.

---

## Exec into shell — `ctrl-e`

Runs `kubectl exec -it <pod> -- /bin/sh` (falls back to `/bin/bash` if sh is not present) and drops you into an interactive shell inside the container.

Type `exit` or press `ctrl-d` to return to kf.

> Disabled in `--read-only` mode.

---

## Delete — `ctrl-d`

![delete demo](https://raw.githubusercontent.com/syedazeez337/kubefuzz/master/docs/delete.gif)

Shows a confirmation prompt before deleting anything:

```
  • pod/pod-crashloop [ns/testbed]

Delete 1 resource? [y/N]
```

- **Single or small batch (≤10)**: type `y` to confirm, anything else cancels.
- **Large batch (>10)**: kf requires typing `yes` in full — a single `y` is not accepted.

Multi-select with `tab` first to delete several resources at once.

> Disabled in `--read-only` mode.

---

## Port-forward — `ctrl-f`

Works on pods and services. kf prompts for a local port and a remote port, then runs `kubectl port-forward`:

```
Local port: 8080
Remote port [8080]: 80
Forwarding localhost:8080 → svc/my-service port 80  (Ctrl-C to stop)
```

- Ports below 1024 show a warning (`may require root/admin`) but are not blocked.
- Port 0 is rejected.
- Selecting a non-pod/service resource shows an informational error and does nothing.

Press `ctrl-c` to stop the forward and return to kf.

> Disabled in `--read-only` mode.

---

## Rollout restart — `ctrl-r`

Runs `kubectl rollout restart <deploy|sts|ds>/<name>` and then immediately tracks `kubectl rollout status` until the rollout completes.

```
↺ restarting deploy/api-server
Waiting for deployment "api-server" rollout to finish: 0 out of 2 new replicas have been updated...
deployment "api-server" successfully rolled out
```

Works on: Deployments, StatefulSets, DaemonSets. Other resource types show an informational message and are skipped.

> Disabled in `--read-only` mode.

---

## Print YAML — `ctrl-y`

Runs `kubectl get <kind> <name> -o yaml` and prints the full manifest to stdout. Supports multi-select.

Useful for piping to a file or a diff tool:

```bash
# in one terminal
kf
# press ctrl-y on a deployment, then:
# kf prints the YAML and relaunches the TUI
```

---

## Multi-select

Press `tab` on any item to toggle its selection (marked `>>`). All actions that support multi-select will act on every selected item in sequence.

The header shows the current match count (`77/77`) — selected items are indicated by `>>` in the list.
