# AGENTS.md — `contextra-vault`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-vault` · **Ring:** – · **Version:** 0.1.0
- **Beschreibung:** Ephemerer, speichergesicherter Chunk- und Kontext-Tresor (Ebene 1, Kern-Closure sys)
- **src:** 4 Dateien, 320 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 2):** `contextra-sys`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 3):** `thiserror`, `tracing`, `zeroize`
- **Reverse-Abhängigkeiten (1):** `contextra-db`
- **Kopplung an Hub-Crates:** ausgehend `contextra-types`; eingehend `contextra-db`
- **Features (2):** `default`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (9): `CommitReceipt`, `VaultChunk`, `VaultChunkMetadata`, `VaultConfig`, `VaultDocId`, `VaultTenantId`, `VaultTxId`, `VolatileContextVault`, `VolatileDrainReceipt`
- enum (2): `SignalModality`, `VaultError`
- fn (14): `as_u64`, `commit`, `current_size_bytes`, `ingest`, `is_empty`, `is_memory_locked`, `len`, `new`, `open`, `preview_metadata`, `purge`, `purge_memory`, `size_bytes`, `with_label`
- mod (3): `compat`, `error`, `vault`
- pub use: `error::VaultError`, `vault::{ CommitReceipt, SignalModality, VaultChunk, VaultChunkMetadata, VaultConfig, VaultDocId, VaultTenantId, VaultTxId, VolatileContextVault, VolatileDrainReceipt, *, }`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->
