# Contextra — AI-Assistenten-Kontext (`contextra`)

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Build-Blocker: `contextra_core` und Feature `candle` (SPEC §16, B3). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

## Verifizierter Codestand · Ring 4 (Hauptfassade)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `contextra`.
> `contextra` dient als einzige Composition Root und öffentliche Hauptfassade des Gesamtsystems.

---

## Crate-Topologie

- **Ring 4 Composition Root**:
  - `ContextraBuilder`: Konfiguration und Assemblierung aller Subsysteme.
  - Re-Exports der primären User-Facing-APIs.
