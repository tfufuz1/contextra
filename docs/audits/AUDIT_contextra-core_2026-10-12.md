# Audit-Bericht: Strangler-Fassade `contextra-core`

**Datum:** 2026-10-12
**Auditor:** Principal Senior Rust Architect für Contextra
**Ziel:** Strangler-Fassade `crates/contextra-core/src/lib.rs`
**Git HEAD:** `15b27624719ad6a6855dc3f721b2fab17377ea04`
**Claim Ticket:** `facade-audit`

---

## 1. Prüftabelle P1–P8

| Prüfpunkt | Beschreibung | Status | Severity | Beleg (Datei:Zeile) & Befund |
|---|---|---|---|---|
| **P1** | **Fassaden-Reinheit** | PASS | OK | `crates/contextra-core/src/lib.rs:1-127`<br>Enthält ausschließlich `pub use` und `pub mod` Re-Exports sowie Modul-Dokumentation. Keine eigene Logik, Typen oder Inline-Tests. |
| **P2** | **Dependency-Konformität** | PASS | OK | `crates/contextra-core/Cargo.toml:17-21`<br>Abhängigkeiten stimmen exakt mit `capabilities.toml` (`may_depend_on`) überein (`types`, `ports`, `mvcc`, `wire`). Ring-0 Async/I/O Purity eingehalten. |
| **P3** | **Deprecation-Konsistenz** | PASS | OK | `crates/contextra-core/src/lib.rs:13-126`<br>Sämtliche Re-Exports tragen `#[deprecated(since = "0.1.0", note = "...")]`. Glob-Re-Exports und Submodule (`traits`, `ipc`) tragen das Attribut auf Modulebene. |
| **P4** | **Re-Export-Kollisionen** | PASS | OK | `crates/contextra-core/src/lib.rs`<br>`cargo check -p contextra-core --all-features` compiliert ohne Kollisions- oder Mehrdeutigkeitsfehler. `check-duplicate-core-primitives` bestanden. |
| **P5** | **Migrationsfortschritt** | WARN | MINOR | `crates/`<br>353 Dateien in 7 Downstream-Crates importieren noch über `contextra_core` (HEAD 6c8e8e40 Referenz: 268). Ring-0 Crates `contextra-vector` (107 Dateien) und `contextra-simd` (2 Dateien) nutzen Legacy-Importe. |
| **P6** | **Error-Enum-Relokation** | PASS | OK | `crates/contextra-types/src/error.rs:107`<br>Genau eine Definition von `enum ContextraError`, `#[non_exhaustive]` ist gesetzt. 3 `impl From<...>` außerhalb `contextra-types` sind Orphan-Rule-bedingt und DTO-konform. |
| **P7** | **AGENTS-Drift** | PASS | OK | `crates/contextra-core/AGENTS.md`<br>Nennt `#![forbid(unsafe_code)]`, exakte Übereinstimmung mit `src/lib.rs`. Freshness und Integrity Checks bestanden. |
| **P8** | **Dokumentation** | PASS | OK | `crates/contextra-core/src/lib.rs`<br>`#![warn(missing_docs)]` ist aktiv. 0 Rustdoc-Warnungen für `contextra-core`. |

---

## 2. Re-Export-Inventar (`crates/contextra-core/src/lib.rs`)

| Export-Pfad in `contextra-core` | Ziel-Crate / Pfad | `#[deprecated]` Attribute | Alt-Nutzer Dateien (Anz.) |
|---|---|---|---|
| `contextra_core::error` | `contextra_types::error` | `since = "0.1.0"` | 28 |
| `contextra_core::error_dto` | `contextra_types::error_dto` | `since = "0.1.0"` | 2 |
| `contextra_core::model_fingerprint` | `contextra_types::model_fingerprint` | `since = "0.1.0"` | 1 |
| `contextra_core::schema` | `contextra_types::schema` | `since = "0.1.0"` | 12 |
| `contextra_core::tombstone` | `contextra_types::tombstone` | `since = "0.1.0"` | 8 |
| `contextra_core::types` | `contextra_types::types` | `since = "0.1.0"` | 84 |
| `contextra_core::traits` | `contextra_ports` (`pub mod traits`) | `since = "0.1.0"` | 19 |
| `contextra_core::seq_log` | `contextra_mvcc::seq_log` | `since = "0.1.0"` | 4 |
| `contextra_core::snapshot` | `contextra_mvcc::snapshot` | `since = "0.1.0"` | 11 |
| `contextra_core::tx_buffer` | `contextra_mvcc::tx_buffer` | `since = "0.1.0"` | 16 |
| `contextra_core::ipc` | `contextra_wire` (`pub mod ipc`) | `since = "0.1.0"` | 3 |
| `contextra_core::{SeqLogChange, SeqLogEntry, SequenceLog}` | `contextra_mvcc::seq_log` | `since = "0.1.0"` | 9 |
| `contextra_core::{SnapshotGuard, SnapshotRegistry}` | `contextra_mvcc::snapshot` | `since = "0.1.0"` | 15 |
| `contextra_core::{IndexOp, TxBuffer}` | `contextra_mvcc::tx_buffer` | `since = "0.1.0"` | 22 |
| `contextra_core::*` | `contextra_ports::*` | `since = "0.1.0"` | 61 |
| `contextra_core::{ContextraError, Result}` | `contextra_types::error` | `since = "0.1.0"` | 42 |
| `contextra_core::ContextraErrorDto` | `contextra_types::error_dto` | `since = "0.1.0"` | 2 |
| `contextra_core::ModelFingerprint` | `contextra_types::model_fingerprint` | `since = "0.1.0"` | 1 |
| `contextra_core::{DocIdWidth, ManifestSchemaVersion}` | `contextra_types::schema` | `since = "0.1.0"` | 3 |
| `contextra_core::{SeqBitTombstone, TombstoneSemanticsCheck}` | `contextra_types::tombstone` | `since = "0.1.0"` | 10 |

