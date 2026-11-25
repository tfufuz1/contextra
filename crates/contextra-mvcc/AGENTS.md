# AGENTS.md — contextra-mvcc
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.18, §A2, §0, §4, §20

## 1. Zweck
`contextra-mvcc` stellt Multi-Version Concurrency Control (MVCC) Primitiven, ein sequentielles Änderungsprotokoll (`SequenceLog`), Snapshot-Registrierung (`SnapshotRegistry`), Transaktions-Staging (`TxBuffer`) und Serializable Snapshot Isolation (`SequenceLogSsiValidator`) bereit.
Die Crate ist strikt synchron (P26), frei von Async-Runtimes und erzwingt `#![forbid(unsafe_code)]`.

## 2. Modul-Karte
| Datei / Verzeichnis | Verantwortung |
| :--- | :--- |
| `src/lib.rs` | Re-Exporte von `SequenceLog`, `SnapshotRegistry`, `TxBuffer`, `ReadSet`, `SequenceLogSsiValidator`. |
| `src/seq_log.rs` | Monoton steigendes Änderungsprotokoll (`SequenceLog`) für punktgenaue Visibility-Checks. |
| `src/snapshot.rs` | Snapshot-Isolation, Active-Snapshot-Pinning und automatisches Reaping verwaister Pings. |
| `src/ssi.rs` | `SequenceLogSsiValidator` und `ReadSet` für Serializable Snapshot Isolation (SSI, INV-MVCC-SSI-1). |
| `src/tx_buffer.rs` | Transaktionales Staging uncommitted Writes (`TxBuffer`), `is_key_staged_for_tx` und Orphan-Reaper. |

## 3. Invarianten
- **INV-MVCC-SSI-1:** `SequenceLogSsiValidator` bietet SSI-Read-Tracking und Pruning über `prune_through(bound_seq)` zur Verhinderung von Read-Skew und Stale-Snapshots.
- **INV-TOCTOU-PUT-IF-ABSENT:** `TxBuffer::is_key_staged_for_tx` prüft, ob ein Schlüssel in einer uncommitted Transaktion gestaged ist, um TOCTOU-Race-Conditions in `put_if_absent` atomar aufzulösen.
- **INV-DETERMINISTIC-TIME:** Verwaiste Transaktionen und Snapshots werden in Tests über zeit-injizierte Methoden (`begin_at`, `reap_orphans_at`, `min_retention_seq_at`) deterministisch bereinigt.

## 4. Verboten / Anti-Patterns
- **VERBOTEN:** `unsafe`-Code (erzwingt `#![forbid(unsafe_code)]`).
- **VERBOTEN:** Blockieren von `SnapshotRegistry`-Locks über I/O- oder Thread-Sleep-Operationen.
- **VERBOTEN:** Direkte `SystemTime::now()` Aufrufe im Produktionspfad.

## 5. Nebenläufigkeit, Async- und Lock-Regeln
- Strikt synchroner Ring-0-Code (P26). Lock-Primitiven basieren ausschließlich auf `parking_lot::{Mutex, RwLock}`.
- Sperrenreihenfolge: `SnapshotRegistry` vor `SequenceLog` vor `TxBuffer`.
- Loom-Concurrency-Tests (`loom::sync::Mutex`) sind isoliert unter `tests/loom_snapshot_registry.rs`.

## 6. Verifikation

```bash
cargo test -p contextra-mvcc
cargo test -p contextra-mvcc --test ssi_write_skew
cargo test -p contextra-mvcc --test campaign_cov_tx_buffer_reap
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-mvcc
cargo xtask check-unsafe-islands
cargo xtask check-ring0-async-purity
```

## 7. Bekannte Lücken / SOLL
- SSI-Coarsening schaltet bei Auslastung der gewachteten Commit-Keys (>= 80%) automatisch auf Grob-Granularität um (bestätigt durch `campaign_cov_ssi_coarsening.rs`).
