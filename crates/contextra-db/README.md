# contextra-db

`contextra-db` stellt die zentrale Orchestruierungs- und Collection-API für Contextra bereit (Ring 3).

## Zweck

Haupt-Monolith für Collection-Verwaltung, 4-Signal-Hybridsuche, Multi-Step-Retrieval, Kontext-Kompaktierung und Speicher-Konsolidierung.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 3 (Anwendungskern / Orchestrator)
- **Status:** 🟡 In Migration (wird schrittweise in `engine`, `cognition`, `privacy`, `rank`, `adapt` zerlegt)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Hauptinstanz & Collections:** `Contextra`, `Collection`
- **Retrieval & Fusion Results:** `SearchResult`, `ProvenanceRecord`, `SignalContribution`
- **Konsolidierung & Workers:** `ConsolidationEngine`, `execute_sleep_cycle`, `run_consolidation_pass`
- **Kontext-Kompaktierung:** `ContextCompactor`, `CompactedContext`, `CompactionStrategy`

## Architektur & Verweise

Details zum Zerlegungsplan von `contextra-db` gemäß Strangler-Muster finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §20 (Phase 3b).
