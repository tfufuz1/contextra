# Quick Reference for Jules Agent Sessions

**Master Spec:** [`docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md`](../docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md)

## Common Commands (`cargo xtask`)

```bash
# Dashboard & Context
cargo xtask repo-doctor
cargo xtask crate-context <crate_name>

# Analysis & Gates
cargo xtask doctrine-scan --crate <crate_name>
cargo xtask find-unwired-gaps
cargo xtask check-ring-capabilities-consistency
cargo xtask check-unsafe-islands
cargo xtask determinism-check

# Verification
cargo xtask verify
cargo xtask sync-docs
```
