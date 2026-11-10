# BASELINE: contextra-sys

Erfasst am: 2026-11-09
Werkzeuge: `tools/unsafe_boundary_auditor.py`, `tools/unsafe_boundary_tracer.py`, `cargo 1.89.0`

## 1. Build & Test Baseline
- `cargo check -p contextra-sys --no-default-features --offline`: OK (0.07s)
- `cargo check -p contextra-sys --offline`: OK (0.07s)
- `cargo test -p contextra-sys --all-targets --offline`: OK (33 Unit Tests, 8 Integrations-Tests, 9 Benchmark-Tests, 0 failed, 0 ignored)
- `cargo clippy -p contextra-sys --all-targets --offline -- -D warnings`: OK (0 warnings)

## 2. Unsafe Audit Baseline
- Anzahl `unsafe`-Blöcke: 22
- Anzahl `unsafe fn`: 0
- Anzahl `unsafe impl`: 0
- Fehlende `// SAFETY:`-Kommentare: 0
- Unsafe-Insel Status: Ja (Ring 0 Unsafe-Insel)

## 3. Abhängigkeiten Baseline
- Interne Workspace-Abhängigkeiten: 0
- Externe Abhängigkeiten: `libc` (POSIX), `windows-sys` (Win32), `memmap2`, `thiserror`, `tracing`
- Dev-Abhängigkeiten: `criterion`, `tempfile`
