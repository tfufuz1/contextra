# contextra-bench — Crate-Spezifikation (Benchmark-Harness & Regressions-Gates)

**Ebene:** E3 (Endpunkte / Benchmarks)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `benchmarks/contextra-bench`  
**Referenz:** `docs/SPEC.md` §2, §12

---

## 1. Zweck & Systemrolle

`contextra-bench` ist die offizielle, automatisierte Benchmark- und Regressionsprüf-Harness für Contextra:
- **Genauigkeits-Evaluation:** Reproduzierbare Messung von Retrieval-Genauigkeit (Recall@K, MRR@K, NDCG@K, MAP@K) über standardisierte Datensätze (BEIR-Benchmark, synthetische Korpora).
- **Latenz- & Durchsatz-Profiling:** Messung von Latenz-Perzentilen (p50, p95, p99, p99.9) für Vektor-, Volltext- und Hybrid-Suchanfragen unter variierender nebenläufiger Last.
- **Regressions-Gates:** Automatischer CI-Abbruch bei Performance-Einbußen oder Qualitätsverlusten gegenüber festgelegten Baselines (`baseline_metrics.json`).
- **Vergleich mit Referenz-Systemen:** Standardisierte Vergleichsadapter gegen externe Engines (z.B. LanceDB, Qdrant, Tantivy, SQLite/Redb).

---

## 2. Architektur & Module

```
┌─────────────────────────────────────────────────────────────┐
│                       contextra-bench                       │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                    BenchmarkRunner                  │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ AnnBenchmarks      │    │ BeirEvaluator     │   │   │
│   │   │ (HNSW vs Flat)     │    │ (BEIR Datasets)   │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ PathRagSweep       │    │ CompetitiveKv     │   │   │
│   │   │ (Graph Traversal)  │    │ (Store vs Redb)   │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ LatencyProfiler    │    │ RegressionGate    │   │   │
│   │   │ (HdrHistogram)     │    │ (Ratchet Guard)   │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Bewertungsmetriken

### 3.1 Retrieval Accuracy
- **Recall@K:** Anteil der relevanten Dokumente in den Top-$K$ Ergebnissen:
  $$\text{Recall}@K = \frac{|\text{Top}_K \cap \text{Relevant}|}{|\text{Relevant}|}$$
- **Mean Reciprocal Rank (MRR@K):** Reziproker Rang des ersten relevanten Treffers:
  $$\text{MRR} = \frac{1}{|Q|} \sum_{i=1}^{|Q|} \frac{1}{\text{rank}_i}$$
- **NDCG@K:** Normalized Discounted Cumulative Gain zur Bewertung abgestufter Relevanzen.

### 3.2 Latenzmessung
Messung über `hdrhistogram` mit Mikrosekunden-Präzision. Kaltstart- und Warmup-Phasen werden strikt getrennt protokolliert.

---

## 4. Richtlinien für Benchmark-Ergebnisse (INV-CLAIMS-1)

Jede publizierte oder im CI erfasste Benchmark-Zahl MUSS zwingend folgende Kontextdaten enthalten:
1. Exakter Git-Commit-Hash.
2. Hardware-Konfiguration: CPU-Architektur (x86_64 / aarch64), Core-Count, RAM, SSD-Modell.
3. Betriebssystem und Kernel-Version.
4. Datensatz-Name und SHA256-Prüfsumme des Korpus.
5. Konfigurationsprofil (z.B. `EnterpriseRegulated`, Thread-Pool-Größe, SIMD-Modus).

---

## 5. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig sicher.
2. **Keine Rückwirkung auf den Kern:** Benchmarks dienen ausschließlich der externen Messung; kein Kern-Crate importiert `contextra-bench`.
3. **Ratchet-Prinzip:** Baselines dürfen nur nachweislich verbessert oder bei dokumentierten Architektur-Upgrades aktualisiert werden.
