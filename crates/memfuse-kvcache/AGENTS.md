# MemFuse — AI-Assistenten-Kontext (`memfuse-kvcache`)

## Verifizierter Codestand · Ring 1 (State & Storage)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `memfuse-kvcache`.
> `memfuse-kvcache` implementiert Prefix-Radix-Bäume, KV-Blöcke, Tiering, AEAD-Verschlüsselung und Segmentdateien für das KV-Cache-Management.
> Er erzwingt `#![forbid(unsafe_code)]`.

---

## Crate-Topologie

- **Ring 1 Komponente**:
  - Prefix-Radix-Baum & KV-Block-Management
  - Tenant-isoliertes Caching & Tiering
  - Integration mit `memfuse-crypto` für KV-Segment-Sicherheit
