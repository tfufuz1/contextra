# Baseline Tooling (`tools/baseline/`)

> **Historisches Dokument (Stand vor Reduktion, 30.10.2026).** Beschreibt den Vorzustand bis Commit `3c1f144fa`. Nicht normativ. Maßgeblich: `docs/SPEC.md` und `docs/PLAN.md` (Fassung 4.0).

This directory contains the reproducible baseline execution tool `run_baseline.py` for the Contextra Rust workspace (35 workspace members).

## Overview

`run_baseline.py` runs systematic build, check, clippy, and test passes across all workspace members as well as workspace-wide checks (`cargo check --workspace`, `cargo clippy --workspace`, `cargo test --workspace`, `cargo fmt --check`, `cargo deny check`, `cargo audit`).

It produces:
1. **Uncut Raw Logs**: Captured in `evidence/baseline/logs/<crate>/<command>.log` (automatically gzip-compressed to `.log.gz` if size >= 2 MB).
2. **Root Cause Analysis (`ROOT_CAUSES.md`)**: Parses compiler and clippy JSON diagnostics (`--message-format=json`) to attribute compiler errors and clippy warnings to their actual source crate (`root_cause_crate`), distinguishing victim crates (`caused_by_dependency = true`).
3. **Crate Evidence Files (`evidence/baseline/<crate>.json`)**: Standardized evidence JSON files detailing test counts, status, compile and test durations, up to 20 slowest tests, last running tests (in case of timeouts or crashes), and full command snippets.
4. **Summary (`evidence/baseline/SUMMARY.json`)**: Workspace-wide summary containing aggregated statistics and status determinations.

## Usage

```bash
# Run self-tests for run_baseline.py
python3 tools/baseline/run_baseline.py --self-test

# Run full baseline pass (requires a clean git worktree)
python3 tools/baseline/run_baseline.py

# Run baseline for a single crate
python3 tools/baseline/run_baseline.py --crate contextra-types

# Custom timeout per crate (default: 1800 seconds / 30 minutes)
python3 tools/baseline/run_baseline.py --timeout 600

# Allow running on a dirty worktree (e.g. during development/testing)
python3 tools/baseline/run_baseline.py --allow-dirty
```

## Guarantees & Invariants

- **Path Normalization**: Absolute paths (e.g. `/app/`, `/home/...`) are normalized to relative repository paths across all logs, JSONs, and markdown reports.
- **Uncut Evidence**: No stdout/stderr log output is truncated. Logs >= 2 MB are compressed with gzip.
- **Root Cause Attribution**: Groups cascading errors (e.g. `contextra-types` clippy warnings breaking downstream crates) by the root cause crate and lint code rather than blaming victim crates.
- **Standard Library Only**: Built purely using Python 3 standard library tools.
