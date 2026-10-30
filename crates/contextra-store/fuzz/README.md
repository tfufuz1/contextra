# Fuzzing infrastructure for `contextra-store`

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Verweise auf `contextra-core` (Trait-Herkunft) und auf die entfernte Scanner-Logik (SPEC §16, B2). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

This directory contains `cargo-fuzz` targets for `contextra-store`.

## Targets

- `wal_roundtrip`: Fuzzes raw byte streams against `Wal::open_with_config` and `wal.replay()`.

## Running locally

```bash
cargo fuzz run wal_roundtrip
```

To run with a time limit (e.g. 60 seconds):

```bash
cargo fuzz run wal_roundtrip -- -max_total_time=60
```
