# Read-Only Mode

Launch kf with `--read-only` to disable every action that modifies the cluster or opens an interactive shell:

```bash
kf --read-only
kf pods -n production --read-only
kf --all-contexts --read-only
```

The header shows `[READ-ONLY]` so the mode is always visible.

---

## What is blocked

| Key | Blocked action | Message shown |
|---|---|---|
| `ctrl-e` | exec into container | `[kubefuzz] read-only mode: exec is disabled` |
| `ctrl-d` | delete resources | `[kubefuzz] read-only mode: delete is disabled` |
| `ctrl-f` | port-forward | `[kubefuzz] read-only mode: port-forward is disabled` |
| `ctrl-r` | rollout restart | `[kubefuzz] read-only mode: rollout-restart is disabled` |

kf prints the message and immediately relaunches the TUI — no state is changed.

---

## What remains available

All read operations work normally in `--read-only`:

- **`enter`** — describe
- **`ctrl-l`** — stream logs
- **`ctrl-y`** — print YAML to stdout
- **`ctrl-p`** — cycle preview modes
- **`ctrl-x`** — switch context

---

## Use cases

**Shared clusters / production access**: give team members a convenient way to inspect cluster state without the ability to accidentally delete or restart resources.

**Pair debugging**: share your screen while someone else drives kf — the read-only flag prevents accidental mutations during a live session.

**CI / monitoring scripts**: pipe kf output to other tools knowing the cluster cannot be modified.