---

## 3. Migrationsfortschritt je Crate

### (a) Übersicht der Downstream-Crates

| Crate | Ring | Cargo.toml Dep | `.rs`-Dateien mit `contextra_core` | Status / Bewertung |
|---|---|---|---|---|
| `contextra-store` | Ring 1 | Ja | 219 | [MINOR] Hauptlast der Alt-Importe; schrittweise Umstellung empfohlen |
| `contextra-vector` | Ring 0 | Ja | 107 | [MINOR] Ring-0-Crate sollte direkt auf `contextra-types`/`ports` zugreifen |
| `contextra` | Ring 4 | Ja | 15 | [MINOR] Umbrella/Convenience Crate |
| `contextra-checkpoint` | Ring 1 | Ja | 5 | [MINOR] Geringer Restaufwand |
| `contextra-py` | Ring 4 | Ja | 4 | [MINOR] Python Bindings |
| `contextra-simd` | Ring 0 | Ja | 2 | [MINOR] Ring-0-Crate in Benchmarks |
| `contextra-engine` | Ring 3 | Ja | 1 | [MINOR] Fast vollständig migriert |
| **Gesamtsumme** | - | **8 Crates** | **353 Dateien** | **Referenz HEAD 6c8e8e40: 268 (+85 Dateien)** |

### (b) Ring-0 Crates mit Legacy-Importen (Direkt-Aufklärungsliste)
1. **`contextra-vector`** (Ring 0): 107 `.rs`-Dateien importieren über `contextra_core`.
2. **`contextra-simd`** (Ring 0): 2 `.rs`-Dateien (u. a. Benchmark-Dateien) importieren über `contextra_core`.

---

## 4. `From`-Implementierungen für `ContextraError`

| Von-Typ | Datei:Zeile | Orphan-Rule-bedingt? | Konform & DTO-sicher? |
|---|---|---|---|
| `GraphMutationError` | `crates/contextra-ports/src/graph.rs:135` | **Ja** (`GraphMutationError` lokal in `contextra-ports`) | **Ja** — Wandelt in `ContextraError::InvalidInput(msg)`. Bricht `error_dto` Roundtrip nicht. |
| `FormatError` | `crates/contextra-store/src/format/error.rs:24` | **Ja** (`FormatError` lokal in `contextra-store`) | **Ja** — Wandelt in `ContextraError::Storage(msg)`. Bricht `error_dto` Roundtrip nicht. |
| `GraphMutationError` | `crates/contextra-graph/src/error.rs:54` | **Ja** (`GraphMutationError` lokal in `contextra-graph`) | **Ja** — Wandelt in `ContextraError::InvalidInput(msg)`. Bricht `error_dto` Roundtrip nicht. |

---

## 5. VERDICT & EVIDENCE

```
================================================================================
VERDICT: PASSED
================================================================================
Die Strangler-Fassade `crates/contextra-core/src/lib.rs` erfüllt sämtliche
Anforderungen an Fassaden-Reinheit (P1), Dependency-Konformität (P2),
Deprecation-Konsistenz (P3), Re-Export-Kollisionsfreiheit (P4), Single-Source
Error Enum Discipline (P6), AGENTS-Integrität (P7) und Dokumentation (P8).

[EVIDENCE: P1_FACADE_PURITY_OK]
[EVIDENCE: P2_DEPENDENCY_CONFORMITY_OK]
[EVIDENCE: P3_DEPRECATION_CONSISTENCY_OK]
[EVIDENCE: P4_REEXPORT_COLLISIONS_NONE]
[EVIDENCE: P5_MIGRATION_PROGRESS_353_FILES]
[EVIDENCE: P6_ERROR_ENUM_RELOCATION_OK]
[EVIDENCE: P7_AGENTS_DRIFT_NONE]
[EVIDENCE: P8_DOCUMENTATION_WARNINGS_NONE]
[EVIDENCE: TEST_PASSED_2_UNIT_5_ROBUSTNESS]
[EVIDENCE: CLIPPY_TEST_WARNINGS_3_CAST_SIGN_LOSS]
================================================================================
```

### Empfehlungen für Nachfolge-Tasks:
1. **Migration Ring 0**: Refactoring von `contextra-vector` (107 Dateien) und `contextra-simd` (2 Dateien) auf direkte Imports (`contextra_types`, `contextra_ports`, `contextra_mvcc`).
2. **Clippy Fix in Test-Target**: Beheben der 3 `clippy::cast_sign_loss`-Warnungen in `crates/contextra-core/tests/robustness.rs` (Zeilen 85, 129, 134).
