# KubeRift — Test Report

**Last updated**: 2026-02-25
**Binary**: `target/release/kf` (v0.1.0)
**Cluster**: `kind-kuberift-dev` (kind v1.32.0, single node)
**Test environment**: Linux (Fedora), kubectl v1.35.0

---

## Test Cluster Resources

The test cluster (`kind-kuberift-dev`) contains intentional diversity across all status types:

| Namespace | Resource | Status | Purpose |
|-----------|----------|--------|---------|
| testbed | pod-crashloop | CrashLoopBackOff | Tests red status + priority sort |
| testbed | pod-imagepull | ImagePullBackOff | Tests red status |
| testbed | pod-init-wait | Init:0/1 | Tests init container status |
| testbed | pod-running | Running | Tests green status, logs, exec |
| testbed | deploy-healthy | 2/2 | Tests green ratio status |
| testbed | deploy-degraded | 0/3 | Tests yellow ratio + priority sort |
| testbed | sts-web (2 replicas) | Running | Tests StatefulSet watcher |
| testbed | ds-logger | Running | Tests DaemonSet watcher |
| testbed | svc-clusterip | ClusterIP | Tests Service watcher |
| testbed | svc-nodeport | NodePort | Tests Service types |
| testbed | job-success | Complete | Tests Job status |
| testbed | job-failing | Error | Tests Job failure |
| testbed | cron-nightly | Scheduled | Tests CronJob watcher |
| testbed | app-config | ConfigMap | Tests ConfigMap watcher |
| testbed | app-secret | Opaque | Tests Secret watcher |
| testbed | data-pvc | Pending | Tests PVC watcher |
| demo | demo-running | Running | Cross-namespace pod |
| demo | demo-failing | Error | Terminated pod |
| demo | demo-pending | ImagePullBackOff | Pending pod |

---

## Results Summary

| Group | Feature Area | Tests | PASS | PARTIAL | FAIL |
|-------|-------------|-------|------|---------|------|
| 1 | Build & Binary | 4 | 4 | 0 | 0 |
| 2 | CLI Parsing | 4 | 4 | 0 | 0 |
| 3 | Context Persistence | 2 | 2 | 0 | 0 |
| 4 | Status Priority Logic | 16 | 16 | 0 | 0 |
| 5 | kubectl Action Layer | 13 | 13 | 0 | 0 |
| 6 | Multi-cluster | 4 | 4 | 0 | 0 |
| 7 | Status Extraction | 7 | 6 | 1 | 0 |
| 8 | Demo Mode | 1 | 1 | 0 | 0 |
| 9 | Resource Filter Aliases | 13 | 13 | 0 | 0 |
| 10 | Safe Action Verification | 2 | 2 | 0 | 0 |
| 11 | TUI — Live cluster (2026-02-25) | 24 | 24 | 0 | 0 |
| **Total** | | **90** | **89** | **1** | **0** |

**Result: 89/90 PASS, 1 PARTIAL PASS, 0 FAIL**

---

## Detailed Results

### GROUP 1 — Build & Binary

| # | Test | Result | Notes |
|---|------|--------|-------|
| 1 | Binary exists and is executable | ✅ PASS | `-rwxr-xr-x` |
| 2 | `kf --version` | ✅ PASS | Output: `kf 0.1.0` |
| 3 | `kf --help` shows all flags | ✅ PASS | RESOURCE arg, `--all-contexts`, `--context`, `-n`, `--read-only`, `--kubeconfig` all present |
| 4 | `--all-contexts` and `--context` in help text | ✅ PASS | Descriptions match implementation |

---

### GROUP 2 — CLI Parsing

| # | Test | Result | Notes |
|---|------|--------|-------|
| 5 | Unknown resource type prints warning | ✅ PASS | Falls back to all resources, no panic |
| 6 | `--context does-not-exist` falls back gracefully | ✅ PASS | `[kuberift] No cluster (...). Showing demo data.` |
| 7 | kubeconfig has `kind-kuberift-dev` context | ✅ PASS | `kubectl config get-contexts` returns it |
| 8 | `--all-contexts` flag accepted by parser | ✅ PASS | No error on parse |

