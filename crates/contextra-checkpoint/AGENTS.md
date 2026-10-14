# AGENTS.md — contextra-checkpoint
> Ring 1 · stable · Quelle: capabilities.toml · Spec: K.5 / III.11

## 1. Zweck
Verwaltet transaktionale Checkpoints, Point-in-Time State Snapshots und Time-Travel Rollbacks über Storage-Komponenten hinweg.
Bietet den RAII-Guard `CheckpointGuard` für automatischen Rollback bei Fehlern/Drop sowie den `PersistentCheckpointStore` zur persistenten Ablage von Checkpoints mit BLAKE3-Manifest-Verifikation.
Implementiert den Trait `contextra_ports::CheckpointCoordinator` und entkoppelt synchrone Drop-Aktionen über eine verwaiste Registerverwaltung (`InstanceOrphanRegistry`).

## 2. Modul-Karte

| Datei | Verantwortung |
|---|---|
| `lib.rs` | Einstiegspunkt mit `#![deny(dead_code)]`, Modul-Deklarationen und Re-Exports von Guards, Store und Metadata |
| `guard.rs` | `CheckpointGuard` und `PinGuard` (RAII-Lebenszyklus, Rollback/Commit, Zeitstempel-Ermittlung via Clock-Port) |
| `hardlink_cloner.rs` | `CheckpointHardlinkCloner` Trait und `DefaultHardlinkCloner` für physikalisches Hardlink-Klonen von SSTables |
| `manifest.rs` | `CheckpointManifest` (Metadaten-Komponenten und BLAKE3-Prüfsummenvalidierung) |
| `meta.rs` | `CheckpointMeta` und `StateCheckpoint` (Ablagestruktur und Identifikation von Checkpoints) |
| `orphan.rs` | `InstanceOrphanRegistry` und `OrphanRegistry` (Erfassung und Recovery verwaister Pins/Checkpoints) |
| `store.rs` | `PersistentCheckpointStore` (Thread-sicheres Registrieren, Wiederherstellen und Löschen von Checkpoints) |

## 3. Invarianten

- **RAII CheckpointGuard Semantik**: Ein `CheckpointGuard` muss explizit per `commit()` oder `rollback()` finalisiert werden. Verworfene Guards werden synchron als "orphaned" registriert.
  *Test*: `cargo test -p contextra-checkpoint --lib`
- **Determinismus via Clock-Port (INV-CHECKPOINT-DETERMINISM-1)**: Zeitstempel (`timestamp_ms`) werden ausschließlich über die injizierte `contextra_ports::Clock`-Schnittstelle bezogen (kein `SystemTime::now()` im Nicht-Test-Produktivcode).
  *Test*: `cargo test -p contextra-checkpoint --lib`
- **BLAKE3-Manifest-Integrität**: `CheckpointManifest` berechnet und verifiziert Prüfsummen vor dem Laden oder Wiederherstellen von Komponenten.
  *Test*: `cargo test -p contextra-checkpoint --lib`
- **Snapshot Pinning Integration**: Persistent store pinnt Sequenznummern in der Unter-StorageEngine, um Compaction-Verwurf während Time-Travel-Optionen zu verhindern.
  *Test*: `cargo test -p contextra-checkpoint --lib`

## 4. Verboten / Anti-Patterns

- **Direkter `SystemTime::now()`-Aufruf im Nicht-Test-Code**: Bricht P28 (Determinismus). Uhr muss stets per `with_clock` oder `Clock`-Port injiziert werden.
- **Blockierendes Disk-I/O in synchronous `Drop`**: `CheckpointGuard::drop()` schreibt nicht synchron auf Disk, sondern trägt Einträge in die `InstanceOrphanRegistry` ein.
- **Verwechslung mit Store-internem Pinning**: `contextra-checkpoint` ist der einzige öffentliche Einstiegspunkt für Checkpoints; `contextra_store::lsm::guard` ist ein reines Store-Detail.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- In-Memory-Zustand (Checkpoints, Orphan-Register) wird durch `parking_lot::RwLock` oder `parking_lot::Mutex` geschützt.
- `parking_lot`-Guards werden nie über `.await`-Grenzen gehalten.
- Längere E/A-Operationen (Hardlink-Klonen, File-Flushes) nutzen `tokio::fs` bzw. `tokio::task::spawn_blocking`.

## 6. Verifikation

```bash
cargo test -p contextra-checkpoint --locked
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-checkpoint
cargo xtask check-result-dropped-io
cargo xtask determinism-check
```

## 7. Bekannte Lücken / SOLL

- **Rückwärtskompatible Hilfsfunktionen in `orphan.rs`**: Funktionen wie `await_pending_rollbacks()` und `pending_rollback_count()` dienen der Abwärtskompatibilität und verweisen auf den globalen Orphan-Registry-Fallback.
