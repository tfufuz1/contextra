# <p align="center">CONTEXTRA</p>
<p align="center"><b>Local Knowledge & Memory Engine in Rust</b></p>

<!-- Das Hauptbanner ganz oben -->
<p align="center">
  <img src=".github/assets/banner.jpg" alt="Contextra GitHub Banner" width="100%">
</p>

---

## 🏗️ Architektur & Domänen

Contextra ist modular aufgebaut. Die folgenden Grafiken visualisieren die Kernkomponenten der einzelnen Crates:

### 1. Hybrid Retrieval & Suche
<p align="center">
  <img src=".github/assets/hybrid-retrieval.jpg" alt="Hybrid Retrieval & Suche" width="80%">
</p>
Kombiniert semantische Vektorsuche (HNSW, SQ8, SIMD) mit klassischer Volltextsuche (BM25) und Rangfusion (RRF).

---

### 2. Autonome Agenten & Kognition (`agentic`)
<p align="center">
  <img src=".github/assets/agentic.jpg" alt="Agentic & Cognition" width="80%">
</p>
Steuert die Execution-Engine für mehrstufige Agentenprozesse und das langfristige Langzeitgedächtnis.

---

### 3. Adaptives Routing & Modellsteuerung (`llm`)
<p align="center">
  <img src=".github/assets/routing.jpg" alt="Adaptive Routing" width="80%">
</p>

![Beschreibung des Bildes](.github/assets/routing.jpg)

Nutzt Contextual Bandits und PID-Regler zur dynamischen Latenz- und Modellsteuerung.

---

### 4. Datenschutz, Sicherheit & Compliance
<p align="center">
  <img src=".github/assets/security.jpg" alt="Safety & Compliance" width="80%">
</p>
Gewährleistet Egress-Schutz, Prompt-Injection-Audits und kryptografisches Shredding (DeletionProof V5).

---

### 5. Speichersystem, Transaktionen & Isolation
<p align="center">
  <img src=".github/assets/storage.jpg" alt="Storage & Foundation" width="80%">
</p>
Basiert auf einem LSM-Tree-Kern mit MVCC-Snapshot-Isolation und Crash-konsistentem WAL.

---

### 6. Ingest, Inferenz & Anwendungen
<p align="center">
  <img src=".github/assets/ingest.jpg" alt="Ingest & Inference" width="80%">
</p>
Übernimmt strukturiertes Chunking, On-Device-Inferenz (Candle/ONNX) und die MCP-Schnittstelle.
