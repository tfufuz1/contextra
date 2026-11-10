# Handoff MVCC-2 — contextra-mvcc: Loom, Tombstone-Kopie, MvccError, compat/

- **Crate:** `contextra-mvcc`
- **Commit-Kennung:** MV1
- **Branch:** `jules/mvcc-2-harden`
- **Stand:** 2026-11-10

## 1. Ergebnis je Arbeitspaket (WP)

| Arbeitspaket | Status | Prüfbefehl | Bemerkung |
|---|---|---|---|
| **MV1.0** Baseline, Kopplungs- & Feature-Befund | 🟢 grün | `cargo check -p contextra-mvcc --all-targets` | Baseline abgelegt unter `docs/decoupling/baseline/mvcc/` |
| **MV1.1** Loom-Modell & Nebenläufigkeits-Tests | 🟢 grün | `RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --test loom_snapshot_registry --test loom_sequence_log --test loom_tx_buffer --release` | 3 Loom-Modelle (`snapshot_registry`, `sequence_log`, `tx_buffer`) |
| **MV1.2** Golden- & Differenz-Korpus | 🟢 grün | `cargo test -p contextra-mvcc --test golden_corpus` | Byteweise Golden-Validierung + Truncation/Bitflip-Tests |
| **MV1.3** Tombstone-Kopie (Commit A) | 🟢 grün | `cargo test -p contextra-mvcc --test tombstone_diff` | Wörtliche Kopie aus `types`, Differenztest bitgleich |
| **MV1.4** Fehlerabbildung (`MvccError`) | 🟢 grün | `cargo check -p contextra-mvcc --all-targets --locked` | `MvccError` eingeführt, `compat/error.rs` From-Impl |
| **MV1.5** Umbau MvccError-Umstellung & compat/ | 🟢 grün | `cargo test -p contextra-mvcc --locked` | Quellkompatibilität für `store`/`vector`/`engine`/`checkpoint` gewahrt; ids/env zurückgestellt |
| **MV1.6** Härtung & Zero-Panic | 🟢 grün | `python3 tools/zero_panic_analyzer.py --crate contextra-mvcc --fail-on-prod --output-dir /tmp/MV1-diag` | 0 Prod-Panics, 0 Wording-Verstöße |
| **MV1.7** Dokumentation & Handoff | 🟢 grün | `cargo clippy -p contextra-mvcc --all-targets -- -D warnings` | AGENTS.md Modulkarte aktualisiert |

## 2. Abweichungen von SPEC-v2

Gemäß UNABHÄNGIGKEITSKLAUSEL des Auftrags:
- Der Umbau auf `contextra-ids` und `contextra-env` sowie Manifest-Umbau (Entfernen von `types`/`ports` aus normalen Abhängigkeiten) entfällt in diesem Paket und ist als „zurückgestellt (Folgeschritt Integration)“ eingestuft.
- `tombstone.rs` verbleibt in `contextra-types` unverändert (Tabu; Koordinationsanforderung für PORTS-5).

## 3. Koordinationsanforderungen

1. **`contextra-types/src/lib.rs`**: Sobald alle Workspace-Verbraucher von `SeqBitTombstone` und `TombstoneSemanticsCheck` auf `contextra_mvcc::tombstone::*` umgestellt sind, können die Re-Exports in `contextra-types` ersatzlos entfernt werden (zugeordnet Paket PORTS-5).
2. **Folgeschritt Integration (ids/env)**: Sobald der Workspace-weite Integrationsschritt erfolgt, werden `DocId`, `TxId`, `TOMBSTONE_BIT` aus `contextra-ids` und `Clock`, `MetricsSink` aus `contextra-env` bezogen.

## 4. API-Freeze-Tabelle

| Symbol / Pfad | Externe Nutzer | Status Quellkompatibilität |
|---|---|---|
| `contextra_mvcc::SnapshotRegistry` | `contextra-checkpoint`, `contextra-engine`, `contextra-store` | 🟢 Identisch, unverändert |
| `contextra_mvcc::SnapshotGuard` | `contextra-checkpoint`, `contextra-engine` | 🟢 Identisch |
| `contextra_mvcc::SequenceLog` | `contextra-vector` | 🟢 Identisch |
| `contextra_mvcc::SeqLogChange` | `contextra-vector` | 🟢 Identisch |
| `contextra_mvcc::TxBuffer` | `contextra-vector`, `contextra-engine`, `contextra-store` | 🟢 Identisch (`stage`, `stage_kv`, etc. quellkompatibel) |
| `contextra_mvcc::IndexOp` | `contextra-engine`, `contextra-store`, `contextra-vector` | 🟢 Identisch |
| `contextra_mvcc::SsiValidator` | `contextra-store` | 🟢 Identisch (`validate` gibt weiterhin `contextra_types::Result<()>` zurück) |
| `contextra_mvcc::SequenceLogSsiValidator` | `contextra-store` | 🟢 Identisch |
| `contextra_mvcc::ReadSet` | `contextra-store` | 🟢 Identisch |
| `contextra_mvcc::tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES` | `contextra-store` | 🟢 Identisch |

## 5. Kern-API-Lücken

- Keine Kern-API-Lücken. Alle extern konsumierten Methoden behalten ihre bisherige Quellkompatibilität bzw. konvertieren transparente Fehlertypen über `From<MvccError> for ContextraError`.

## 6. Messwerte vorher / nachher

| Metrik | Vorher (MV1.0) | Nachher (MV1.7) |
|---|---|---|
| Tests bestanden (`cargo test -p contextra-mvcc`) | 51 unit + 17 integration tests (68) | 52 unit + 21 integration tests (73) |
| Loom Tests | 1 Test (`loom_snapshot_registry`) | 3 Tests (`snapshot_registry`, `sequence_log`, `tx_buffer`) |
| Golden Files | 0 | 2 (`mvcc_golden.json`, `mvcc_golden.bin`) + `MANIFEST.sha256` |
| Prod Zero-Panic Verstöße | 0 | 0 |
| Forbidden Wording Befunde | 2 | 0 |

## 7. Offene Risiken

- Keine offenen Risiken für den laufenden Betrieb. Alle bestehenden Regressionstests, Isolationstests und Property-Tests laufen fehlerfrei durch.
