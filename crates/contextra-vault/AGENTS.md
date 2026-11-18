# AGENTS.md — contextra-vault — Dormant Crate (Ausgeschlossen)

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/SPEC.md §2.1 & §2.2`](file:///home/freddy/Projekte/contextra/docs/SPEC.md §2.1 & §2.2)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2.1 (Excluded Crates), §2.2 (Gate G-REACTIVATE), ADR-SCOPE-REDUCTION-001  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** Dormant (In Cargo.toml exclude)  
**Kern-Closure:** Dormant  
**Unsafe-Status:** Historisch sys-Insel  

---

## 1. Zweck & Systemrolle

Dormantes Crate: Funktionalität ist vollständig in `contextra-erasure` aufgegangen.

---

## 2. Richtlinien für Agenten

> [!CAUTION]
> **Reaktivierungsverbot:** Dieses Crate ist in der Root-`Cargo.toml` unter `exclude` eingetragen. Es darf OHNE Bestehen des formalen Gates `G-REACTIVATE` (siehe `docs/SPEC.md` §2.2) NICHT reaktiviert oder als Workspace-Member eingetragen werden!
- Sämtliche Tresor- und Schlüsselvernichtungs-Logik wurde normativ in `contextra-erasure` (`SagaVault`) konsolidiert.

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-vault` · **Ring:** – · **Version:** 0.1.0
- **Beschreibung:** Ephemerer, speichergesicherter Chunk- und Kontext-Tresor (Ebene 1, Kern-Closure sys)
- **src:** 4 Dateien, 320 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 2):** `contextra-sys`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 3):** `thiserror`, `tracing`, `zeroize`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra-types`; eingehend –
- **Features (2):** `default`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (9): `CommitReceipt`, `VaultChunk`, `VaultChunkMetadata`, `VaultConfig`, `VaultDocId`, `VaultTenantId`, `VaultTxId`, `VolatileContextVault`, `VolatileDrainReceipt`
- enum (2): `SignalModality`, `VaultError`
- fn (14): `as_u64`, `commit`, `current_size_bytes`, `ingest`, `is_empty`, `is_memory_locked`, `len`, `new`, `open`, `preview_metadata`, `purge`, `purge_memory`, `size_bytes`, `with_label`
- mod (3): `compat`, `error`, `vault`
- pub use: `error::VaultError`, `vault::{ CommitReceipt, SignalModality, VaultChunk, VaultChunkMetadata, VaultConfig, VaultDocId, VaultTenantId, VaultTxId, VolatileContextVault, VolatileDrainReceipt, *, }`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
