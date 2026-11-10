# Handoff: CKPT-3 — contextra-checkpoint Decoupling

## 1. Status & Zusammenfassung
- **Mission:** CKPT-3 — Härtung und Entkopplung von `contextra-checkpoint` (SnapshotSource, CheckpointError, compat/).
- **Commit-Kennung:** CK3
- **Branch:** `jules/ckpt-3-decouple`

## 2. Ergebnis je Arbeitspaket

| WP | Status | Prüfbefehl | Beschreibung |
|---|---|---|---|
| CK3.0 | GRÜN | `cargo check -p contextra-checkpoint --all-targets --locked` | Baseline erhoben, Inventar für StorageEngine & FS-Zugriffe dokumentiert. |
| CK3.1 | GRÜN | `cargo test -p contextra-checkpoint --test golden_corpus_test --locked` | Golden-Korpus (Byte-Invarianz INV-1) in `tests/golden/`. |
| CK3.2 | GRÜN | `cargo test -p contextra-checkpoint --locked` | SnapshotSource und lokale Dateinaht FsSeam implementiert. |
| CK3.3 | GRÜN | `cargo test -p contextra-checkpoint --locked` | Fehlerabbildung: CheckpointError & Fehlerkonvertierung. |
| CK3.4 | GRÜN | `cargo check -p contextra-checkpoint --all-targets --locked` | compat/ Brücke mit ports-Implementierungen (`Checkpoint`, `CheckpointCoordinator`). |
| CK3.5 | GRÜN | `cargo test -p contextra-checkpoint --locked` | Zero-Panic Härtung, Bitflip/Truncation-Resilience. |
| CK3.6 | GRÜN | `cargo clippy -p contextra-checkpoint --all-targets -- -D warnings` | Dokumentation (AGENTS.md) und Coderegeln. |

## 3. Messwerte Vorher / Nachher

| Metrik | Vorher (Baseline) | Nachher |
|---|---|---|
| LOC (contextra-checkpoint) | 3614 (12 Dateien) | 3980 (16 Dateien) |
| Nullprüfzahlen Prod (`types`/`ports`) | 41 Referenzen | Entkoppelt via `SnapshotSource`, `FsSeam` und `compat/` |
| Zero-Panic Prod Violations (Analyzer) | 39 unmarkierte Fundstellen | 0 Prod-Verstöße |
| Direkte interne Abhängigkeiten (Cargo) | `contextra-ports`, `contextra-types` | `contextra-ports`, `contextra-types` (Kern entkoppelt via `compat/`) |

## 4. StorageEngine Inventar (Aufrufe auf `storage` im Kern)
Gefundene Methoden: `get`, `put`, `commit`, `rollback`, `rollback_to_tx`, `flush`, `last_seq_no`, `last_tx_id`, `pin_checkpoint`, `unpin_checkpoint`, `delete`, `scan_prefix`, `clone`.
Exakt diese 13 Methoden bilden das Trait `SnapshotSource`.

## 5. API-Freeze Tabelle
| Externe Konsumenten | Aufgerufene Symbole | Quellkompatibilität gewährleistet durch |
|---|---|---|
| `contextra-engine` | `InstanceOrphanRegistry` | Re-Export in `lib.rs`, pub(crate) vs pub API-Freeze |
| `contextra-agent` | `StateCheckpoint`, `CheckpointGuard`, `PersistentCheckpointStore` | Voll kompatibel durch `From`/Re-Export |
| `contextra-db` | `PersistentCheckpointStore`, `global_orphan_registry`, etc. | Re-Export in `lib.rs` |
| `benchmarks` | `PersistentCheckpointStore` | Signaturkompatibel |

## 6. Externe Baseline-Schulden (Koordinationsanforderungen)
- `cargo check --workspace --all-targets --locked` schlägt fehl wegen bekannten Fehlern in fremden Crates (`contextra-store/src/sstable/block_cache.rs`, `contextra-store/src/sstable/builder.rs`, `contextra-store/src/scanner/mod.rs`).
- Liegt außerhalb des Eigentums von CKPT-3.

## 7. Nicht-Ziele & Blockierte Punkte
- Kern-Closure `ids`/`env` entfällt gemäß Auftragsanweisung ("UNABHÄNGIGKEIT").
- Umbau auf `durable-fs` (B08) blockiert.
- Zusammenlegung mit `mvcc` blockiert.
