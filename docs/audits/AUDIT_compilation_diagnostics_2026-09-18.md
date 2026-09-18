# Workspace-weite Kompilierungs-Diagnose (AUDIT Report)

**Auditor:** Google-Jules (Principal Rust Systems Engineer)
**Datum:** 2026-09-18
**HEAD Commit:** `ceee3f40251b85af177d1a5efcf876be2066e6f5`
**VERDICT:** FAILED WITH DIAGNOSTICS (1 Harter Kompilierfehler in Standard-Check, 3 Harte Kompilierfehler in Feature-Matrix, 1 Clippy-Lint-Typ über 4 Crates, 4 Deprecated-API-Warnungen, 4 Unused-Variable-Warnungen, 6 DAG Layer-Inversionen)

---

## 1. Zusammenfassungstabelle pro Crate

| Crate | CHECK | CLIPPY | TEST | FEATURE-MATRIX | ERRORS | DAUER(s) |
|---|---|---|---|---|---|---|
| `memfuse-core-ipc-gen` | PASS | PASS | PASS | N/A | 0 | 6 |
| `memfuse-core` | PASS | PASS | FAIL | `docid-128`: FAIL | 0 (Check) | 42 |
| `memfuse-store` | PASS | PASS | FAIL | `block-cache-v2`: FAIL<br>`fault-injection`: FAIL | 0 (Check) | 67 |
| `memfuse-crypto` | PASS | PASS | FAIL | N/A | 0 (Check) | 103 |
| `memfuse-text` | PASS | PASS | FAIL | N/A | 0 (Check) | 20 |
| `memfuse-index` | PASS | **FAIL** | FAIL | `experimental-diskann`: PASS | 0 (Check) | 23 |
| `memfuse-graph` | PASS | PASS | FAIL | `edge-reinforcement-learning`: PASS | 0 (Check) | 17 |
| `memfuse-checkpoint` | PASS | PASS | PASS | N/A | 0 | 44 |
| `memfuse-calibration` | PASS | PASS | PASS | N/A | 0 | 8 |
| `memfuse-sandbox` | PASS | PASS | PASS | N/A | 0 | 6 |
| `memfuse-db` | **FAIL** | SKIP | SKIP | N/A | 1 | 64 |
| `memfuse-router` | PASS | **FAIL** | FAIL | `egress-sherman-morrison`: PASS<br>`bandit-routing`: PASS | 0 (Check) | 130 |
| `memfuse-candle` | PASS | PASS | FAIL | `kv-bridge`: PASS | 0 (Check) | 41 |
| `memfuse-ollama` | PASS | PASS | PASS | N/A | 0 | 42 |
| `memfuse-embed` | PASS | PASS | PASS | N/A | 0 | 61 |
| `memfuse-agent` | PASS | **FAIL** | PASS | N/A | 0 (Check) | 160 |
| `memfuse-mcp` | PASS | **FAIL** | FAIL | `cloud-egress-guard`: PASS<br>`wasm-sandbox`: PASS | 0 (Check) | 39 |
| `memfuse-bench` | PASS | **FAIL** | FAIL | N/A | 0 (Check) | 66 |

---

## 2. Fehler & Warnungen gruppiert nach Datei

### `benches/relate_bench.rs` (1 Harter Fehler)
- **[E0599]** @ `benches/relate_bench.rs:70:24`: `no method named relate_bidirectional found for struct MemFuse in the current scope`
  - *Hinweis:* Methode existiert nicht an `MemFuse` (oder wurde umbenannt/refactored), was `cargo check -p memfuse-db --all-targets` fehlschlagen lässt.

### `crates/memfuse-index/src/hnsw.rs` (2 Clippy-Warnungen / -D warnings Blocker)
- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:1309:1`: `this function has too many arguments (8/7)`
  - *Hinweis:* `#[warn(clippy::too_many_arguments)]` wird durch `-D warnings` getriggert in `memfuse-index`, `memfuse-router`, `memfuse-agent`, `memfuse-mcp`, `memfuse-bench`.
- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:2085:5`: `this function has too many arguments (8/7)`
  - *Hinweis:* betrifft Funktion `insert_with_layer_cap`.

### `crates/memfuse-db/src/memory_consolidation.rs` (1 Deprecated-API Warnung)
- **[deprecated]** @ `crates/memfuse-db/src/memory_consolidation.rs:388:68`: `use of deprecated method memfuse_core::DocId::as_u64: Nutze inner()`
  - *Hinweis:* `d.as_u64()` sollte durch `d.inner()` ersetzt werden.

### `crates/memfuse-db/tests/consolidation_integration_test.rs` (2 Deprecated-API Warnungen)
- **[deprecated]** @ `crates/memfuse-db/tests/consolidation_integration_test.rs:5:67` & `82:18`: `use of deprecated function memfuse_db::start_consolidation_worker: Konsolidiert in MaintenanceScheduler`
  - *Hinweis:* Veraltete Testfunktion `start_consolidation_worker`.

### `crates/memfuse-db/tests/deletion_proof_integration.rs` (4 Unused-Variable Warnungen)
- **[unused_variables]** @ `crates/memfuse-db/tests/deletion_proof_integration.rs:233:9`: `unused variable: lsm_proof_1`
- **[unused_variables]** @ `crates/memfuse-db/tests/deletion_proof_integration.rs:238:9`: `unused variable: sstable_proof_1`
- **[unused_variables]** @ `crates/memfuse-db/tests/deletion_proof_integration.rs:258:9`: `unused variable: lsm_proof_2`
- **[unused_variables]** @ `crates/memfuse-db/tests/deletion_proof_integration.rs:263:9`: `unused variable: sstable_proof_2`
  - *Hinweis:* Mit Prefix `_` versehen oder in Assertions verwenden.

---

## 3. Kategorisierung aller Befunde

### (a) Harte Kompilierfehler (`cargo check` schlägt fehl)
1. **`benches/relate_bench.rs:70:24` [E0599]:** `relate_bidirectional` existiert nicht auf `MemFuse`. Blockiert `cargo check -p memfuse-db --all-targets`.

### (b) Clippy-Lints (`-D warnings` schlägt fehl)
1. **`crates/memfuse-index/src/hnsw.rs:1309:1 & 2085:5` [clippy::too_many_arguments]:** 8 Parameter überschreiten das Clippy-Limit von 7. Schlägt fehl bei `cargo clippy` für `memfuse-index`, `memfuse-router`, `memfuse-agent`, `memfuse-mcp` und `memfuse-bench`.

### (c) Deprecated-API-Nutzung
1. **`crates/memfuse-db/src/memory_consolidation.rs:388:68` [deprecated]:** `DocId::as_u64` nutzung.
2. **`crates/memfuse-db/tests/consolidation_integration_test.rs:5:67 & 82:18` [deprecated]:** `start_consolidation_worker` nutzung.

### (d) Sonstige Compiler-Warnungen (unused variables / dead code)
1. **`crates/memfuse-db/tests/deletion_proof_integration.rs:233, 238, 258, 263` [unused_variables]:** Unbesetzte Test-Variablen (`lsm_proof_1`, `sstable_proof_1`, `lsm_proof_2`, `sstable_proof_2`).

---

## 4. Feature-Matrix-Abschnitt (Default-off Feature Failures)

Befunde, die **ausschließlich** beim Bauen mit bestimmten Feature-Flags auftreten:

### 1. Feature `memfuse-core/docid-128`
- **Command:** `cargo check -p memfuse-core --features docid-128`
- **`crates/memfuse-core/src/types/domain.rs:1821:34` [E0308]:** Typ-Mismatch zwischen `u64` und `u128` in `DocId`-Konstruktor / Proptest-Gleichheitsprüfungen (`proptest-1.11.0/src/sugar.rs:797:21`).

### 2. Feature `memfuse-store/block-cache-v2`
- **Command:** `cargo check -p memfuse-store --features block-cache-v2`
- **`crates/memfuse-store/src/sstable.rs:182:59` [E0053]:** Method standard signature mismatch in `BlockWeighter`: Trait erwartet Rückgabetyp `u64`, Implementierung liefert `u32`.

### 3. Feature `memfuse-store/fault-injection`
- **Command:** `cargo check -p memfuse-store --features fault-injection`
- **`crates/memfuse-store/tests/group_commit_test.rs:54:13 & 71:13` [E0308]:** Typ-Mismatch in Invalidation callback: Erwartet `Option<bytes::Bytes>`, vorhanden ist `Option<Vec<u8>>`.

---

## 5. DAG-Abschnitt (cargo metadata --dag-check)

Folgende 6 Layer-Inversionen wurden durch `--dag-check` identifiziert:
1. `memfuse-store` (Layer 2) hängt von `memfuse-crypto` (Layer 3) ab.
2. `memfuse-db` (Layer 10) hängt von `memfuse-candle` (Layer 12) ab.
3. `memfuse-db` (Layer 10) hängt von `memfuse-embed` (Layer 14) ab.
4. `memfuse-db` (Layer 10) hängt von `memfuse-ollama` (Layer 13) ab.
5. `memfuse-ollama` (Layer 13) hängt von `memfuse-embed` (Layer 14) ab.
6. `memfuse-router` (Layer 11) hängt von `memfuse-embed` (Layer 14) / `memfuse-ollama` (Layer 13) ab.

---

## 6. Abgleich mit früheren Audit-Läufen (`results/20260917_211619`)

Stichproben-Vergleich der früheren Befunde aus `results/20260917_211619/FEHLERBERICHT.md` mit dem aktuellen Lauf:
1. **`clippy::useless_conversion` in `crates/memfuse-index/src/hnsw.rs:857 & 1992`:**
   *Ergebnis:* **BEHOBEN** (Tritt im aktuellen HEAD `ceee3f40251b` nicht mehr auf).
2. **`clippy::manual_range_contains` in `crates/memfuse-calibration/tests/calibration_deep_tests.rs` & `gasp.rs`:**
   *Ergebnis:* **BEHOBEN** (Keine `manual_range_contains` Lints mehr in `memfuse-calibration` oder `memfuse-candle`).
3. **`clippy::field_reassign_with_default` in `crates/memfuse-store/tests/docid_128_kv_engine.rs`:**
   *Ergebnis:* **BEHOBEN** (In `memfuse-store` vollständig bereinigt).
4. **NEUE Befunde im aktuellen HEAD:**
   *Neu:* `clippy::too_many_arguments` in `crates/memfuse-index/src/hnsw.rs:1309 & 2085` nach der HNSW-Layer-Refactorierung.
   *Neu:* Feature-Matrix Fehler in `block-cache-v2` (`BlockWeighter` Signature), `fault-injection` (`Bytes` vs `Vec<u8>`), und `docid-128` (`proptest` Mismatch).

---

## 7. Priorisierte Kandidatenliste für nachgelagerte FIX-Prompts

Nachfolgend sind alle identifizierten Kompilierungs- und Lint-Probleme priorisiert aufgelistet, getrennt nach Dringlichkeit:

### Priorität 1: Harte Kompilierfehler (Standard & Feature-Matrix)
1. `benches/relate_bench.rs` — Fix `relate_bidirectional` Call oder Anpassung an die aktuelle `MemFuse` API in `memfuse-db`.
2. `crates/memfuse-store/src/sstable.rs` — Feature `block-cache-v2`: Rückgabetyp von `BlockWeighter::weight` von `u32` auf `u64` anpassen.
3. `crates/memfuse-store/tests/group_commit_test.rs` — Feature `fault-injection`: Ersetze `Vec<u8>` durch `bytes::Bytes` im mock response callback.
4. `crates/memfuse-core/src/types/domain.rs` & `src/tx_buffer.rs` — Feature `docid-128`: `DocId.inner()` Vergleich in proptest Macro von `u64` auf `u128` für `docid-128` anpassen.

### Priorität 2: Clippy-Lints (-D warnings Blocker)
1. `crates/memfuse-index/src/hnsw.rs` — `clippy::too_many_arguments` an den Funktionen bei Zeile 1309 (`insert_with_layer`) und 2085 (`insert_with_layer_cap`) beheben (z.B. Parameter in ein Context-Struct bündeln oder `#[allow(clippy::too_many_arguments)]` mit Begründung annotieren).

### Priorität 3: Deprecated API Cleanup
1. `crates/memfuse-db/src/memory_consolidation.rs:388` — `d.as_u64()` zu `d.inner()` migrieren.
2. `crates/memfuse-db/tests/consolidation_integration_test.rs:5, 82` — `start_consolidation_worker` Aufrufe auf `MaintenanceScheduler` umstellen.

### Priorität 4: Sonstige Compiler-Warnungen
1. `crates/memfuse-db/tests/deletion_proof_integration.rs` — Verwahrloste Variablen `lsm_proof_1`, `sstable_proof_1`, `lsm_proof_2`, `sstable_proof_2` mit Unterstrich versehen.
