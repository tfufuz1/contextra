# contextra-graph — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §§5–7 und `docs/crates/contextra-graph.md`. **Ausgangslage:** Der 100+ Modul große GraphRAG-Altbestand (PPR, Community, LLM/Provider-Kompatibilität) enthält 168 Produktions-Panikmuster und überschreitet den autonomen E1-Scope.

1. Reduziere auf kanonischen Property-Graph, Kanten/Hyperedges, deterministische Traversal-API und Persistenz; streiche/archiviere nicht spezifizierte Provider-, LLM-, Lern- und Research-Pfade.
2. Halte nur `ids, env` als Kern-Closure; entferne höhere Ports/Types-Kompatibilität und lokale Identitätsduplikate.
3. Jede Traversal-Entscheidung prüft Gate/LiveMask für Kante und Ziel, vor Score/Fusion und Ausgabe; der aktuelle Gate-Stand schlägt jede Zeitreise.
4. Speichere Lineage für abgeleitete Knoten/Kanten; `on_gate` maskiert atomar, Purge baut betroffene Adjazenzgeneration ohne gelöschte Referenzen, Witness liefert null Residuen.
5. Ersetze jede Nicht-Test-Panic durch `GraphError` und definiere deterministische Sortierung nach IDs für alle Gleichstände.

**Abnahme:** Gate-vor-Kante-Test, Lineage-Kaskade, Generation-/Snapshot-Tests, deterministische Traversal-Golden-Dateien, Purge-Witness und Zero-Panic-Gate.
