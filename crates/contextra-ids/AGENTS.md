# AGENTS.md — contextra-ids — Identitätstypen & Referenzen

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-ids.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-ids.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §3.2 (Kapselung personenbezogener Daten), §4 (Schlüsselhierarchie), §13 (Invarianten)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Fundamentaler Mikro-Kernel für Identifikatoren, Mandanten-IDs, Referenzen und Sequenznummern.

---

## 2. Kern-Richtlinien für Agenten

- **Kanonischer Typ-Besitz:** Besitzt kanonisch `DocId`, `SubjectRef`, `PersonalRef`, `TenantId`, `CollectionId`, `TxId`, `ErasureId`, `TOMBSTONE_BIT`, `MAX_SEARCH_K`. Keine Duplikate in anderen Crates zulässig (R4).
- **Layout-Prüfung:** `size_of::<DocId>() == 8` (bzw. `16` unter Feature `docid-128`).
- **Sicherheits-Eigenschaften für `SubjectRef`:**
  - MUSS fehlbar über `rand::rngs::OsRng::try_fill_bytes` erzeugt werden.
  - KEIN `Clone` zulässig.
  - `ZeroizeOnDrop` zwingend implementiert.
  - Kein Klartext-`Display` (Verhinderung von Log-Leaks).
- **Mandanten-Isolation:** `TenantId(0)` ist fest für `TenantId::SYSTEM` reserviert. Reguläre IDs müssen > 0 sein.

## 3. Test-Pflichten
- T-ID1 bis T-ID6 (RNG-Erschöpfung, Zeroize bei Drop, BLAKE3-Präfix-Determinismus, TenantId(0)-Reservierung, CollectionId(0)-Ablehnung, Layout-Assertions).

---

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

<!-- END GENERATED: crate_inventory.py -->
