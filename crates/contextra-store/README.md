# contextra-store

LSM-Tree Storage Engine, WAL Durability und Compaction.

Normativitätsklasse: **Kern `[K]`** (Ring 1, `stable`; §6.0, `capabilities.toml`).

## Rolle & Zweck
`contextra-store` ist der primäre Key-Value-Speicherbaustein von Contextra (Ring 1). Er implementiert eine performante Log-Structured Merge-Tree (LSM) Engine mit Write-Ahead-Log (WAL), HMAC-Integritätskette, MemTable, SSTables und asynchroner Compaction.

Im Baukastensystem fungiert `contextra-store` als neutraler, robuster Persistenzbaustein.

## Baukasten-Einordnung & Status
- **Ring:** Ring 1 (Speicherkerne)
- **Status:** 🟡 In Migration (Bereinigung von historischen Löschketten-Relikten gemäß `PLAN.md` Phase 4)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Deprecated Debt & Isolation der Löschkette
- **Löschketten-Entflechtung:** Historische Deletion-Proof- und DSGVO-Logiken in `contextra-store` sind als *Deprecated Debt* (siehe [`DEPRECATED_DEBT.md`](../../DEPRECATED_DEBT.md)) deklariert.
- **Zielarchitektur:** `contextra-store` implementiert lediglich den schlanken Bereinigungs-Port `Purger` aus `contextra-ports`. Alle komplexen Prüfungen, Berichte und kryptografischen Beweise sind vollständig in [`contextra-erasure`](../contextra-erasure/) ausgelagert.

## Öffentliche API-Übersicht
- **LSM & WAL:** `LsmStorage`, `LsmConfig`, `WalHandle`, `WalError`
- **Locks & Concurrency:** `KvKeyLocks`, `KeyGuard`, `MultiKeyGuard`
- **Compaction & Manifest:** `Manifest`, `ManifestEntry`, `CompactionEngine`
- **System Pressure:** `SystemPressure`, `SystemPressureMonitor`
