# Aggregate Gate Integrity (`gate-integrity`)

## Zweck
Orchestriert Test- und Integritäts-Gates (`test-integrity`, `ratchet`, `determinism-check`, `unsafe-audit`, `lock-audit`, `wal-replay-verify`).

## Bestandteile & Risikotrigger
| Bestandteil | Trigger-Bedingung | Skip-Grund |
| :--- | :--- | :--- |
| `test-integrity` | Immer aktiv | - |
| `ratchet` | Immer aktiv (`ratchet check`) | - |
| `determinism-check` | Diff berührt Ring 0/1, `xtask/` oder Zufall/Zeit/IDs | `risk threshold` |
| `unsafe-audit` | Diff berührt Unsafe-Inseln, `unsafe`, `capabilities.toml` | `no path match` |
| `lock-audit` | Diff berührt Async/Locks (`parking_lot`, `Mutex`, Engine) | `no path match` |
| `wal-replay-verify` | Diff berührt Storage/WAL/MVCC/Wire | `not applicable` |

## CLI-Syntax
```bash
cargo xtask gate-integrity [--root <dir>] [--base <rev>] [--head <rev>] [--changed|--all] [--json]
```
- Exit-Codes: `0` wenn alle aktiven Bestandteile OK/SKIP; `1` bei jedem FAIL.
