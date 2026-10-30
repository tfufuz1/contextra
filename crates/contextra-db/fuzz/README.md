# Fuzzing infrastructure for `contextra-db`

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. 12 Quelldateien mit Verweisen auf entfernte Crates (SPEC §16, B2). Modul `advanced` verweist auf `adapt`, `cognition` und `graph`. Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

This directory contains `cargo-fuzz` targets for `contextra-db`.

## Targets

- `rrf_fusion`: Fuzzes `weighted_reciprocal_rank_fusion` with arbitrary signal weights (including NaN/Infinity), result sets, and `max_results` values.

## Running locally

```bash
cargo fuzz run rrf_fusion
```

To run with a time limit (e.g. 60 seconds):

```bash
cargo fuzz run rrf_fusion -- -max_total_time=60
```
