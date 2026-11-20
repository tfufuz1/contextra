# contextra-mvcc → contextra-version — Umbenennung, Reduktion und Verteilung

**Ebene:** E1 (Domäne). Zielcrate `contextra-version` (ADR-06, SPEC 4.1 Tabelle Zeile 33, Zeile 225).
**Status:** Normativ (Soll-Zustand), ersetzt die Fassung 2.1.
**Version:** 3.0 (SPEC 4.1)
**Geltungsbereich:** `crates/contextra-mvcc` (Ist). Nach U-VER-0 liegt die Datei unter `crates/contextra-version/PLAN.md`.
**Normative Quellen:** `docs/SPEC.md` ADR-06, ADR-32, REQ-VER-1, REQ-VER-3, REQ-VER-5, Befund F-09 (`compat/`), Entfernungen REM-12, REM-27, Auflösung CMP-1, Phase M1 und M2, Gates G-03, G-07, G-08, G-09, G-11, G-13.

---

## 1. Ausgangslage

**Ist (gemessen):**

| Datei | Zeilen | Inhalt | Entscheidung |
|---|---|---|---|
| `src/snapshot.rs` | 965 | `SnapshotRegistry`, `SnapshotRegistryConfig`, `SnapshotGuard`, Pins | behalten, kürzen (Abschnitt 3) |
| `src/ssi.rs` | 1.237 | `ReadSet`, `SsiValidator`, `SequenceLogSsiValidator`, `PruningBlockerInfo` | entfernen (REM-12, ADR-06) |
| `src/tx_buffer.rs` | 1.474 | `TxBuffer`, `TxBufferConfig`, `IndexOp`, `StagedOpSize`, `Staging` | entfernen (REM-12, ADR-06) |
| `src/tombstone.rs` | 55 | `SeqBitTombstone`, `TombstoneSemanticsCheck` | entfernen (E-IDS-04, Tombstone ist Bitmap in `store`/`erasure`) |
| `src/mvcc_error.rs` | 64 | `MvccError` mit 6 Varianten | in `error.rs` überführen |
| `src/compat/error.rs` | 70 | Kompatibilitätsfehler | in `error.rs` überführen (CMP-1) |
| `src/compat/mod.rs` | 12 | Kompatibilitäts-Modul | entfernen (CMP-1) |
| `src/contract.rs` | 11 | Vertragsreste | entfernen. Verträge liegen in `contextra-ports` |
| `src/lib.rs` | 112 | Modulwurzel, Features, `#![deny(dead_code)]` | neu schreiben (Abschnitt 4) |

**Abhängigkeiten (Ist):** `contextra-ids`, `contextra-env`, optional `contextra-types` und `contextra-ports`, `parking_lot`, `ahash`, `serde`, `bytes`, `thiserror`, `tracing`, `loom` (nur `cfg(loom)`).

**Features (Ist):** `default = []`, `workspace`, `types-compat`, `ports-compat`, `docid-128`. Alle außer `default` entfallen (R3, R7).

**SequenceLog:** liegt im Ist in `contextra-env/src/seq_log.rs` (`SequenceLog`, `SeqLogEntry`). `ssi.rs` hat daneben `SequenceLogSsiValidator`, der mit SSI entfällt.

**Verbraucher außerhalb (13 Dateien):**
- `contextra-store`: 12 Dateien (`memtable/mvcc.rs`, `compaction/tests.rs`, `compaction/engine.rs`, `error.rs`, `lsm/ops/write/batch.rs`, `lsm/ops/write/put_delete.rs`, `lsm/ops/write/commit_pipeline.rs`, `lsm/engine.rs`, `lsm/commit.rs`, `lsm/tests/commit_tests/basic.rs`, `lsm/recovery/startup.rs`, `tests/budget_drift.rs`)
- `contextra-checkpoint`: 1 Datei (`src/hardlink_cloner.rs`)