---

### GROUP 3 — Context Persistence

| # | Test | Result | Notes |
|---|------|--------|-------|
| 9 | `~/.config/kuberift/last_context` starts empty | ✅ PASS | File absent on fresh system |
| 10 | Write/read round-trip | ✅ PASS | Wrote `kind-kuberift-dev`, read back `kind-kuberift-dev` exactly |

---

### GROUP 4 — Status Priority Logic (Unit Tests)

All 16 cases covered by `cargo test`:

| Status | Expected Priority | Result |
|--------|------------------|--------|
| CrashLoopBackOff | 0 (critical) | ✅ PASS |
| ImagePullBackOff | 0 (critical) | ✅ PASS |
| Error | 0 (critical) | ✅ PASS |
| Failed | 0 (critical) | ✅ PASS |
| OOMKilled | 0 (critical) | ✅ PASS |
| NotReady | 0 (critical) | ✅ PASS |
| Failed(3) | 0 (critical) | ✅ PASS |
| Evicted | 0 (critical) | ✅ PASS |
| BackOff | 0 (critical) | ✅ PASS |
| [DELETED] | 1 (warning) | ✅ PASS |
| Pending | 1 (warning) | ✅ PASS |
| Terminating | 1 (warning) | ✅ PASS |
| Init:0/1 | 1 (warning) | ✅ PASS |
| Running | 2 (healthy) | ✅ PASS |
| 3/3 | 2 (healthy) | ✅ PASS |
| Complete | 2 (healthy) | ✅ PASS |

---

### GROUP 5 — kubectl Action Layer

All 13 resource types confirmed reachable via the live cluster:

| # | Test | Result | Notes |
|---|------|--------|-------|
| 11 | `kubectl describe pod pod-running -n testbed` | ✅ PASS | Full describe output |
| 12 | `kubectl describe deploy deploy-healthy -n testbed` | ✅ PASS | Full describe output |
| 13 | `kubectl get pod pod-running -n testbed -o yaml` | ✅ PASS | Valid YAML returned |
| 14 | `kubectl logs pod-running -n testbed --tail=5` | ✅ PASS | nginx log lines returned |
| 15 | `kubectl logs demo-job-ztkd6 -n demo --tail=5` | ✅ PASS | Completed job logs accessible |
| 15a | pods | ✅ PASS | All pods visible across namespaces |
| 15b | deployments | ✅ PASS | 4 deployments |
| 15c | services | ✅ PASS | 4 services |
| 15d | nodes | ✅ PASS | 1 node |
| 15e | namespaces | ✅ PASS | 7 namespaces |
| 15f | configmaps | ✅ PASS | All configmaps |
| 15g | secrets | ✅ PASS | 3 secrets |
| 15h | pvc | ✅ PASS | 1 PVC |
| 15i | jobs | ✅ PASS | 4 jobs |
| 15j | cronjobs | ✅ PASS | 1 cronjob |
| 15k | statefulsets | ✅ PASS | 1 statefulset (2 pods) |
| 15l | daemonsets | ✅ PASS | 3 daemonsets |
| 15m | ingresses | ✅ PASS | 1 ingress |

---

### GROUP 6 — Multi-cluster

| # | Test | Result | Notes |
|---|------|--------|-------|
| 16 | `list_contexts()` returns expected contexts | ✅ PASS | `kind-kuberift-dev`, `kind-cilium-test` returned |
| 17 | `kf --context kind-kuberift-dev` connects and streams | ✅ PASS | TUI opens, all resources load |
| 18 | Context file write/read round-trip | ✅ PASS | Persistent across invocations |
| 19 | `kf --all-contexts pod` shows both clusters | ✅ PASS | `kind-kuberift-dev/` and `kind-cilium-test/` prefixes visible, cross-cluster preview works |

---

### GROUP 7 — Status Extraction Correctness

