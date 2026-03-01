# Demo Gallery

All recordings were made against a live kind cluster using [vhs](https://github.com/charmbracelet/vhs). Tape files are in [`contrib/tapes/`](https://github.com/syedazeez337/kuberift/tree/master/contrib/tapes).

---

## Main demo

Full showcase: unhealthy-first sort, fuzzy filter, preview cycling, namespace filter.

![demo](https://raw.githubusercontent.com/syedazeez337/kuberift/master/docs/media/demo.gif)

---

## Filtering — resource types and namespaces

`kf pods`, `kf deploy`, `kf svc`, namespace filter with `-n`.

![filter](https://raw.githubusercontent.com/syedazeez337/kuberift/master/docs/media/filter.gif)

---

## Preview modes — describe / yaml / logs

Cycling the right-hand pane with `ctrl-p` on a running pod and a crashing pod.

![preview](https://raw.githubusercontent.com/syedazeez337/kuberift/master/docs/media/preview.gif)

---

## Actions — logs, yaml, rollout restart

`ctrl-l` streams logs, `ctrl-y` dumps the YAML manifest, `ctrl-r` restarts a deployment and tracks rollout status.

![actions](https://raw.githubusercontent.com/syedazeez337/kuberift/master/docs/media/actions.gif)

---

## Safe delete with confirmation

`tab` multi-select, `ctrl-d`, confirmation prompt, cancel with `n`.

![delete](https://raw.githubusercontent.com/syedazeez337/kuberift/master/docs/media/delete.gif)

---

## Multi-cluster — `--all-contexts` and `ctrl-x`

Streaming from multiple clusters simultaneously, then switching context interactively.

![multicluster](https://raw.githubusercontent.com/syedazeez337/kuberift/master/docs/media/multicluster.gif)
