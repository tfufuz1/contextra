# Zero-Panic & Unsafe Indexing Audit Report

> Generiert automatisch durch `tools/zero_panic_analyzer.py`.
> Workspace Root: `/app`
> Gesamte gefundene Produktivcode-Verletzungen: **1**

## Crate Übersicht (Produktivpfade)

| Crate | Prod unwrap | Prod expect | Prod panic | Prod todo | Prod assert | Test Findings |
|---|---|---|---|---|---|---|
| `contextra-crypto` | 0 | 0 | 0 | 0 | 1 | 1027 |

## Detaillierte Produktiv-Fundstellen

### `contextra-crypto`

- `/app/crates/contextra-crypto/fuzz/fuzz_targets/wal_hmac_chain_verify_fuzz.rs:144` [assert]: `assert!(`
