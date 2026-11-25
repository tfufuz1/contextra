# WAL Replay Verification Gate

## Overview
The `wal-replay-verify` gate executes crash recovery and chaos matrix test suites for storage engine crates (`contextra-store`, `contextra-checkpoint`, `contextra-mvcc`, `contextra-wire`) to ensure crash consistency and WAL durability.

## Usage
```bash
cargo xtask wal-replay-verify [--root <dir>] [--json] [--budget-secs <secs>] [--base <rev>] [--head <rev>] [--tests <list>]
```

## Applicability
Runs whenever pull request diffs touch files under `crates/contextra-store/`, `crates/contextra-checkpoint/`, `crates/contextra-mvcc/`, or `crates/contextra-wire/`. Yields `not_applicable` otherwise.
