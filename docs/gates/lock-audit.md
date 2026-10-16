# Lock Audit Gate

## Overview
The `lock-audit` gate checks touched crates for potential deadlocks and lock ordering violations:
- Scans for nested async `.lock().await` invocations in close proximity using `ast-grep`/`sg` or syn fallback.
- Runs `clippy::await_holding_lock` on touched crates.
- Checks `AGENTS.md` for documented lock hierarchy rules.

## Usage
```bash
cargo xtask lock-audit [--root <dir>] [--json] [--base <rev>] [--head <rev>]
```
