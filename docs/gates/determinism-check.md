# Determinism Check Gate

## Overview
The `determinism-check` gate performs static scanning over Ring 0 and Ring 1 crates to detect non-deterministic clock source calls, unseeded random number generation, non-deterministic UUID generation, and HashMap/HashSet iteration within state serialization/hashing logic.

## Usage
```bash
cargo xtask determinism-check [--root <dir>] [--json] [--write-baseline] [--crate <name>]
```

## Governance & Baselines
- `governance/determinism-allow.toml`: Allowlist exceptions backed by ADRs (e.g. ADR-098 HMAC key/salt generation).
- `governance/determinism-baseline.toml`: Baseline of pre-existing findings. Only NEW violations cause PR gate failure.
