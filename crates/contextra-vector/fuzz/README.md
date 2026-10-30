# Fuzzing infrastructure for `contextra-vector`

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Enthält Code für DiskANN (entfernt) und RaBitQ (Nicht-Ziel, offen: D-R4). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

This directory contains `cargo-fuzz` targets for `contextra-vector`.

## Targets

- `hnsw_insert_search`: Fuzzes sequences of `HnswIndex` operations (insert, search, delete) with arbitrary dimensions, vector values (including NaN/Inf), and `k` values.

## Running locally

```bash
cargo fuzz run hnsw_insert_search
```

To run with a time limit (e.g. 60 seconds):

```bash
cargo fuzz run hnsw_insert_search -- -max_total_time=60
```
