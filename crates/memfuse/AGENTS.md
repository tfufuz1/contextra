# MemFuse — AI-Assistenten-Kontext (`memfuse`)

## Verifizierter Codestand · Ring 4 (Hauptfassade)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `memfuse`.
> `memfuse` dient als einzige Composition Root und öffentliche Hauptfassade des Gesamtsystems.

---

## Crate-Topologie

- **Ring 4 Composition Root**:
  - `MemFuseBuilder`: Konfiguration und Assemblierung aller Subsysteme.
  - Re-Exports der primären User-Facing-APIs.
