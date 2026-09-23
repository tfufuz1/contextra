# Contextra — AI-Assistenten-Kontext (`contextra`)

## Verifizierter Codestand · Ring 4 (Hauptfassade)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `contextra`.
> `contextra` dient als einzige Composition Root und öffentliche Hauptfassade des Gesamtsystems.

---

## Crate-Topologie

- **Ring 4 Composition Root**:
  - `ContextraBuilder`: Konfiguration und Assemblierung aller Subsysteme.
  - Re-Exports der primären User-Facing-APIs.
