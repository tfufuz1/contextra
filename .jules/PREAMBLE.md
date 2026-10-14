# Contextra — Jules Preamble & Operating Doctrine

**Version:** 1.0 (SOLL-Zustand 14.10.2026) · **Master Spec:** [`docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md`](../docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md)

---

## Operating Principles
1. **Single Source of Truth:** `docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md` overrides all legacy plans.
2. **`cargo xtask` Exclusive Tooling:** Never run python/sh/bash scripts to analyze the codebase. Use `cargo xtask` commands.
3. **Strict Ring Layering:** Adhere to the Ring DAG (Ring 0 to Ring 4).
4. **Zero-Panic Doctrine:** `#![forbid(unsafe_code)]` outside of `sys`, `simd`, and `wire`. Zero `unwrap()`/`expect()` in Ring 0 and Ring 1 production code.
5. **EP1 Product Hull:** Keep `contextra-mcp` default build hull strictly ≤ 18 crates.
