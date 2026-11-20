# contextra-bench — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E3 (Endpunkte / Benchmarks)  
**Kern-Closure:** `db`, `store`, `vector`, `text`, `rank`, `types`  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `benchmarks/contextra-bench`  
**Referenzdokumente:** `docs/SPEC.md` §2, §12; `ALTLASTEN-AUDIT.md` (AP-14)  

---

## 1. Zweck & Architekturgarantien

`contextra-bench` ist die offizielle Benchmark-, Evaluations- und Regressions-Harness für Contextra:
- **Retrieval Accuracy:** Wissenschaftlich fundierte Messung von Recall@K, MRR@K, NDCG@K über standardisierte Testkorpora (BEIR-Standard).
- **Latenz- & Durchsatz-Profiling:** Mikrosekunden-genaue Aufzeichnung von Latenz-Perzentilen (p50, p95, p99, p99.9) via `hdrhistogram`.
- **Regressions-Gates (Ratchet Guard):** CI-Überwachung zur Verhinderung von Performance- oder Genauigkeitsrückschritten gegenüber der gespeicherten Baseline (`baseline_metrics.json`).
- **Wettbewerbsvergleiche:** Standardisierte Benchmarks gegen externe Speicher-Engines (z. B. Redb, Sled).

### Beseitigung von Altlasten (AP-14)
1. **AP-14a (`src/main.rs`):** Entfernen der unkompilierbaren ONNX-Importe (`#[cfg(feature = "onnx-bench")] use contextra_infer_onnx::...`).
2. **AP-14b (`src/main.rs`):** Entfernen der Candle-Embedding-Instantiierung (`contextra_infer_candle::CandleEmbedClient`).
3. **Leere Default-Features (R3):** `default = []`.

### Architekturgarantien (R1–R6, INV-3, INV-4, INV-7)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Reine Auswertungsebene:** Keine Rückwirkung auf Kern-Bibliotheken (E0 bis E2).
3. **Keine ungeprüften Behauptungen (INV-7):** Jedes Benchmark-Ergebnis erfasst Git-Commit, CPU-Architektur, RAM, SSD und Datensatz-Prüfsumme.

---

## 2. Bewertungsmetriken & Formeln

### 2.1 Retrieval-Genauigkeit
- **Recall@K:**
  $$\text{Recall}@K = \frac{|\text{Top}_K \cap \text{Relevant}|}{|\text{Relevant}|}$$
- **Mean Reciprocal Rank (MRR@K):**
  $$\text{MRR} = \frac{1}{|Q|} \sum_{i=1}^{|Q|} \frac{1}{\text{rank}_i}$$
- **NDCG@K:** Normalized Discounted Cumulative Gain zur Bewertung differenzierter Relevanzstufen.

### 2.2 Latenz- und Durchsatz-Metriken
- Durchsatz: Operationen pro Sekunde (QPS).
- Latenz-Perzentile: $p50$, $p90$, $p99$, $p99.9$ unter konstanter und variierender Thread-Last.

---

## 3. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-bench"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
default-run = "contextra-bench"
description = "Reproducible benchmark harness and regression gates for Contextra"

[lints]
workspace = true

[package.metadata.contextra]
ring = "tooling"
level = 3

[dependencies]
contextra-types = { workspace = true }
contextra-db = { workspace = true }
contextra-store = { workspace = true }
contextra-vector = { workspace = true }
contextra-text = { workspace = true }
contextra-rank = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tempfile = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
hdrhistogram = "7"
rand = { workspace = true }
anyhow = "1"

[dev-dependencies]
criterion = { workspace = true }
redb = "2"

[features]
default = []
datasets = []
external-benchmarks = []
```

---

## 4. Implementierungs-Arbeitspakete für Entwickler

1. **AP-14a & AP-14b (`src/main.rs`):**
   - Veraltete Bezüge auf `contextra_infer_onnx` und `contextra_infer_candle` bereinigen.
2. **Baselines auditieren:**
   - Sicherstellen, dass Regression-Gate-Schwellenwerte mit den konsolidierten 23 Crates harmonieren.
3. **Verifikation:**
   - `cargo check -p contextra-bench`
