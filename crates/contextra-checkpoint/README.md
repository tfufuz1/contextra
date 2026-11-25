# contextra-checkpoint

`contextra-checkpoint` verwaltet Time-Travel Checkpoints und Snapshots für Contextra (Ring 1).

## Zweck

Ermöglicht deterministische Wiederherstellung von Zuständen, Session-Snapshots und RAII-gesicherte Transaktions-Rollbacks über eine saubere Port-Schnittstelle ohne globalen Zustand.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 1 (Persistenz)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Snapshot Guards:** `CheckpointGuard`, `PinGuard`
- **Manifest & Metadaten:** `CheckpointManifest`, `CheckpointMeta`, `StateCheckpoint`
- **Registry:** `PersistentCheckpointStore`

## Architektur & Verweise

Details zur Checkpoint-Architektur (gemäß ADR-011) finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §5.
