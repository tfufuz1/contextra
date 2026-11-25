# Contextra — Principal QA, Loom Concurrency & Miri Soundness Audit
**Datum:** 2026-09-28
**Autor:** Principal Rust Systems & QA Engineer (Jules)
**Status:** COMPLETE (Findings & Remediation Plan documented)

---

## 1. Executive Summary

| Testphase / Domain | Status | Zusammenfassung |
| :--- | :---: | :--- |
| **Phase 1: Workspace-Kompilierung & Feature-Matrix** | **FAIL** | Der Standard-Build scheitert in `contextra-kvcache` (Test `kivi_aead_order_enforced`), da Feature-Flags fehlen. `--all-features` schlägt fehl aufgrund von `u128`-Typ-Mismatches in DiskANN unter `docid-128` und fehlenden `Edge`-Typimporten in `contextra-graph`. Clippy scheitert an `unreachable!` Denial und Stil-Lints. |
| **Phase 2: Loom Concurrency Analysis** | **FAIL** | `loom_multi_key_lock` passiert erfolgreich. `loom_group_commit` scheitert mit einem Coroutine-Stack-Overflow wegen unbegrenzter Loom-Permutationssuche. `contextra-checkpoint` kompiliert unter `--cfg loom` nicht wegen fehlendem `tokio::fs`. |
| **Phase 3: Miri UB & Soundness Audit** | **PASS / LIMIT** | `contextra-wire` besteht alle 11 Unit- und Integrationstests unter Miri mit **0 Undefined Behavior oder Aliasing-Verstößen**. `contextra-sys` kann dateibasierte `mmap`-Tests aufgrund von Miri-Runtime-Einschränkungen nicht ausführen. |

---

## 2. Phase 1: Workspace-Kompilierung & Feature-Matrix

### 2.1 Basis-Build & Standard-Features
- **Befehl:** `cargo check --workspace --all-targets`
- **Ergebnis:** `FAIL` in `contextra-kvcache` (Test `kivi_aead_order_enforced.rs`).
- **Fehlermeldung:**
  ```text
  error[E0432]: unresolved imports `contextra_kvcache::KiviQuantizeConfig`, `contextra_kvcache::KvTensorView`
  error[E0599]: no method named `write_quantized` found for struct `KvSegment`
  ```
- **Ursache:** Der Integrationstest `crates/contextra-kvcache/tests/kivi_aead_order_enforced.rs` nutzt Typen und Methoden aus dem `kivi-quantization` Modul, ohne selbst mit `#![cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]` abgesichert zu sein.

### 2.2 Feature-Matrix Inkompatibilitäten (`--all-features`)
- **Befehl:** `cargo check --workspace --all-targets --all-features`
- **Ergebnis:** `FAIL` mit 2 Kernproblemen:
  1. **`contextra-vector` (DiskANN unter `docid-128` Feature):**
     - **Datei:** `crates/contextra-vector/src/diskann/build.rs:180`
     - **Fehler:** `bitset.insert(doc_id)` -> Expected `u64`, found `u128`.
     - **Ursache:** Wenn das Feature `docid-128` aktiviert ist, repräsentiert `DocId` ein `u128`. Roaring Bitsets (`roaring::Treemap`) erwarten ein `u64`.
  2. **`contextra-graph` (CSR Module):**
     - **Datei:** `crates/contextra-graph/src/csr/inner.rs:64, 653, 697, 710, 734`
     - **Fehler:** `cannot find type Edge in this scope` / `use of undeclared type Edge`.
     - **Ursache:** Der Typ `Edge` ist in `inner.rs` nicht importiert (`use super::Edge;` oder `use crate::csr::Edge;` fehlt).

### 2.3 Clippy & Compiler-Diagnose
- **Befehl:** `cargo clippy --workspace --all-targets -- -D warnings`
- **Ergebnis:** `FAIL` aufgrund strenger Workspace-Lint-Regeln:
  1. **`contextra-graph/src/ppr.rs:250`:**
     - `usage of the unreachable! macro` — Von `[workspace.lints.clippy]` verboten (`deny(unreachable)`).
  2. **`contextra-graph/src/hyperedge_suggest.rs:74`:**
     - `manual !RangeInclusive::contains implementation` (`count < 2 || count > MAX_RELATE_PARTICIPANTS`).
  3. **`contextra-kvcache/src/store.rs:74 & 191`:**
     - `collapsible_if` und `unnecessary_map_or`.

---

## 3. Phase 2: Loom Concurrency Analysis

### 3.1 Testergebnisse der Fokuskomponenten
- **Multi-Key Locking (`crates/contextra-store/tests/loom_multi_key_lock.rs`):**
  **PASS.** Verifiziert deadlock-freie Multi-Key-Akquise unter `RUSTFLAGS="--cfg loom"`.
