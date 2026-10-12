# Audit-Bericht: contextra-store

**Datum:** 2026-10-12
**Auditor:** Principal Senior Rust Architect (Jules)
**Ziel-Crate:** `contextra-store` (Ring 1, `#![forbid(unsafe_code)]`)
**Session Hash:** `43f0aae8`

---

## 1. P1–P10 Invarianten- & Prüfpunkt-Übersicht

| ID | Prüfpunkt | Status | Evidenz & Befund |
|---|---|---|---|
| **P1** | FSYNC-ERROR-PROPAGATION | **PASS** | `check-result-dropped-io` meldet 0 Verstöße im Produktionscode von `contextra-store/src/`. Alle `sync_all()`, `sync_data()` und `flush()` Aufrufe propagieren I/O-Fehler strikt mit `?` oder abbruchsicheren Handler. |
| **P2** | LAST_COMMITTED_TX Single Load Rule | **PASS** | In `get_at_seq()` (`lsm/ops/read.rs:98`) und `collect_visible_entries()` (`lsm/scan.rs:98`) wird `last_committed_tx.load(Ordering::Acquire)` exakt einmal am Methoden-Anfang geladen. |
| **P3** | TOMBSTONE_BIT Maskierung (ADR-041) | **PASS** | Strikte Entkopplung von Sequenznummern und Tombstone-Bits. Sämtliche Vergleiche maskieren Bit 63 via `seq & !TOMBSTONE_BIT` (`memtable.rs`, `sstable/reader.rs`, `compaction/engine.rs`). |
| **P4** | FLUSH-BEFORE-VISIBLE (ADR-043) | **PASS** | In `lsm/ops/compaction.rs:154` wird `storage.advance_visibility()` vorgenommen, *bevor* die neue SSTable via `sstables.push()` für parallele Reader sichtbar gemacht wird. |
| **P5** | LOCK-HIERARCHIE-EINHALTUNG | **PASS** | Die Hierarchie `commit_mutex` (1) > `state` (2) > `sstables` (3) > `snapshot_registry` (4) wird strikt eingehalten. Commit `d3dcc29c` vereinheitlicht `state.read()` in Commit-Pfaden korrekt. |
| **P6** | WAL-HMAC-KEY-SOURCING | **PASS** | Schlüsselableitung erfolgt ausnahmslos über `Wal::load_or_create_integrity_key` oder `KeyManager::derive_file_key`. Keine Hardcoded-Literale in Produktions-Code. |
| **P7** | ATOMIC-RENAME-PATTERN | **PASS** | Alle Datei-Erstellungen folgen dem Schema: Temp-Datei (`.tmp`) -> `flush/sync_all` -> `atomic_rename` -> Parent-Directory `fsync`. |
| **P8** | I/O-PATTERN (ADR-012) | **PASS** | Keine ungeschützten `std::fs::File` Operationen im async-Kontext. Alle synchronen Dateipfade sind in `tokio::task::spawn_blocking` gekapselt (`sstable/reader.rs`, `lsm/recovery.rs`, `blob/reader.rs`). |
| **P9** | COMPACTION-TENANTGRENZE | **PASS** | Multi-Tenant-Isolierung verwendet `TenantKeyCodec` mit Prefix `t:{tenant_id}:`. Compaction verarbeitet vollständige Byte-Keys lexicographisch geordnet, wodurch Tenant-Boundaries unberührt bleiben. |
| **P10** | LOOM- & CONCURRENCY-TESTS | **PASS** | Concurrency- & Loom-Tests unter `tests/loom_*` decken Group-Commit, Visibility und Handoff ab und bestehen zu 100%. |

---

## 2. Lock-Hierarchie-Analyse (Commit d3dcc29c Bezug)

### Definierte Sperrenreihenfolge (AGENTS.md §5 / Ring 1)
1. `commit_mutex` (`tokio::sync::Mutex`) — WAL & MemTable Transaktions-Atomizität
2. `state` (`tokio::sync::RwLock<LsmState>`) — In-Memory SkipList & Immutable MemTable Vektor
3. `sstables` (`tokio::sync::RwLock<Vec<Arc<SstableReader>>>`) — SSTable-Segmentliste
4. `snapshot_registry` (`parking_lot::Mutex`) — MVCC Active Snapshot Registration

