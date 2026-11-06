# contextra-db

Contextra Collection- und Hybrid-Search-Orchestrierungsbaustein.

Normativitätsklasse: **Kern `[K]`** (Ring 3, `stable`; §6.0, `capabilities.toml`).

## Rolle & Zweck
`contextra-db` stellt eine modulare Collection- und Hybridsuch-API für Contextra bereit (Ring 3). Der Crate fungiert als High-Level Orchestrator, der Vektor-, Volltext- und Metadaten-Indizes zu einem kohärenten Speicher vereint.

Im Zuge der Transformation zum modularen Baukastensystem wird `contextra-db` von einem monolithischen Kern zu einem schlanken Koordinationsbaustein refaktorisiert.

## Baukasten-Einordnung & Status

- **Ring:** Ring 3 (Anwendungskern / Orchestrator)
- **Status:** 🟡 In Migration (schrittweise Entkopplung in autarke Bausteine: `contextra-engine`, `contextra-cognition`, `contextra-privacy`, `contextra-rank`, `contextra-adapt`)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Deprecated Debt & Entkopplung
- **Historische Löschkette:** Sämtliche früheren direkten Verknüpfungen mit DSGVO-, Nachweis- und Löschkettenlogiken sind als *Deprecated Debt* (siehe [`DEPRECATED_DEBT.md`](../../DEPRECATED_DEBT.md)) deklariert. Diese Aufgaben werden vollständig von [`contextra-erasure`](../contextra-erasure/) übernommen.
- **Lokale API-Deprecations:** Ältere Suchsignaturen und Filtertypen sind in [`DEPRECATED_DEBT.md`](DEPRECATED_DEBT.md) detailliert erfasst und bleiben bis zum Ende der Übergangsphase abwärtskompatibel.

## Öffentliche API-Übersicht

- **Hauptinstanz & Collections:** `Contextra`, `Collection`
- **Retrieval & Fusion Results:** `SearchResult`, `ProvenanceRecord`, `SignalContribution`
- **Konsolidierung & Maintenance:** `ConsolidationEngine`, `execute_sleep_cycle`, `run_consolidation_pass`
- **Kontext-Kompaktierung:** `ContextCompactor`, `CompactedContext`, `CompactionStrategy`

## Baukasten-Nutzung

```rust
use contextra_db::{Contextra, ContextraConfig};

let db = Contextra::open_with_config(path, config).await?;
let collection = db.collection("documents").await?;
```
