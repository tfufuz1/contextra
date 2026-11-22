# Contextra — AI-Assistenten-Kontext (`contextra`)

## Verifizierter Codestand · Ring 4 (Hauptfassade)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `contextra`.
> `contextra` dient als einzige Composition Root und öffentliche Hauptfassade des Gesamtsystems.

---

## Crate-Topologie

- **Ring 4 Composition Root**:
  - `ContextraBuilder`: Konfiguration und Assemblierung aller Subsysteme.
  - Re-Exports der primären User-Facing-APIs.

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: Fassade; nutzt `lsm::DurabilityMode`, `DurabilityConfigError`, `kv::delete_mode::KvDeleteMode` (5 Zeilen, 2 davon Produktivcode in `collection_profile.rs`, `performance_profile.rs`).
- Vertrag IF-8: `DurabilityMode` → `Durability` (je Change und `durability_default` je Space); `delete_mode` → `Change::delete`/`delete_range`/`Eraser` mit `EraseLevel`.
- Verboten: vor G-5 Änderungen; Hinweis: Die Fassade baut laut CLAUDE.md §5 derzeit nicht.
- Arbeitspakete: WP-5.5.
