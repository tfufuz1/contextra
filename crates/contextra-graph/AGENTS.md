# AGENTS.md — `contextra-graph`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-graph` stellt eine performante Compressed Sparse Row (CSR) Graph-Engine für Entitätsbeziehungen, GraphRAG-Traversierung und Community Detection bereit (Ring 0).
Als autarker Baustein implementiert er den standardisierten Port `GraphIndex` aus `contextra-ports`.
Er bietet lock-freie Lesepfade und Delta-Puffer für hochparallele Graph-Abfragen und ist vollständig unabhängig von konkreten Storage-Engines nutzbar.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Graph-Engine & CSR-Indizierung)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Kausalitäts-Erhaltung:** Entitäts- und Kanten-IDs korrespondieren 1:1 mit deterministischen Transaktions- und Dokument-IDs.
- **Lock-freier Lesepfad:** CSR-Repräsentation ermöglicht parallele Graph-Traversierung ohne Reader-Locks.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; robuste Zyklen- und Tiefenbegrenzung bei Graph-Traversierungen.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]` strikt erzwungen).
- **Keine Querabhängigkeiten zu höheren Schichten** (insbesondere keine Bindung an `contextra-adapt` oder `contextra-store`).
- Keine unbegrenzte Rekursion bei zyklischen Graphen (strikte `max_hops` Begrenzung).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `GraphIndex` (`CsrGraph`).
- **Konsumiert:** `contextra-types`, `contextra-ports`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Zyklus- & Stress-Tests:** Traversierungs-Prüfungen auf dichten und zyklischen Graphen zur Verhinderung von Stack-Overflows.
- **Fuzzing (REQ-QA-003):** CSR-Kanten- und Knoten-Mutations-Fuzzer.
- **Sabotage-Testing:** Prüfung der Traversierungs-Abbrüche bei inkonsistenten Kanten-Verweisen.

## 7. Offene Punkte & Technische Schulden
- DEBT-04: Eliminierung historischer temporärer Dev-Dependencies zu `contextra-store`.
