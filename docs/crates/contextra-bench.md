# contextra-bench — Crate-Spezifikation

**Ebene:** E3 (Endpunkte / Benchmarks)  
**Bindung:** `docs/SPEC.md` §2, §12

---

## 1 Zweck & Domäne

`contextra-bench` (unter `benchmarks/contextra-bench`) ist die reproduzierbare Benchmark-Harness des Projekts:
- Messung von Recall@K, MRR@K und NDCG@K über BEIR- und synthetische Korpora
- Latenz- und Durchsatzmessung für HNSW-, Volltext- und Hybrid-Suchen
- Regressions-Gate zur Erkennung von Performance- und Genauigkeitsverlusten

## 2 Unabhängigkeit & Neutralität

- `contextra-bench` wird von keinem Kern-Crate als Abhängigkeit eingebunden.
- Enthält deterministische Korpora mit festen Ground-Truth-Referenzen.
- Jede Benchmark-Publikation dokumentiert zwingend Commit-Hash, Plattform, Hardware und Modellkonfiguration.
- `#![forbid(unsafe_code)]`.
