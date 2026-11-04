# Workspace-Baseline Protokoll (WP-W.0, Gate G-A)

Datum: 2026-11-04
Messung: Reiner Quelltextabgleich und Baseline-Erfassung vor Änderungen in Paket P01.

## 1. Workspace-Check (`cargo check --workspace --all-targets --locked`)

- **Status:** FEHLGESCHLAGEN (Rot)
- **Exit-Code:** 101
- **Dauer:** 9.36s

### Fehlerzusammenfassung Workspace-Check:
```text
error[E0428]: the name `deletion` is defined multiple times
error[E0432]: unresolved import `persistence::MmapIndex`
error[E0425]: cannot find type `KeyManager` in this scope
error[E0425]: cannot find type `KeyManager` in this scope
error[E0592]: duplicate definitions with name `verify_no_ghost_pointers`
error[E0609]: no field `mmap_index` on type `HnswColdCore`
error[E0609]: no field `mmap_index` on type `HnswColdCore`
error[E0282]: type annotations needed
error[E0609]: no field `mmap_index` on type `HnswColdCore`
error[E0560]: struct `SearchContext<'_>` has no field named `mmap`
error[E0560]: struct `SearchContext<'_>` has no field named `mmap_node_count`
error[E0282]: type annotations needed
error[E0282]: type annotations needed
error[E0609]: no field `mmap_index` on type `HnswColdCore`
error[E0609]: no field `mmap_index` on type `HnswColdCore`
error[E0282]: type annotations needed
error[E0282]: type annotations needed
error[E0609]: no field `mmap_index` on type `HnswColdCore`
error[E0609]: no field `mmap_index` on type `HnswColdCore`
error[E0609]: no field `mmap_index` on type `HnswColdCore`
error: could not compile `contextra-vector` (lib) due to 18 previous errors
error: could not compile `contextra-store` (lib) due to 2 previous errors
```

## 2. Einzelmessung je Crate (`cargo check` & `cargo test --locked`)

| Crate | Ring | Check Status | Check Dauer | Test Status | Test Dauer |
|---|---|---|---|---|---|
| `contextra-types` | Ring 0 | PASS | 0.30s | PASS | 18.39s |
| `contextra-ports` | Ring 0 | PASS | 0.19s | PASS | 9.43s |
| `contextra-sys` | Ring 1 | PASS | 0.22s | PASS | 23.10s |
| `contextra-simd` | Ring 1 | PASS | 0.21s | PASS | 20.60s |
| `contextra-durable-fs` | Ring 1 | PASS | 0.28s | PASS | 10.84s |
| `contextra-crypto` | Ring 1 | PASS | 0.25s | PASS | 28.38s |
| `contextra-mvcc` | Ring 2 | PASS | 0.25s | PASS | 23.63s |
| `contextra-store` | Ring 2 | FAIL | 6.23s | FAIL | 14.75s |
| `contextra-vector` | Ring 2 | FAIL | 1.87s | FAIL | 5.69s |
| `contextra-text` | Ring 2 | FAIL | 1.37s | FAIL | 18.63s |
| `contextra-rank` | Ring 2 | PASS | 0.18s | PASS | 1.54s |
| `contextra-erasure` | Ring 3 | FAIL | 0.34s | FAIL | 6.98s |
| `contextra-engine` | Ring 3 | FAIL | 3.48s | FAIL | 11.99s |
| `contextra` | Ring 4 | FAIL | 3.51s | FAIL | 8.23s |
| `contextra-testkit` | dev | PASS | 0.21s | PASS | 12.97s |
| `contextra-testmacros` | dev | PASS | 0.16s | PASS | 27.08s |
| `ctx-verify` | dev | PASS | 0.24s | PASS | 11.99s |
| `contextra-lab` | dev | PASS | 0.22s | PASS | 3.48s |
| `contextra-arch-tests` | dev | FAIL | 0.27s | FAIL | 0.27s |

## 3. Fehlerdetails der fehlschlagenden Crates

### Crate `contextra-store`
- Check Exit-Code: 101
- Test Exit-Code: 101
#### Fehlerliste:
```text
error[E0425]: cannot find type `KeyManager` in this scope
error[E0425]: cannot find type `KeyManager` in this scope
error[E0428]: the name `deletion` is defined multiple times
error[E0432]: unresolved import `persistence::MmapIndex`
error[E0592]: duplicate definitions with name `verify_no_ghost_pointers`
error[E0609]: no field `mmap_index` on type `HnswColdCore`
   = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0609]: no field `mmap_index` on type `HnswColdCore`
   = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0282]: type annotations needed
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0560]: struct `SearchContext<'_>` has no field named `mmap`
error[E0560]: struct `SearchContext<'_>` has no field named `mmap_node_count`
error[E0282]: type annotations needed
error[E0282]: type annotations needed
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
... (14 weitere Fehlerzeilen abgeschnitten)
```

### Crate `contextra-vector`
- Check Exit-Code: 101
- Test Exit-Code: 101
#### Fehlerliste:
```text
error[E0428]: the name `deletion` is defined multiple times
error[E0432]: unresolved import `persistence::MmapIndex`
error[E0592]: duplicate definitions with name `verify_no_ghost_pointers`
error[E0609]: no field `mmap_index` on type `HnswColdCore`
   = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0609]: no field `mmap_index` on type `HnswColdCore`
   = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0282]: type annotations needed
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0560]: struct `SearchContext<'_>` has no field named `mmap`
error[E0560]: struct `SearchContext<'_>` has no field named `mmap_node_count`
error[E0282]: type annotations needed
error[E0282]: type annotations needed
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0282]: type annotations needed
error[E0282]: type annotations needed
... (10 weitere Fehlerzeilen abgeschnitten)
```

### Crate `contextra-text`
- Check Exit-Code: 101
- Test Exit-Code: 101
#### Fehlerliste:
```text
error[E0432]: unresolved imports `contextra_text::buffer::open_binding_envelope`, `contextra_text::buffer::seal_binding_envelope`
error[E0432]: unresolved import `contextra_text::phrase`
error[E0432]: unresolved import `contextra_text::postings`
error[E0432]: unresolved import `contextra_text::segment::SegmentWriter`
error[E0432]: unresolved import `contextra_text::stats`
error[E0425]: cannot find function `get_analyzer` in module `analyzer`
error[E0425]: cannot find function `get_analyzer` in module `analyzer`
error[E0599]: no function or associated item named `open_from_bytes` found for struct `SegmentReader<'a>` in the current scope
note: if you're trying to build a new `SegmentReader<'_>`, consider using `SegmentReader::<'a>::open` which returns `Result<SegmentReader<'_>, contextra_types::error::ContextraError>`
Some errors have detailed explanations: E0425, E0432, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `contextra-text` (test "t_text_spec_tests") due to 8 previous errors
warning: build failed, waiting for other jobs to finish...
```