**Ziel:** `contextra-version` (E1) mit Versionskatalog, Snapshots, Aufbewahrung und Sichtbarkeit. Kein SSI, kein TxBuffer, keine interaktiven Transaktionen (ADR-06). Zielgröße laut SPEC-Tabelle: 2.500 Zeilen. Ausgangsgröße ohne Compat: rund 4.000 Zeilen, davon 2.711 in SSI und TxBuffer.

---

## 2. Umbenennung (ADR-06)

- Verzeichnis `crates/contextra-mvcc` → `crates/contextra-version` (`git mv`, Historie bleibt).
- Paketname `contextra-version`, Bibliotheksname `contextra_version`.
- Workspace-Mitglied ersetzen. Verbraucher erhalten den neuen Namen erst, wenn sie selbst umgebaut sind (Abschnitt 5, U-VER-8).
- Die Umbenennung ist **nicht** vor dem Lösen von `store` von `mvcc` durchführbar (REM-27), weil `store` dann auf einen E1-Crate zeigt, den R2 verbietet.

---

## 3. Verteilung je Item

### 3.1 Behalten und kürzen

| Item | Datei | Aktion |
|---|---|---|
| `SnapshotRegistry`, `SnapshotRegistryConfig`, `SnapshotGuard` (`seq_no`, `tenant`) | `snapshot.rs` | behalten |
| `register`, `try_register`, `register_for_tenant`, `*_at`-Varianten | `snapshot.rs` | behalten. Zeitquelle ist ausschließlich `Clock` aus `contextra-env` (R10) |
| `pin`, `pin_for_tenant`, `pin_at`, `unpin`, `longest_active_pin`, `reap_expired_pins`, `reap_expired_pins_at` | `snapshot.rs` | behalten. Pins begrenzen die Aufbewahrung (REQ-VER-3) |
| `oldest_active_snapshot`, `min_active_seqno`, `min_read_snapshot`, `min_for_compaction` | `snapshot.rs` | behalten. Wert für `store`-Kompaktion über `ports` |
| `with_time_fn` | `snapshot.rs` | entfernen. Testzeit kommt über `ManualClock` aus `contextra-env` (Feature `test-utils`) |
| `revoke_covering` | `snapshot.rs` | nur nach Entscheidung O-VER-1. Vorschlag: entfernen |

### 3.2 Neue Module (additiv, SPEC M2)

| Modul | Inhalt | Anforderung | Aufbau |
|---|---|---|---|
| `catalog.rs` | Kette je Dokument `(rev, seq, valid_from, valid_to, content_ref, state)`. `state` ist `Current`, `Superseded` oder `Retired` | REQ-VER-1 | Kette als Wert, Persistenz über den `KvStore`-Port |
| `retention.rs` | Aufbewahrungsregeln: Mindestdauer, Pins, Ausnahmen für Rechtshaltung | REQ-VER-3 | Reine Entscheidungsfunktionen ohne I/O |
| `visibility.rs` | Sichtbarkeit einer Fassung zum Zeitpunkt `T` (Zeitreise mit `index_history`) | REQ-VER-2, REQ-VER-5 | Logik. Typen `Visibility` und `ContentLifecycle` liegen in `contextra-ports::version` |
| `sequence.rs` | `SequenceLog`, `SeqLogEntry` aus `contextra-env/src/seq_log.rs` | ADR-06, U-ENV-4 | verschieben (Entscheidung O-VER-3) |
| `error.rs` | `VersionError` (Varianten `LimitExceeded`, `InvalidInput`, `Internal`) plus `From<VersionError> for ContextraError` | CMP-1 | Zusammenführung von `mvcc_error.rs` und `compat/error.rs` |

### 3.3 Entfernen