| # | Pod / Resource | Raw value | kuberift shows | Result |
|---|----------------|-----------|----------------|--------|
| 20 | pod-crashloop | `CrashLoopBackOff` (waiting.reason) | `CrashLoopBackOff` | ✅ PASS |
| 21 | pod-imagepull | `ImagePullBackOff` (waiting.reason) | `ImagePullBackOff` | ✅ PASS |
| 22 | pod-init-wait | *(init container running, no waiting.reason)* | `Init:0/1` (from container count) | ⚠️ PARTIAL |
| 23 | pod-running | `Running` (phase) | `Running` | ✅ PASS |
| 24 | demo-failing | `Error` (terminated.reason) | `Error` | ✅ PASS |
| 25 | deploy-healthy | `readyReplicas/spec.replicas` = `2/2` | `2/2` | ✅ PASS |
| 26 | deploy-degraded | `0/3` (readyReplicas absent → 0) | `0/3` | ✅ PASS |

**Note on test 22:** The init container is actively running (not waiting), so `.state.waiting.reason` is absent. `pod_status()` correctly falls back to the init container progress counter, producing `Init:0/1`. This matches `kubectl get pods` output exactly.

---

### GROUP 8 — Demo Mode

| # | Test | Result | Notes |
|---|------|--------|-------|
| 27 | `KUBECONFIG=/nonexistent kf` shows demo data | ✅ PASS | `[kuberift] No cluster (...). Showing demo data.` — 11 sample resources |

---

### GROUP 9 — Resource Filter Aliases

| Alias | Result |
|-------|--------|
| `kf pods` | ✅ PASS |
| `kf svc` | ✅ PASS |
| `kf deploy` | ✅ PASS |
| `kf sts` | ✅ PASS |
| `kf ds` | ✅ PASS |
| `kf cm` | ✅ PASS |
| `kf secret` | ✅ PASS |
| `kf node` | ✅ PASS |
| `kf ns` | ✅ PASS |
| `kf pvc` | ✅ PASS |
| `kf job` | ✅ PASS |
| `kf cj` | ✅ PASS |
| `kf badtype` | ✅ PASS (warning + fallback to all) |

---

### GROUP 10 — Safe Action Verification

| # | Test | Result | Notes |
|---|------|--------|-------|
| 28 | Rollout restart (live against deploy-healthy) | ✅ PASS | `↺ restarting deploy/deploy-healthy` — rollout status tracked to completion |
| 29 | Delete dry-run | ✅ PASS | Confirmation prompt shown before any deletion |

---

### GROUP 11 — TUI Live Testing (2026-02-25, kind-kuberift-dev)

All features tested interactively against the live cluster using tmux.

