# AGENTS.md — `contextra-ids`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-ids` · **Ring:** – · **Version:** 0.1.0
- **Beschreibung:** Canonical identity and tenant types for the Contextra ecosystem
- **src:** 1 Dateien, 501 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 3):** `blake3`, `serde`, `thiserror`
- **Reverse-Abhängigkeiten (8):** `contextra-checkpoint`, `contextra-crypto`, `contextra-graph`, `contextra-mvcc`, `contextra-store`, `contextra-text`, `contextra-types`, `contextra-vector`
- **Kopplung an Hub-Crates:** ausgehend –; eingehend `contextra-types`
- **Features (3):** `default`, `docid-128`, `std`

### Öffentliche API (grob, aus src/)
- struct (11): `CollectionId`, `DocId`, `DocRef`, `EntityId`, `FileId`, `Seq`, `StoreId`, `TenantId`, `TenantScoped`, `Timestamp`, `TxId`
- enum (2): `IdError`, `TenantScopeViolation`
- fn (21): `as_bytes`, `as_micros`, `as_u64`, `from_doc_id`, `from_key`, `from_micros`, `inner`, `internal`, `into_inner`, `into_inner_checked`, `is_internal`, `is_system`, `is_valid`, `is_valid_origin`, `map` … (+6)
- type (1): `Result`
- const (12): `DEFAULT`, `ID_WIDTH_BITS`, `INTERNAL_BASE`, `INVALID`, `MAX`, `MAX_COLLECTION_SEQUENCE`, `MAX_REF_LEN`, `MAX_SEARCH_K`, `MIN`, `SYSTEM`, `TOMBSTONE_BIT`, `ZERO`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->
