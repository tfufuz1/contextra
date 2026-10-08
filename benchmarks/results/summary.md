# Contextra — Retrieval Accuracy Benchmark Report

Hinweis: Diese Zahlen messen nur Speicher- und Indexlatenz bzw. Retrieval-Genauigkeit auf dem genannten Testkorpus. Embedding-Inferenz addiert 10 bis 500 ms. Der Testkorpus umfasst 50 Chunks und 10 Abfragen; die Zahlen sind nicht auf größere Bestände extrapolierbar.

**Stand / Zeitstempel**: `2026-09-03T10:00:00Z`
**Testkorpus**: 50 Dokument-Chunks, 10 Testabfragen

## Zusammenfassung der Messergebnisse

| Szenario | Modus | Recall@1 | Recall@3 | Recall@5 | MRR | Fehlerrate@1 | Delta (Recall@1) | Delta (Fehler) |
|---|---|---|---|---|---|---|---|---|
| **Szenario A**: Kontext-Präfix | Baseline (Ohne) | 80.0% | 80.0% | 80.0% | 0.800 | 20.0% | - | - |
| | Mit Kontext-Präfix | 80.0% | 80.0% | 80.0% | 0.800 | 20.0% | **+0.0%** | **-0.0%** |
| **Szenario B**: Reranking | Standard RRF (Ohne) | 0.0% | 0.0% | 0.0% | 0.000 | 0.0% | - | - |
| | Mit Cross-Encoder | 0.0% | 0.0% | 0.0% | 0.000 | 0.0% | **n/a — Mechanismus nicht aktiv/nicht verdrahtet** | **n/a — Mechanismus nicht aktiv/nicht verdrahtet** |
