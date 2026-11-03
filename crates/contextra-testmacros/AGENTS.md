# AGENTS – contextra-testmacros

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Proc-Makros für Contextra-Tests und Benchmarks (`#[ctx_test]`, `#[ctx_bench]`).

**Ring und Architektur-Einordnung:**
- Ring: Tooling (`dev`).
- `proc-macro = true`.
- Implementiert L-6 / T-TRACE-1: Validiert Test-IDs (`T-...`) und Benchmark-IDs (`BM-...`) zur Compile-Zeit.

**Verboten:** Direkte Einbindung in Produktions-Crates (nur als `[dev-dependencies]`).
