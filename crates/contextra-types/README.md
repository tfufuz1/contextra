# contextra-types

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Restmodul `types/saos.rs` stammt aus dem entfernten Agenten-Kontext (SPEC §16, B7). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

Grundlegende Typdefinitionen (DocId, TxId, TenantId, Error-Typen).

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-types` definiert die zentralen, produkt- und I/O-unabhängigen Domänentypen, Identifikatoren und Fehler-Enums des Gesamtsystems.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Typen-Kern, deterministisch, kein `tokio`, I/O-frei)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `(keine)` (Spec §6.9, `capabilities.toml`)
