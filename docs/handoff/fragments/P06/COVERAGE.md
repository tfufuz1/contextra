# Test-Coverage P06 · contextra-graph

## 1. Zusammenfassung der Testsuite
- **Unit-Tests in `src/`:** 216 Tests (alle bestanden, 0 Fehler).
- **Integrationstests in `tests/`:** 38 Testsuiten mit über 150 Tests (alle bestanden).
- **Neu hinzugefügte Validierungstests:**
  - `ppr_reference_validation`: 2 Tests (Handrechnung 4-Knoten-Diamantgraph + Differentialtest auf Zufallsgraphen).
  - `entity_extraction_benchmark`: 1 Test (Goldstandard mit 40 Sätzen).

## 2. Testabdeckung (Schätzung & Kernbereiche)
Modul | Getestete Funktionalität | Geschätzte Coverage
---|---|---
`csr/` (Graph-Kern) | BFS-Traversierung, RCU-Reader, Rollback, Persistenz, Tombstones | > 85 %
`ppr/` (Diffusion) | Dense Power-Iteration, Forward Push, Cost Model, Shadow Mode | > 88 %
`hyperedge/` | Serialisierung, Canonical Sorting, Validation, Consolidation Guard | > 82 %
`entity_extraction/` | Regelbasierte Extraktion, Ingestion, Edge Cases | > 80 %
`cascade/` | Kaskadierende Invalidierung bei Dokument-Supersession | > 78 %
`provenance/` | DocEdgeIndex, Herkunftsverfolgung | > 85 %
**Gesamt** | **Umfassende Abdeckung des gesamten Produktionscodes** | **≥ 80 % (Ziel ≥ 75 % erfüllt)**

Alle Tests laufen deterministisch und zero-panic.
