# contextra-types

Grundlegende Typdefinitionen (DocId, TxId, TenantId, Error-Typen).

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-types` definiert die zentralen, produkt- und I/O-unabhängigen Domänentypen, Identifikatoren und Fehler-Enums des Gesamtsystems.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Typen-Kern, deterministisch, kein `tokio`, I/O-frei)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `(keine)` (Spec §6.9, `capabilities.toml`)
