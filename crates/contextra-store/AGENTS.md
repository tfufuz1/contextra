# AGENTS.md — `contextra-store`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-store` stellt eine hochperformante, crash-resistente Log-Structured Merge-tree (LSM) Storage Engine mit Write-Ahead-Log (WAL), MemTable (Skip-List), SSTable (Block-basiert mit Bloom-Filtern, CRC32, Block-Cache) und Background-Compaction bereit (Ring 1).
Als autonomer Speicherbaustein implementiert er den Port `StorageEngine` aus `contextra-ports`.
Im Sinne der modularen Baukasten-Architektur wurde die historische, tief verwobene Löschkette vollständig aus dem Kern entfernt: Der Store fungiert als reiner, robuster Datenhalter mit Append- und Tombstone-Semantik und exportiert standardisierte Bereinigungs-Ports (`Purger`, `LayerScanner`) für `contextra-erasure`.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 1 (Persistenz & LSM Storage Engine)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-mvcc`, `contextra-crypto` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Durability & fsync Error Propagation (T-FSYNC-1):** JEDER Aufruf von `sync_all()` und `sync_data()` MUSS Fehler strikt mit `?` propagieren. Fatale fsync-Fehler vergiften das Handle dauerhaft.
- **Single Load Rule:** In `get_at_seq()` und `scan_prefix_at()` wird `last_committed_tx` exakt EINMAL am Anfang gelesen (Snapshot Isolation Garantie).
- **Flush-before-Visible (ADR-043):** `last_committed_tx` MUSS vor dem Sichtbarmachen neuer SSTables in der Generationen-Liste aktualisiert werden.
- **TOMBSTONE_BIT-Disziplin (ADR-041):** Bit 63 strikt maskieren (`seq & !TOMBSTONE_BIT`) vor allen `max_seq`-Vergleichen.
- **WAL-HMAC Chaining & Key Derivation:** Deterministische HMAC-Verkettung pro Log-Record; Schlüsselableitung erfolgt über `contextra-crypto`.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()` in produktiven I/O- und Compaction-Pfaden.

## 4. Verbotene Muster
- **Keine Abhängigkeiten zu `contextra-core`** (Imports erfolgen direkt aus `contextra-types` und `contextra-ports`).
- Keine direkten Peer-Abhängigkeiten zu `contextra-vector` oder `contextra-text` im Produktionscode.
- Keine Multi-Level Deletion-Saga-, Rechts- oder DSGVO-Logik im Store (gehört exklusiv nach `contextra-erasure`).
- Kein Blockieren des asynchronen Tokio-Reactors durch synchrone I/O (SSTable pread läuft via `spawn_blocking`).
- Kein Ignorieren von Verzeichnis-Syncs beim atomaren File-Rename.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `StorageEngine`, `Purger`, `LayerScanner`.
- **Konsumiert:** `Clock`-Port, `Vfs`-Port (aus `contextra-durable-fs`), `contextra-mvcc` für Transaktionsverwaltung.
- **Genutzt von:** `contextra-engine`, `contextra-db`, `contextra-agent`.

## 6. Concurrency, Lock-Hierarchie & Performance
- **Lock-Ordnung (Deadlock-Prävention):**
  1. `write_lock` (`tokio::sync::Mutex` — WAL & MemTable Atomizität)
  2. `memtable` (`Arc<parking_lot::RwLock>`)
  3. `sstables` (`Arc<parking_lot::RwLock>`)
  4. `snapshot_registry` (`parking_lot::Mutex`)
- **Regel:** Niemals einen Lock in umgekehrter Reihenfolge akquirieren. Niemals einen synchronen Lock-Guard über `.await`-Punkte halten.

## 7. Tests, Sabotage & Qualitätssicherung
- **Fault-Injection & Recovery Tests:** WAL-Replay-Validierung und Crash-Simulation mitten im Flush/Compaction-Zyklus.
- **Fuzzing (REQ-QA-003):** WAL-Decoder- und SSTable-Block-Fuzzer.
- **Mutations-Testing:** Prüfung der Fehlerfortpflanzung bei sabotierten fsync-Operationen.

## 8. Offene Punkte & Technische Schulden
- DEBT-01: Bereinigung historischer Lösch-Signaturen zugunsten des reinen `Purger`-Ports.