| Item | Grund | Ersatz |
|---|---|---|
| `ReadSet`, `SsiValidator`, `SequenceLogSsiValidator`, `PruningBlockerInfo` | REM-12, ADR-06 | keine. Konsistenz über Katalog und Sichtbarkeit |
| `TxBuffer`, `TxBufferConfig`, `IndexOp`, `StagedOpSize`, `Staging` | REM-12, ADR-06 | atomare Schreibstapel liegen in `store` (`WriteOp`, `Family` aus `ports::kv`) |
| `SeqBitTombstone`, `TombstoneSemanticsCheck` | E-IDS-04 | Tombstone-Bitmap in `store`/`erasure` |
| `MvccError` (6 Varianten: `Conflict`, `LimitExceeded`, `Transaction`, `Storage`, `InvalidInput`, `Internal`) | in `VersionError` überführt. `Conflict` und `Transaction` entfallen mit SSI | `ContextraError`-Variante aus `ports::error` für `Storage` |
| `compat/mod.rs`, `contract.rs` | CMP-1, Verträge gehen an Ports | — |

### 3.4 Abhängigkeiten nach Reduktion

**Ziel-Cargo:** `contextra-ids`, `contextra-env`, `contextra-ports`, `arc-swap` (Snapshot-Register), `thiserror`. Für `loom` nur als `dev-dependency` mit `cfg(loom)`.

**Entfällt:** `contextra-types`, `parking_lot` (sofern `std::sync` genügt, prüfen), `ahash`, `serde`, `bytes`, `tracing`, `contextra-durable-fs` (Entscheidung O-VER-2), `blake3`, `roaring`.

**Features:** `default = []`. Entfernen: `workspace`, `types-compat`, `ports-compat`, `docid-128` (R3, R7, E-IDS-01).

---

## 4. Modulwurzel `lib.rs`

Neu schreiben. Inhalt:

```rust
#![forbid(unsafe_code)]        // R5
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))] // R6

pub mod catalog;
pub mod error;
pub mod retention;
pub mod sequence;
pub mod snapshot;
pub mod visibility;

pub use error::VersionError;
pub use snapshot::{SnapshotGuard, SnapshotRegistry, SnapshotRegistryConfig};
```

`#![deny(dead_code)]` entfällt. Die Architekturdokumentation in `lib.rs` nennt `contextra-engine` als Verbraucher. Das ist REM-19 und wird gestrichen.

---

## 5. Umbauschritte

| ID | Schritt | Abhängig von | Abnahme |
|---|---|---|---|
| **U-VER-0** | Store von `mvcc` lösen (REM-27, eigener Schritt im `store`-PLAN). Danach `git mv crates/contextra-mvcc crates/contextra-version`, PLAN verschieben, Paketname setzen | REM-27 | `cargo check -p contextra-version` |
| **U-VER-1** | Additiv `error.rs` mit `VersionError` und Konvertierung anlegen. `mvcc_error.rs` und `compat/error.rs` bleiben, bis Verbraucher umgestellt sind | U-PRT-3 (`ports::error`) | `cargo check` |
| **U-VER-2** | Additiv `catalog.rs`, `retention.rs`, `visibility.rs` mit Tests T-VER-07 und T-VER-08 | U-PRT-3 (`ports::version`), U-IDS | Tests grün |
| **U-VER-3** | `sequence.rs` übernimmt `SequenceLog` aus `contextra-env`. Der Loom-Test `loom_sequence_log.rs` wandert mit | U-ENV-4 (Ziel `contextra-version`) | `RUSTFLAGS="--cfg loom" cargo test -p contextra-version --release loom_sequence` grün |
| **U-VER-4** | `snapshot.rs` kürzen: `with_time_fn` entfernen, Zeitquelle auf `Clock`. `revoke_covering` gemäß O-VER-1 | U-ENV-3 | `cargo test -p contextra-version --test loom_snapshot_registry` grün |
| **U-VER-5** | Entfernen: `ssi.rs`, `tx_buffer.rs`, `tombstone.rs`, `mvcc_error.rs`, `compat/`, `contract.rs`. Vorher `grep -rn "SsiValidator\|TxBuffer\|SeqBitTombstone\|MvccError" crates` leer außerhalb des Crates | U-VER-1, U-VER-2 | Build grün |
| **U-VER-6** | Tests bereinigen (Abschnitt 6). Golden-Korpus neu erzeugen und prüfen | U-VER-5 | Golden-Diff im PR begründet |
| **U-VER-7** | Cargo: Abhängigkeiten und Features nach 3.4 | U-VER-5 | `cargo tree -p contextra-version -e normal` nur erlaubte Crates |
| **U-VER-8** | Verbraucher umstellen: `contextra-checkpoint/src/hardlink_cloner.rs` (1 Datei) über Port statt Version-Import. `contextra-store` ist bereits gelöst (U-VER-0). Dann `contextra_mvcc` → `contextra_version` in allen Verbrauchern | U-VER-7 | `grep -rln "contextra_mvcc\|contextra-mvcc" crates` leer |

