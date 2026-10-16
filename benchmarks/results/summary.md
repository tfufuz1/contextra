# Contextra — Retrieval Accuracy Benchmark Report

**Stand / Zeitstempel**: `2026-09-03T10:00:00Z`
**Testkorpus**: 50 Dokument-Chunks, 10 Testabfragen

## Zusammenfassung der Messergebnisse

| Szenario | Modus | Recall@1 | Recall@3 | Recall@5 | MRR | Fehlerrate@1 | Delta (Recall@1) | Delta (Fehler) |
|---|---|---|---|---|---|---|---|---|
| **Szenario A**: Kontext-Präfix | Baseline (Ohne) | 80.0% | 80.0% | 80.0% | 0.800 | 20.0% | - | - |
| | Mit Kontext-Präfix | 80.0% | 80.0% | 80.0% | 0.800 | 20.0% | **+0.0%** | **-0.0%** |
| **Szenario B**: Reranking | Standard RRF (Ohne) | 60.0% | 60.0% | 60.0% | 0.600 | 40.0% | - | - |
| | Mit Cross-Encoder | 60.0% | 60.0% | 60.0% | 0.600 | 40.0% | **+0.0%** | **-0.0%** |

---

## Skalierungslauf (100.000 Dokumente) Messung

**Stand / Zeitstempel**: `2026-09-30T21:10:00Z`
**Bericht**: [`docs/reports/scale_100k_measurement_2026-09-30.md`](../../docs/reports/scale_100k_measurement_2026-09-30.md)
**Rohdaten**: [`benchmarks/results/scale_100k_2026-09-30.json`](scale_100k_2026-09-30.json)

Der gemessene 100k-Skalierungslauf im Release-Profil brach vor Abschluss des ersten 10.000er Intervalls bei 0 verarbeiteten Intervallen ab (`Transaction error: Storage error: Memory budget exceeded (95%)`), da das Speicherbudget der Standardkonfiguration (2048 MB) während des Befüllens überschritten wurde. Für Details siehe den verlinkten Messbericht.
