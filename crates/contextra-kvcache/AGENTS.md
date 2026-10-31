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
