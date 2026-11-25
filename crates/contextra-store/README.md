# contextra-store

`contextra-store` ist die LSM-Tree Storage Engine von Contextra (Ring 1).

## Zweck

Verwaltet persistenten Key-Value-Speicher mit Write-Ahead-Log (WAL), HMAC-Integritätskette, MemTable, SSTables, `KvKeyLocks` und Compaction.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 1 (Persistenz-Engine)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **LSM & WAL:** `LsmStorage`, `LsmConfig`, `WalHandle`, `WalError`
- **Key-Granulare Locks:** `KvKeyLocks`, `KeyGuard`, `MultiKeyGuard`
- **Manifest & Compaction:** `Manifest`, `ManifestEntry`, `CompactionEngine`
- **Pressure Monitoring:** `SystemPressure`, `SystemPressureMonitor`

## Architektur & Verweise

Details zur Storage-Engine und Lock-Hierarchie finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §5.
