# contextra-types

Grundlegende Domänentypen, Identifikatoren und Fehlerstrukturen.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, `capabilities.toml`).

## Rolle & Zweck
`contextra-types` ist die Basis aller Typdefinitionen im gesamten Contextra-Ökosystem. Es enthält produkt- und I/O-unabhängige Domänentypen wie Identifikatoren (`DocId`, `TxId`, `TenantId`), Token-Budgets, Tags und kanonische Fehler-Enums.

Der Crate ist vollständig autark, I/O-frei, deterministisch und enthält keine externen Laufzeit-Abhängigkeiten wie `tokio`.

## Baukasten-Einordnung & Status
- **Ring:** Ring 0 (Typen-Kern)
- **Status:** 🟢 Autarker Baustein
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Abhängigkeiten:** Keine internen Workspace-Abhängigkeiten (0 Deps).
