// FILE-CONTEXT: Entwicklerhandbuch und Governance-Vertrag für contextra-store
// ZWECK: Regeln, Invarianten, Abhängigkeiten und Modulkarte für Coding-Agenten
// INVARIANTEN: #![forbid(unsafe_code)], Zero-Panic, Fail-Closed bei fsync, INV-STORE-1 bis INV-STORE-5
// STAND: 2026-11-11

# contextra-store — Entwicklerhandbuch & Governance

---

> **Crate:** `contextra-store` | **Zweck:** Autarker LSM-Speicherkern
> **Normative Vorgabe:** `docs/CONTEXTRA_IMPLEMENTIERUNGS_SPEC.md`, `crates/contextra-store/SPEC.md`

---

## 1. Crate-Steckbrief & Features

| Merkmal | Wert |
|---|---|
| Zweck | Autarker LSM-Speicherkern für persistente Schlüssel-Wert-Speicherung |
| Abhängigkeiten (`Cargo.toml`) | `contextra-ids`, `contextra-env`, `contextra-sys`, `contextra-mvcc`, `contextra-crypto` (optional: `contextra-types`, `contextra-ports`) |
| Features | `default`, `workspace`, `inspect`, `block-cache-v2`, `memory-only-storage`, `docid-128`, `legacy-wal-key`, `fault-injection`, `test-hooks`, `sieve-cache` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

---

## 2. Architektur & Modularisierung

- **Entkopplung:** Der Kern von `contextra-store` hängt direkt nur an Micro-Cores (`contextra-ids`, `contextra-env`) und Systemschichten (`contextra-sys`, `contextra-crypto`, `contextra-mvcc`).
- **Optionale Workspace-Brücken:** Integrationen mit `contextra-types` und `contextra-ports` liegen isoliert im Feature `workspace` (`src/compat/`).
- **Autarke I/O-Naht:** `src/io_seam/` (`types.rs`, `real_fs.rs`, `store_io.rs`) und `src/traits.rs` stellen lokale Abstraktionen für Dateisysteminteraktionen bereit.
- **Berichte der Modularisierung:** Verweise auf die Abschlussberichte der Teilkomponenten liegen unter `crates/contextra-store/docs/modularization/<KOMPONENTE>/REPORT.md` (z. B. Komponente G unter `docs/modularization/G/REPORT.md`).

---

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `error.rs` | Lokaler Fehlertyp `StoreError` (`#[non_exhaustive]`) und `StoreResult<T>` |
| `traits.rs` | Autarke Speicher- und Dateisystem-Traits (`StoreFs`, `StoreFile`) |
| `io_seam/` (`types`, `real_fs`, `store_io`) | Modulare VFS-Naht und atomare I/O-Hülle (`Vfs`, `RealFs`, `StoreIo`) |
| `fsync.rs` | Zentrale Datei- und Verzeichnis-Sync-Stelle (INV-FSYNC-1) |
| `key_source.rs` | Schlüsselmaterial-Quellen für WAL und Chiffrierung |
| `budget.rs` | Ressourcen- und Speicherbudget-Tracker (`ResourceTracker`) |
| `stats.rs` | Speicherstatistiken (`StorageStats`) |
| `util.rs` | Siegel- und Authentifizierungs-Envelope (`RecordSealer`, `SealedEnvelope`) |
| `wal/` (`encode`, `flusher`, `hmac`, `io`, `replay`) | Append-Only WAL, HMAC-Kette, asynchroner Flusher-Actor, Replay, Open-Heal |
| `memtable.rs` | Nebenläufige, geschardete BTreeMap mit Sequenznummern und Tombstones |
| `sstable/` (`builder`, `reader`, `block_cache`, `bloom`, `stream`, `block_search`, `reader_ext`, `io`) | Sortierte Segmente auf Platte (Format V4 Standard), Block-Cache, Bloom-Filter |
| `manifest/` (`core`, `entry`, `recovery`, `rollover`) | Transaktionales Manifest für SSTable-Generationen und Versionierung (`MFMN`) |
| `compaction/` (`engine`, `adaptive`, `config`, `merge_operator`, `retention`) | Leveled und Adaptive Compaction-Engine, `MergeOperator` |
| `system_pressure.rs` | Systemdruck-Monitoring für adaptive Compaction |
| `tenant_codec.rs` | Mandanten-Präfixierung (`TenantId`) und Schlüssel-Isolation |
| `lsm/` (`engine`, `commit`, `flush`, `group_commit`, `guard`, `recovery`, `scan`, `ops/*`, `validate`, `observer`) | Orchestrierung, Transaktions-Commit, Group-Commit Leader/Follower, Recovery |
| `compat/` (`engine`, `scanner`, `error`, `license`) | Optionale Brücke zu Workspace-Typen und -Ports (Feature `workspace`) |

---

## 4. Verträge & Invarianten

1. **`fsync`-Fehler propagieren** (`?`), nie `let _ =`; nach Fehler Handle vergiften (INV-FSYNC-1 / INV-STORE-1). `fsync.rs` ist die zentrale Stelle für Datei- und Verzeichnis-Sync.
2. **Atomares Schreiben:** tmp-Datei → `fsync` → atomarer `rename` → Parent-Directory `fsync`.
3. **Tombstone-Disziplin:** Bit 63 maskieren (`seq & !TOMBSTONE_BIT`) vor jedem `max_seq`-Vergleich (INV-STORE-4).
4. **Single-Load-Regel:** `last_committed_tx` in `get_at_seq`/`scan_prefix_at` genau einmal zu Beginn der Operation laden (INV-STORE-5). **Flush-before-Visible:** Manifest-Commit muss vor Sichtbarkeit der SSTable abgeschlossen sein.
5. **WAL-HMAC-Schlüssel** nur über `load_or_create_integrity_key` (`wal/hmac.rs`), nie hartcodiert; `legacy-wal-key` nur für Migration.
6. **Lock-Handoff Invariante (P06 / ADR-0XX-P0A):** Beim Group-Commit wird `truncate_lock` akquiriert, **bevor** `commit_mutex` freigegeben wird.
7. **Modulare Isolation:** Multi-Layer Deletion, DSGVO- und Deletion-Proof-Logik liegen exklusiv in `contextra-erasure`.

---

## 5. Ports

- **Implementiert:** `StorageEngine`, `MergeOperator`.
- **Konsumiert:** `Clock`, `Vfs`, `MetricsSink`.

---

## 6. Nebenläufigkeit & Sperrenordnung

- **Sperrenordnung:**
  `commit_mutex` → `wal.truncate_lock` → `memtable_shard_lock` → `sstable_manifest_lock`.
  Niemals `parking_lot`-Guards über `.await`-Punkte halten (`clippy::await_holding_lock`).
- **Group-Commit:** Message-Passing und Leader/Follower-Batching über Flusher-Actor.

---

## 7. ANCHOR-Ids (Kollisionsschutz)

Workspace-weit eindeutige Anchors nach dem Muster `ANCHOR[contextra-store:<BEREICH>-<NNN>]`. Komponente G vergibt IDs im reservierten Bereich:
`ANCHOR[contextra-store:ROOT-UTIL-100]` bis `ANCHOR[contextra-store:ROOT-UTIL-199]`.

---

## 8. Verifikationsbefehle

```bash
cargo check -p contextra-store --locked
cargo test -p contextra-store --test golden_corpus
cargo test -p contextra-store --test mz_g_golden_invariants
```
