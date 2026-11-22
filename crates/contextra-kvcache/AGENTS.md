# Contextra — AI-Assistenten-Kontext (`contextra-kvcache`)

## Verifizierter Codestand · Ring 1 (State & Storage)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `contextra-kvcache`.
> `contextra-kvcache` implementiert Prefix-Radix-Bäume, KV-Blöcke, Tiering, AEAD-Verschlüsselung und Segmentdateien für das KV-Cache-Management.
> Er erzwingt `#![forbid(unsafe_code)]`.

---

## Crate-Topologie

- **Ring 1 Komponente**:
  - Prefix-Radix-Baum & KV-Block-Management
  - Tenant-isoliertes Caching & Tiering
  - Integration mit `contextra-crypto` für KV-Segment-Sicherheit

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: kein direkter Bezug zum Storage-Umbau. Es nutzt `contextra-store` nicht im Code (Stand 22.11.2026) und ändert sich bis G-6 nicht. Bei Fragen zum Kern gilt `docs/SPEC.md` (R8).
