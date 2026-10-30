# ADR-0077: HNSW-Löschung per Tombstone + Slot-Nullung (VETO-F02)

> **Status-Update 30.10.2026:** Gültig. Umsetzung in `contextra-vector` bleibt maßgeblich (VETO-F02).

- **Status:** Proposed — benötigt Maintainer-Freigabe
- **Datum:** 2026-10-29
- **Autoren:** Architecture Governance Team
- **Relevante Komponenten:** `contextra-vector`, `capabilities.toml`, `governance/gates.toml`

## Kontext
Die HNSW-Vektorindex-Implementierung in `contextra-vector` muss DSGVO-konforme Löschungen unterstützen. Gemäß **VETO-F02** ist ein partielles Rewiring von Nachbarknoten während der normalen HNSW-Löschung verboten, da es die Graphtopologie inkonsistent machen und nicht-deterministische Distanzpfade erzeugen kann.

## Entscheidung
1. Löschungen im HNSW-Index erfolgen ausschließlich über Bitset-Tombstones und das anschließende Nullieren der Nutzdaten-Slots (VETO-F02).
2. Die Wiederverwendung von Slot-Nummern erfolgt erst nach vollständigem Rebuild des Index.
3. Bis zum Abschluss der Rebuild-Umstellung wird das Rebuild-Gate über das Feature `partial-index-rebuild` gesteuert.

## Konsequenzen
- **Positiv:** Vollständiger Erhalt des Determinismus und der Verifizierbarkeit des HNSW-Graphen.
- **Negativ:** Erhöhte Speicherbelegung bis zum Ausführen eines vollständigen Index-Rebuilds.