### Crate `contextra-erasure`
- Check Exit-Code: 101
- Test Exit-Code: 101
#### Fehlerliste:
```text
error[E0761]: file for module `gate` found at both "crates/contextra-erasure/src/gate.rs" and "crates/contextra-erasure/src/gate/mod.rs"
error[E0761]: file for module `journal` found at both "crates/contextra-erasure/src/journal.rs" and "crates/contextra-erasure/src/journal/mod.rs"
error[E0761]: file for module `ledger` found at both "crates/contextra-erasure/src/ledger.rs" and "crates/contextra-erasure/src/ledger/mod.rs"
error[E0761]: file for module `measure` found at both "crates/contextra-erasure/src/measure.rs" and "crates/contextra-erasure/src/measure/mod.rs"
error[E0761]: file for module `purge` found at both "crates/contextra-erasure/src/purge.rs" and "crates/contextra-erasure/src/purge/mod.rs"
error[E0761]: file for module `report` found at both "crates/contextra-erasure/src/report.rs" and "crates/contextra-erasure/src/report/mod.rs"
error[E0761]: file for module `restore` found at both "crates/contextra-erasure/src/restore.rs" and "crates/contextra-erasure/src/restore/mod.rs"
error[E0761]: file for module `saga` found at both "crates/contextra-erasure/src/saga.rs" and "crates/contextra-erasure/src/saga/mod.rs"
error[E0761]: file for module `scope` found at both "crates/contextra-erasure/src/scope.rs" and "crates/contextra-erasure/src/scope/mod.rs"
error[E0761]: file for module `vault` found at both "crates/contextra-erasure/src/vault.rs" and "crates/contextra-erasure/src/vault/mod.rs"
For more information about this error, try `rustc --explain E0761`.
error: could not compile `contextra-erasure` (lib) due to 10 previous errors
warning: build failed, waiting for other jobs to finish...
```

### Crate `contextra-engine`
- Check Exit-Code: 101
- Test Exit-Code: 101
#### Fehlerliste:
```text
error[E0428]: the name `deletion` is defined multiple times
error[E0432]: unresolved import `persistence::MmapIndex`
error[E0592]: duplicate definitions with name `verify_no_ghost_pointers`
error[E0609]: no field `mmap_index` on type `HnswColdCore`
   = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0609]: no field `mmap_index` on type `HnswColdCore`
   = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0282]: type annotations needed
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0560]: struct `SearchContext<'_>` has no field named `mmap`
error[E0560]: struct `SearchContext<'_>` has no field named `mmap_node_count`
error[E0282]: type annotations needed
error[E0282]: type annotations needed
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0282]: type annotations needed
error[E0282]: type annotations needed
... (14 weitere Fehlerzeilen abgeschnitten)
```

### Crate `contextra`
- Check Exit-Code: 101
- Test Exit-Code: 101
#### Fehlerliste:
```text
error[E0425]: cannot find type `KeyManager` in this scope
error[E0425]: cannot find type `KeyManager` in this scope
error[E0428]: the name `deletion` is defined multiple times
error[E0432]: unresolved import `persistence::MmapIndex`
error[E0592]: duplicate definitions with name `verify_no_ghost_pointers`
error[E0609]: no field `mmap_index` on type `HnswColdCore`
   = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0609]: no field `mmap_index` on type `HnswColdCore`
   = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0282]: type annotations needed
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0560]: struct `SearchContext<'_>` has no field named `mmap`
error[E0560]: struct `SearchContext<'_>` has no field named `mmap_node_count`
error[E0282]: type annotations needed
error[E0282]: type annotations needed
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
error[E0609]: no field `mmap_index` on type `HnswColdCore`
    = note: available fields are: `config`, `validation_error`, `tx_buffer`, `quantizer`, `sq8_bias` ... and 10 others
... (14 weitere Fehlerzeilen abgeschnitten)
```

### Crate `contextra-arch-tests`
- Check Exit-Code: 101
- Test Exit-Code: 101
#### Fehlerliste:
```text
error[E0425]: cannot find function `check_double_modules` in this scope
For more information about this error, try `rustc --explain E0425`.
error: could not compile `contextra-arch-tests` (test "arch") due to 1 previous error
```