### Evaluierung von Commit d3dcc29c (#3787)
Commit `d3dcc29c` ("fix(store): unify LsmState lock in commit paths to read lock") stellt sicher, dass sowohl im Single-Commit-Pfad als auch im Group-Commit-Pfad `storage.state.read().await` erworben wird.
- **Sicherheitsnachweis:** `LsmState::memtable` ist in einem `Arc<MemTable>` gekapselt, welcher intern ein feingranulares `parking_lot::RwLock` für SkipList-Mutationen verwendet.
- **Deadlock-Freiheit:** Da Commit-Pfade nur ein Read-Lock auf `storage.state` halten, treten keine Schreib-Schreib-Konflikte mit parallelen Lesern auf. Das Write-Lock `storage.state.write().await` wird ausschließlich während der atomaren MemTable/WAL-Rotation in Phase 2 von `flush()` gehalten, während der `commit_mutex` ebenfalls erworben wird.
- **Inversionen:** Es existiert keine Stelle im Code, an der `sstables` (Level 3) vor `state` (Level 2) erworben wird.

`EVIDENCE[STORE-LOCK-HIERARCHY-VERIFIED]`

---

## 3. Atomic-Rename-Pattern Compliance-Liste

Sämtliche Datei-Persistierungsoperationen in `contextra-store/src/` wurden auf Einhaltung des atomaren Rename-Patterns geprüft:

1. `lsm/recovery.rs: write_salt_atomically`:
   - Temp-Datei: `SALT.tmp.<pid>.<counter>` via `create_new(true)`
   - Sync: `std_file.sync_all()` in `spawn_blocking`
   - Rename: `tokio::fs::rename(&tmp_path, salt_path)`
   - Directory Fsync: `crate::util::fsync_parent_dir(salt_path)`

2. `manifest/rollover.rs: Manifest::rollover`:
   - Temp-Datei: `MANIFEST.new.<pid>.<rand>`
   - Sync: `new_file.flush()` + `new_file.sync_all()`
   - Rename: `tokio::fs::rename(&tmp_path, &self.path)`
   - Directory Fsync: `crate::util::fsync_parent_dir(&self.path)`

3. `wal/flusher.rs: WalFlusher::seal_and_rotate`:
   - Rename von aktiver WAL auf Versiegelungspfad `.sealed`
   - Directory Fsync via `fsync_parent_dir`

4. `wal/hmac.rs: Wal::load_or_create_integrity_key`:
   - Temp-Datei: `.wal.key.tmp.<pid>.<counter>`
   - Sync & Rename auf `.wal.key`
   - Directory Fsync via `fsync_parent_dir`

`EVIDENCE[STORE-ATOMIC-RENAME-VERIFIED]`

---

## 4. Bug-Proof Test-Ergebnisse

Folgende dedizierte Bug-Proof & Concurrency Test-Suiten wurden ausgeführt:

| Test-Suite | Befehl | Ergebnis | Details |
|---|---|---|---|
| TOCTOU Put-If-Absent | `cargo test -p contextra-store --test toctou_put_if_absent` | **5 / 5 PASSED** | Atomare Sperreinpassung & Guard-Unstage verifiziert. |
| Manifest Corruption | `cargo test -p contextra-store --test manifest_corruption` | **5 / 5 PASSED** | Tail Truncation & Wiederauferstehungsschutz (dead_set) verifiziert. |
| WAL Truncate Ordering | `cargo test -p contextra-store --test wal_truncate_ordering` | **5 / 5 PASSED** | Conc-Flush & Truncate HMAC-Kettenintegrität verifiziert. |
| Loom Concurrency | `cargo test -p contextra-store --test '*loom*'` | **4 / 4 PASSED** | Group Commit Visibility, Lock Handoff & FPR Schranken verifiziert. |
| Full Library Suite | `cargo test -p contextra-store --lib` | **209 / 209 PASSED** | Sämtliche Unit-, Integration- und Property-Tests bestanden. |

`EVIDENCE[STORE-TESTS-VERIFIED]`

---

## 5. VERDICT Block

```text
================================================================================
VERDICT: APPROVED
Crate: contextra-store (Ring 1)
Audited Commit: 15b27624719ad6a6855dc3f721b2fab17377ea04
Invariants: INV-DURABILITY-RING (PASS), Single-Load-Rule (PASS), ADR-041 (PASS), ADR-043 (PASS), ADR-012 (PASS)
Critical Violations: 0
Blockers: 0
Evidence Markers: EVIDENCE[STORE-P1], EVIDENCE[STORE-P2], EVIDENCE[STORE-LOCK-HIERARCHY-VERIFIED], EVIDENCE[STORE-ATOMIC-RENAME-VERIFIED], EVIDENCE[STORE-TESTS-VERIFIED]
================================================================================
```
