# Filtering

![filter demo](https://raw.githubusercontent.com/syedazeez337/kuberift/master/docs/filter.gif)

kf supports three independent levels of filtering that compose freely.

---

## 1. Resource type filter (CLI argument)

Pass a resource type as the first argument to restrict the stream to that type only:

```bash
kf pods          # or: pod, po
kf deploy        # or: deployment, deployments
kf svc           # or: service, services
kf sts           # or: statefulset, statefulsets
kf ds            # or: daemonset, daemonsets
kf cm            # or: configmap, configmaps
kf secret        # or: secrets
kf ing           # or: ingress, ingresses
kf node          # or: nodes, no
kf ns            # or: namespace, namespaces
kf pvc           # or: persistentvolumeclaim
kf job           # or: jobs
kf cj            # or: cronjob, cronjobs
```

Omit the argument to stream **all** resource types simultaneously (the default).

Unknown types fall back to all-resources with a warning — kf never exits on a bad argument.

---

## 2. Namespace filter (`-n`)

```bash
kf -n production
kf pods -n kube-system
kf -n staging --context my-cluster
```

Cluster-scoped resources (Node, Namespace, PersistentVolume) are always visible regardless of `-n` — they appear dimmed when a namespace filter is active.

---

## 3. Fuzzy query (live, inside the TUI)

Once kf is open, **just start typing** to narrow the list in real time. The query matches against the full display string: kind, namespace, name, status, and age are all searchable.

| Tip | Example query |
|---|---|
| Find a pod by partial name | `api-server` |
| Find all failing resources | `error` or `crash` |
| Find resources in one namespace | `production` |
| Find a specific status | `imagepull` |
| Combine terms | `prod crash` |

Press `ctrl-u` to clear the query and return to the full list.

---

## Combining all three

```bash
kf pods -n production
```

Opens with only pods from the `production` namespace pre-loaded. Type in the TUI to further narrow by name or status.

---

## Unhealthy-first sort

Regardless of any filter, resources are always sorted by health priority:

| Priority | Color | Statuses |
|---|---|---|
| 0 — Critical | Red | `CrashLoopBackOff`, `Error`, `ImagePullBackOff`, `OOMKilled`, `Failed`, `Evicted` |
| 1 — Warning | Yellow | `Pending`, `Terminating`, `Init:0/1`, `ContainerCreating` |
| 2 — Healthy | Green | `Running`, `Succeeded`, `Active`, `Bound`, `ClusterIP` |
| — | Dim | `[DELETED]` |

Critical resources always appear at the top — no filter needed to find what is broken.
