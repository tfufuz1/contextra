# contextra-db

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. 12 Quelldateien mit Verweisen auf entfernte Crates (SPEC §16, B2). Modul `advanced` verweist auf `adapt`, `cognition` und `graph`. Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

Contextra Main Database Abstraction, Hybrid Search & Collections.

Normativitätsklasse: **Kern `[K]`** (Ring 3, `stable`; §6.0, `capabilities.toml`).


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