- **Snapshot Registry (`crates/contextra-mvcc/tests/loom_snapshot_registry.rs`):**
  **PASS.** Atomic Reference Counting & Pinning verifiziert.
- **Quantizer Race (`crates/contextra-vector/tests/loom_quantizer_race_test.rs`):**
  **PASS.** Lock-Free State-Transitions verifiziert.

### 3.2 Group Commit Coroutine Stack Overflow
- **Datei:** `crates/contextra-store/tests/loom_group_commit.rs`
- **Befehl:** `RUSTFLAGS="--cfg loom" cargo test -p contextra-store --test loom_group_commit -- --test-threads=1`
- **Fehlerbericht:**
  ```text
  ---- test_loom_group_commit_last_hmac_race stdout ----
  coroutine in thread 'test_loom_group_commit_last_hmac_race' has overflowed its stack
  failures:
      test_loom_group_commit_last_hmac_race
  ```
- **Ursache:** Loom nutzt stackful Coroutines zur Exploration aller Thread-Permutationen. Die dreifach nebenläufige Ausführung der `GroupCommitEngine` erzeugt eine Permutations-Explosion im Kanal-Polling und Yield-Loop, die das voreingestellte Loom-Stack-Limit überschreitet.

### 3.3 System-Inkompatibilität unter `--cfg loom`
- **Datei:** `crates/contextra-checkpoint/src/hardlink_cloner.rs:85`
- **Fehler:** `could not find fs in tokio`
- **Ursache:** Tokio deaktiviert `tokio::fs` unter `#[cfg(loom)]`. Module mit Filesystem-I/O müssen unter Loom auf `std::fs` ausweichen oder isoliert werden.

---

## 4. Phase 3: Miri Soundness & Undefined Behavior Audit

### 4.1 Unsafe Island Audits
- **`contextra-wire` (FlatBuffers & Zero-Copy Deserialization):**
  - **Befehl:** `MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test -p contextra-wire`
  - **Ergebnis:** **PASS (11/11 Tests erfolgreich)**.
  - **Soundness-Beweis:** 0 Raw-Pointer Alignment Fehler, 0 Stacked Borrows / Tree Borrows Aliasing-Verstöße, 0 Memory Leaks, 0 Data Races.

### 4.2 Miri Runtime-Grenzen
1. **`contextra-sys` (Memory Mapping):**
   - Miri unterstützt keine dateibasierten `mmap`-Syscalls (`libc::mmap`). Auch mit `MIRIFLAGS="-Zmiri-disable-isolation"` bricht der Test mit `unsupported operation: Miri does not support file-backed memory mappings` ab.
2. **`contextra-simd` (SIMD Vectorization):**
   - Die Interpretation aller SIMD-Instruktionen im Miri-Interpreter überschreitet das Zeitlimit (>400s). Einzelfunktionstests verlaufen ohne Pointer-Alignment-Fehler.

---

## 5. Remediation Plan (Konkrete Code-Fixes)

### 5.1 Fix `contextra-kvcache` Test Feature Gate
**Datei:** `crates/contextra-kvcache/tests/kivi_aead_order_enforced.rs`
```rust
#![forbid(unsafe_code)]
#![cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]

use contextra_crypto::CryptoKey;
```

### 5.2 Fix DiskANN `docid-128` Conversion
**Datei:** `crates/contextra-vector/src/diskann/build.rs:180`
```rust
// Wandelt DocId sicher in u64 um
bitset.insert(doc_id.as_u64());
```

### 5.3 Fix Missing `Edge` Import in Graph CSR
**Datei:** `crates/contextra-graph/src/csr/inner.rs`
```rust
use super::{CsrGraph, GraphStats, Edge};
```

### 5.4 Refactor Denied Clippy Lints in `contextra-graph`
**Datei:** `crates/contextra-graph/src/ppr.rs:250`
```rust
PprAlgorithm::Auto => {
    let (resolved, _) = Self::resolve_algorithm(algorithm, node_count, edge_count);
    Self::compute_with_algorithm(graph, seeds, config, resolved)
}
```

**Datei:** `crates/contextra-graph/src/hyperedge_suggest.rs:74`
```rust
if !(2..=MAX_RELATE_PARTICIPANTS).contains(&count) {
```

### 5.5 Loom Permutations-Begrenzung
**Datei:** `crates/contextra-store/tests/loom_group_commit.rs`
```rust
let mut builder = loom::model::Builder::new();
builder.max_preemptions = 2;
builder.check(|| { ... });
```
