# Preview Modes

![preview demo](https://raw.githubusercontent.com/syedazeez337/kubefuzz/master/docs/preview.gif)

The right-hand pane updates live as you move the cursor. Press **`ctrl-p`** to cycle through three modes.

---

## Mode 0 — Describe (default)

Runs `kubectl describe <kind> <name> -n <namespace>` and shows the full output inline.

Useful for:
- Viewing Events at the bottom of the describe output (especially for crashing or stuck pods)
- Checking resource limits, labels, and annotations
- Seeing the current replica count and selector for deployments

---

## Mode 1 — YAML

Runs `kubectl get <kind> <name> -n <namespace> -o yaml` and shows the full manifest.

Useful for:
- Inspecting the full spec before editing
- Verifying labels, annotations, and owner references
- Copying the manifest to a local file for comparison

---

## Mode 2 — Logs

Runs `kubectl logs --tail=100 <pod> -n <namespace>` and shows the last 100 lines.

Only available for **pods**. For non-pod resources (deployments, services, etc.) the logs pane shows an empty header — switch back to describe or YAML with another `ctrl-p`.

---

## Cycling

```
ctrl-p → describe → yaml → logs → describe → ...
```

The mode persists as you move the cursor. Navigate to different resources and the same mode is applied to each one.

---

## Tips

- **Crashing pod**: start in **describe** mode — the Events section at the bottom shows the exact back-off reason and container exit code.
- **YAML diff**: switch to **YAML** mode and press `ctrl-y` (`enter` or action key) to dump the manifest to stdout where you can pipe it to `diff`.
- **Logs side-by-side**: while in **logs** mode, move the cursor between pods to compare their logs without leaving kf.
