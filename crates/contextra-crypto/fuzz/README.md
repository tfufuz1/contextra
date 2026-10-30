# Fuzzing infrastructure for `contextra-crypto`

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Feature-Verweise ohne Deklaration (`encryption-at-rest`, `wal-integrity`; SPEC §16, B4) und Restverweise auf entfernte Löschbeweis-Module (B2). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

This directory contains `cargo-fuzz` targets for `contextra-crypto`.

## Targets

- `deletion_proof_tamper`: Generates a valid `DeletionProof`, applies arbitrary byte mutations, and verifies that bincode deserialization and `proof.verify()` never panic and safely return clean error/false results upon tampering.

## Running locally

```bash
cargo fuzz run deletion_proof_tamper
```

To run with a time limit (e.g. 60 seconds):

```bash
cargo fuzz run deletion_proof_tamper -- -max_total_time=60
```
