# Keybindings

## Navigation

| Key | Action |
|---|---|
| Type anything | Fuzzy filter the list in real time |
| `↑` / `↓` | Move cursor up / down |
| `tab` | Toggle multi-select on current item |
| `ctrl-u` | Clear the filter query |
| `esc` | Quit kf |

---

## Actions on selected resource(s)

| Key | Action | Works on | Multi-select |
|---|---|---|:---:|
| `enter` | `kubectl describe` | all types | ✓ |
| `ctrl-l` | Stream pod logs (`--tail=200`) | pods | ✓ |
| `ctrl-e` | `kubectl exec -it` into shell | pods only | — |
| `ctrl-d` | Delete with `[y/N]` confirmation (>10 resources requires typing `yes`) | all types | ✓ |
| `ctrl-f` | Port-forward — prompts for local and remote port | pods, services | — |
| `ctrl-r` | `kubectl rollout restart` + tracks status | deploy, sts, ds | ✓ |
| `ctrl-y` | Print full YAML manifest to stdout | all types | ✓ |

---

## Preview and context

| Key | Action |
|---|---|
| `ctrl-p` | Cycle preview pane: **describe → yaml → logs** |
| `ctrl-x` | Open context picker — switch cluster without restarting |

---

## Read-only mode restrictions

When launched with `--read-only`, the following keys are disabled with an explanatory message:

| Key | Blocked action |
|---|---|
| `ctrl-e` | exec |
| `ctrl-d` | delete |
| `ctrl-f` | port-forward |
| `ctrl-r` | rollout restart |

`enter`, `ctrl-l`, `ctrl-y`, and `ctrl-p` remain fully available in read-only mode.

---

## Notes

- **Multi-select with `tab`**: after tabbing one or more items, every subsequent action applies to all selected items. The list shows `>>` next to selected items and the item count in the header updates.
- **`ctrl-e` exec shell order**: kf tries `/bin/sh` first, then `/bin/bash`, inside the container.
- **`ctrl-f` privileged ports**: ports below 1024 trigger a warning (`may require root/admin`) but are not blocked.
