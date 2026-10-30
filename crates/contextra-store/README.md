# contextra-store

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Verweise auf `contextra-core` (Trait-Herkunft) und auf die entfernte Scanner-Logik (SPEC §16, B2). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

LSM-Tree Storage Engine, WAL Durability und Compaction Engine.

Normativitätsklasse: **Kern `[K]`** (Ring 1, `stable`; §6.0, `capabilities.toml`).


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