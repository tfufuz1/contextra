# AGENTS.md — Contextra Bench

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-bench` ist das eigenständige, reproduzierbare Benchmark- und Evaluierungs-Harness für das Contextra-Baukastensystem.
Es misst empirisch die Retrieval-Genauigkeit (Recall@K, MRR, nDCG), Latenzen (p50, p95, p99) und Durchsatzraten über alle Bausteine (LSM-Store, HNSW-Vektorsuche, BM25-Volltext, RRF-Fusion, Reranking und Inferenz) hinweg.
Als isolierter Messbaustein konsumiert er Komponenten ausschließlich über standardisierte Ports und stellt sicher, dass Performance-Regressionen sofort aufgedeckt werden.

## 2. Invarianten & Messdisziplin
- **Reproduzierbare Korpora:** Alle Benchmarks laufen auf deterministischen, synthetischen oder standardisierten Evaluierungskorpora (LoCoMo, LongMemEval, BEIR) mit annotierter Ground Truth.
- **Unveränderliche Baselines:** Historische Baseline-Dateien (`baseline_metrics.json`, `baseline_recall.json`) dürfen niemals unautorisiert überschrieben werden.
- **Duale Ausgabe:** Alle Messergebnisse werden strukturiert sowohl als maschinenlesbares JSON (`results.json`) als auch als Markdown-Report ausgegeben.
- **Strikte Sicherheit:** `#![forbid(unsafe_code)]` wird über das gesamte Crate hinweg erzwungen.
- **Zero-Panic-Doktrin:** Saubere Fehlerbehandlung auch in Benchmark-Pfaden; Vermeidung von unkontrollierten Panics.

## 3. Verbotene Muster
- Direkte Abhängigkeiten zu internen Implementierungsdetails (Zugriff ausschließlich über `contextra-types` und `contextra-ports`).
- Verwendung von `contextra-core`.
- Nicht-deterministische Messläufe ohne kontrollierte Seed-Initialisierung.

## 4. Schnittstellen & Datenfluss
- **Konsumiert:** `contextra-types`, `contextra-ports`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-db`.

## 5. Verifikation & Reporting
- Automatisiertes Ausführen via `cargo bench -p contextra-bench`.
- Generierung standardisierter Performance- und Recall-Berichte.