---

## 6. Tests

Die Tests stammen aus dem Ist-Bestand (`tests/`, 23 Dateien). Einstufung nach den Begriffen SSI, TxBuffer und Tombstone im Quelltext:

| Datei | Treffer SSI / TxBuffer / Tombstone | Aktion |
|---|---|---|
| `audit_regression_tests.rs` | 6 / 6 / 18 | gemischt. Snapshot- und Sichtbarkeitsfälle behalten, SSI- und TxBuffer-Fälle entfernen |
| `b16_reproduction.rs` | 7 / 0 / 0 | entfernen (SSI) |
| `campaign_cov_ssi_coarsening.rs` | 9 / 0 / 0 | entfernen (SSI) |
| `campaign_cov_tx_buffer_reap.rs` | 0 / 6 / 0 | entfernen (TxBuffer) |
| `campaign_j42_ssi.rs` | 11 / 0 / 0 | entfernen (SSI) |
| `golden_corpus.rs` | 8 / 0 / 8 | entfernen. Neuer Korpus in U-VER-6 |
| `history_anomaly_checker.rs` | 14 / 0 / 6 | entfernen (Anomalien sind SSI-Thema) |
| `isolation_tests.rs` | 23 / 29 / 7 | entfernen. Isolation ohne SSI wird als Katalog-Sichtbarkeit neu getestet (T-VER-07) |
| `loom_tx_buffer.rs` | 0 / 10 / 0 | entfernen |
| `loom_sequence_log.rs` | 0 / 0 / 0 (SequenceLog 10) | behalten, mit `sequence.rs` verschieben |
| `loom_snapshot_registry.rs` | 0 / 0 / 0 | behalten |
| `ssi_coarsening_proptest.rs`, `ssi_determinism_proptest.rs`, `ssi_forget_and_bounds.rs`, `ssi_hardening.rs`, `ssi_metrics_observability.rs`, `ssi_sentinel_guard_tests.rs`, `ssi_serializability_model.rs`, `ssi_write_skew.rs` | 6 bis 36 / 0 bis 3 / 0 bis 7 | entfernen (SSI) |
| `tombstone_diff.rs` | 0 / 0 / 10 (`contextra_types`) | entfernen |
| `tx_buffer_byte_budget.rs` | 0 / 8 / 0 | entfernen |
| `visibility_proptest.rs` | 0 / 0 / 0 (SequenceLog 4) | behalten, auf `visibility.rs` umstellen |
| `tests/golden/` (`mvcc_golden.bin`, `mvcc_golden.json`, `MANIFEST.sha256`, `regen.sh`) | — | neu erzeugen als `version_golden`, nach U-VER-6 |
| `tests/legacy_reference/tombstone.rs` | — | entfernen |

**Neue Tests:** T-VER-07 (Sichtbarkeit zum Zeitpunkt `T`, REQ-VER-5 mit `index_history`), T-VER-08 (Kette mit `Current`, `Superseded`, `Retired`, REQ-VER-1), T-ER-10 und T-ER-11 (aus SPEC Phase M2).