| # | Feature | Test | Result | Notes |
|---|---------|------|--------|-------|
| 30 | Fuzzy filter | Type `pod-crash` | ✅ PASS | `1/77` match, instant |
| 31 | Unhealthy-first sort | Scroll to top in `kf pod` | ✅ PASS | `ImagePullBackOff`, `CrashLoopBackOff`, `Error` at top; `Running`/`Succeeded` at bottom |
| 32 | Color-coded status | Inspect list visually | ✅ PASS | Red/Yellow/Green per health tier |
| 33 | Preview: describe | Hover over pod-crashloop | ✅ PASS | Full `kubectl describe pod -n testbed` in right pane |
| 34 | Preview: yaml (ctrl-p ×1) | Cycle on pod-crashloop | ✅ PASS | `── YAML: pod/pod-crashloop ──` with full manifest |
| 35 | Preview: logs (ctrl-p ×2) | Cycle on pod-crashloop | ✅ PASS | `── LOGS: pod-crashloop (last 100) ──` |
| 36 | Preview cycle wrap (ctrl-p ×3) | Back to describe | ✅ PASS | Wraps correctly describe→yaml→logs→describe |
| 37 | Resource filter | `kf pod --context kind-kuberift-dev` | ✅ PASS | Header shows `res:pod`, only pods listed |
| 38 | Namespace filter | `kf -n testbed` | ✅ PASS | Header shows `ns:testbed`; cluster-scoped resources still appear |
| 39 | Read-only mode | `kf --read-only` | ✅ PASS | Header shows `[READ-ONLY]` |
| 40 | Read-only blocks exec | `ctrl-e` in read-only | ✅ PASS | `[kuberift] read-only mode: exec is disabled` |
| 41 | Read-only blocks delete | `ctrl-d` in read-only | ✅ PASS | `[kuberift] read-only mode: delete is disabled` |
| 42 | Multi-cluster mode | `kf --all-contexts pod` | ✅ PASS | Both `kind-kuberift-dev/` and `kind-cilium-test/` prefixes; cross-cluster preview works |
| 43 | ctrl-l logs | `pod-running` | ✅ PASS | nginx access logs printed to terminal |
| 44 | ctrl-y yaml | `pod-running` | ✅ PASS | `apiVersion: v1 / kind: Pod` output to terminal |
| 45 | ctrl-r rollout restart | `deploy-healthy` | ✅ PASS | `↺ restarting deploy/deploy-healthy` + rollout status tracked |
| 46 | ctrl-e exec | `pod-running` | ✅ PASS | `Dropping into shell: testbed/pod-running` → `#` prompt inside Debian container; `hostname` returned `pod-running` |
| 47 | ctrl-d delete single | `delete-me` pod (throwaway) | ✅ PASS | `• pod/delete-me [ns/testbed]` prompt → confirmed `y` → `✓ deleted pod/delete-me` |
| 48 | ctrl-d delete multi-select | `delete-me` + `delete-me-2` (tab×2) | ✅ PASS | Both `>>` selected; both deleted; confirmed via `kubectl get pods` |
| 49 | ctrl-d confirmation prompt | Count + resource list | ✅ PASS | Shows `Delete 2 resources? [y/N]` with bullet list |
| 50 | ctrl-f port-forward (service) | `svc-clusterip` → `8888:80` | ✅ PASS | `Forwarding localhost:8888 → svc/svc-clusterip port 80`; `curl localhost:8888` → nginx HTML; `Handling connection for 8888` logged |
| 51 | ctrl-f port-forward (pod) | `pod-running` → `9999:80` | ✅ PASS | `Forwarding localhost:9999 → pod/pod-running port 80`; `curl localhost:9999` → `<title>Welcome to nginx!</title>` |
| 52 | ctrl-f guard (non-pod/svc) | `ctrl-f` on ConfigMap | ✅ PASS | `[kuberift] port-forward only works with pods and services (got cm)` |
| 53 | Privileged port warning | Remote port 80 | ✅ PASS | `[kuberift] warning: port 80 is privileged (may require root/admin)` |

---

## Known Behaviors

**`os error 6` in non-TTY test harness** — Binaries invoked without a real terminal (piped stdin, CI) exit with this error. This is skim's correct behavior. In a real terminal the TUI opens normally.

**`kubectl rollout restart --dry-run` not supported** — kubectl does not accept `--dry-run=client` for rollout restart. The live command was used and rolled back immediately.

**pod-init-wait init container `waiting.reason` empty** — The init container is in `running` state, so `.state.waiting.reason` is absent. `pod_status()` correctly falls back to the container progress counter (`Init:0/1`), matching `kubectl get pods` output exactly.

**TUI relaunch after action** — After every action (describe, logs, yaml, etc.), the skim TUI relaunches immediately. Action output printed to stdout may be briefly overwritten by skim. This is by design — the loop keeps kf alive for sequential actions without restarting.

---

## Conclusion

KubeRift passes **89/90 tests** (89 full pass, 1 partial pass with confirmed correct behavior, 0 fail). All 13 resource types are reachable. All CLI flags parse correctly. Status priority logic is verified across all status categories. All 9 interactive actions (describe, logs, exec, delete, port-forward, rollout-restart, yaml, preview cycling, read-only blocking) confirmed working against a live kind cluster.
