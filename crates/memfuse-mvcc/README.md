# memfuse-mvcc

`memfuse-mvcc` stellt Multi-Version Concurrency Control (MVCC) und Sequence Logs bereit (Ring 0).

## Zweck

Sichert Lese- und Schreibisolation ohne globale Mutexes:
- `SequenceLog`: Monotone Sequenznummerierung und Change Notifications.
- `SnapshotRegistry` / `SnapshotGuard`: MVCC Leseisolation.
- `TxBuffer`: Sharded Transaktionsstaging mit Orphan Reaping.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Synchroner Kern, Concurrency Control)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Sequence Logging:** `SequenceLog`, `SeqLogEntry`, `SeqLogChange`
- **Snapshots:** `SnapshotRegistry`, `SnapshotGuard`
- **Transaction Buffer:** `TxBuffer`, `IndexOp`

## Architektur & Verweise

Details zur MVCC-Architektur finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §4.2.