---

## 7. Entfernungen

| ID | Gegenstand | Grund |
|---|---|---|
| E-VER-01 | `ssi.rs` (1.237 Zeilen) | REM-12, ADR-06 |
| E-VER-02 | `tx_buffer.rs` (1.474 Zeilen) | REM-12, ADR-06 |
| E-VER-03 | `tombstone.rs` (55 Zeilen) und `tests/legacy_reference/` | E-IDS-04 |
| E-VER-04 | `mvcc_error.rs` (nach Überführung), `compat/` (CMP-1), `contract.rs` | CMP-1 |
| E-VER-05 | `with_time_fn` in `snapshot.rs` | R10 |
| E-VER-06 | Features `workspace`, `types-compat`, `ports-compat`, `docid-128` | R3, R7 |
| E-VER-07 | Abhängigkeiten `contextra-types`, `parking_lot` (nach Prüfung), `ahash`, `serde`, `bytes`, `tracing`, `contextra-durable-fs` (nach O-VER-2) | Abschnitt 3.4 |
| E-VER-08 | Verweis auf `contextra-engine` in `lib.rs` | REM-19 |
| E-VER-09 | Tests nach Abschnitt 6 | — |

---

## 8. Abnahme

| ID | Prüfung | Soll |
|---|---|---|
| T-VER-01 | `grep -rln "contextra_mvcc\|contextra-mvcc" crates` | leer |
| T-VER-02 | `grep -rn "SsiValidator\|TxBuffer\|SeqBitTombstone\|MvccError\|compat" crates/contextra-version/src` | leer |
| T-VER-03 | `cargo test -p contextra-version` | grün |
| T-VER-04 | `RUSTFLAGS="--cfg loom" cargo test -p contextra-version --release loom_` | grün |
| T-VER-05 | `find crates/contextra-version/src -name '*.rs' -exec cat {} + \| wc -l` | ≤ 2.500 (SPEC-Ziel) |
| T-VER-06 | `grep -rn "unsafe\|\.unwrap()\|\.expect(\|panic!" crates/contextra-version/src` außerhalb von Tests | leer (R5, R6) |
| T-VER-07 | Zeitreise: Sichtbarkeit zum Zeitpunkt `T` mit `index_history` (REQ-VER-5) | grün |
| T-VER-08 | Kette mit Zuständen `Current`, `Superseded`, `Retired` (REQ-VER-1) | grün |

**Gates:** Phase M1 (G-03, G-07, G-08, G-09, G-11, G-13) für Umbenennung und Entfernung. Phase M2 (T-ER-10, T-ER-11, REQ-VER-5) für Katalog und Sichtbarkeit.

---

## 9. Offene Entscheidungen

| ID | Frage | Vorschlag und Begründung |
|---|---|---|
| **O-VER-1** | Soll `SnapshotRevoker` bestehen bleiben? | **Entfernen.** Der Begriff kommt in SPEC 4.1 nicht vor (Suche ohne Treffer). Die Vorfassung band ihn an `ErasureLayer`, den E-PRT-11 entfernt. Das Lesen nach Löschung wird im Lesepfad über das mandantenweite `ReadGate` aus SPEC 7.8 geprüft, nicht über einen Widerruf im Snapshot. Bei Zustimmung entfällt `revoke_covering` in U-VER-4 |
| **O-VER-2** | Bleibt `contextra-durable-fs` als Abhängigkeit? | **Entfällt.** Der Katalog wird über den `KvStore`-Port in `store` persistiert. `version` schreibt keine Dateien selbst |
| **O-VER-3** | Gehört `SequenceLog` zu `version` oder zu `env`? | **`version`.** Die Commit-Reihenfolge ist Katalogwissen. `env` behält nur Uhr, Metriken und Testhilfen. Dies gilt auch für U-ENV-4 in `contextra-env/PLAN.md` |
