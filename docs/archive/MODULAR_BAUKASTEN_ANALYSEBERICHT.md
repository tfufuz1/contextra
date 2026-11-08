> Nicht normativ, widerlegt durch SPEC v4.0 §1

# Umfassender Analysebericht: Transformation in eine modulare Baustein-Bibliothek

**Dokument-ID:** ARCH-ANALYSE-2026-11
**Rolle:** Principal Rust Engineer & Systemarchitekt
**Status:** Abgeschlossen
**Geltungsbereich:** Gesamter Workspace (36 Crates)

---

## 1. Einleitung & Kontext

Die vorliegende Codebase wurde ursprünglich über rund ein Jahr hinweg als vertikal integriertes, monolithisches Ökosystem für LLM- und RAG-Anwendungen konzipiert. In dieser Phase standen Funktionsvielfalt und das Zusammenspiel aller Subsysteme (Speicher, Volltext, Vektoren, Graphen, Multi-Agenten, Inferenz, Caching, DSGVO-Compliance) im Vordergrund.

Das Ziel hat sich grundlegend gewandelt: **Das Repository wird zu einer privaten, internen Baustein-Bibliothek für Enterprise- und Robotik-Projekte umgebaut.**
Einzelne Bausteine (z. B. SIMD-Vektordistanzen, In-Memory-HNSW, RRF-Rankfusion, Prefix-KV-Cache, WASM-Sandbox, Wire-Protokolle) müssen ohne Ballast, ohne Kenntnis anderer Module und mit minimalen Abhängigkeiten in Drittprojekte kopiert oder als schlanke Crates eingebunden werden können.

---

## 2. Ist-Zustand & Kopplungsanalyse (36 Crates)

### 2.1 Kopplungsmetriken & Modularitäts-Scores
Mithilfe des systemweiten Inspektionswerkzeugs `tools/decouple_inspector.py` wurde der Modularitäts- und Autarkiegrad aller 36 Crates quantifiziert (Skala 0–100, wobei 100 für vollständige Autarkie steht):

| Crate | Modularitäts-Score | LOC | Interne Deps | Status & Baukasten-Klassifizierung |
|---|---:|---:|---:|---|
| `contextra-sys` | **100/100** | 1.183 | 0 | **Autarker Basisbaustein** (POSIX mlock, fsync, ACL, direct I/O) |
| `contextra-wire` | **100/100** | 2.038 | 0 | **Autarker Basisbaustein** (Zero-Copy Framing, FlatBuffers, RPC) |
| `contextra-sandbox` | **100/100** | 1.904 | 0 | **Autarker Basisbaustein** (WASM/WASI Tool-Execution via Wasmtime) |
| `contextra-simd` | **94/100** | 1.745 | 1 | **Autarker Basisbaustein** (AVX-512/NEON Vektordistanzen) |
| `contextra-types` | **90/100** | 5.675 | 0 | **Geteilter Basistyp-Pool** (Minimalisierbar auf Standardtypen) |
| `contextra-rank` | **88/100** | 3.456 | 2 | **RAG-Baustein** (RRF, DiBud Fusion, Conformal & Platt Scaling) |
| `contextra-license` | **88/100** | 1.250 | 2 | **Enterprise-Baustein** (Ed25519 Feature-Ring Gates) |
| `contextra-privacy` | **88/100** | 3.695 | 2 | **Enterprise-Baustein** (Rule-Based DLP, PII Detection) |
| `contextra-checkpoint`| **88/100** | 3.584 | 2 | **Storage-Baustein** (Snapshots & Time-Travel) |
| `contextra-adapt` | **88/100** | 2.110 | 2 | **RAG-Baustein** (Adaptive Decay Controller, Kalibrierung) |
| `contextra-ingest` | **84/100** | 4.820 | 1 | **LLM-Baustein** (Multi-Format Document Chunking & Parsing) |
| `contextra-mvcc` | **83/100** | 2.909 | 2 | **Storage-Baustein** (Lock-Free Sequence Log & Snapshot Registry) |
| `contextra-testkit` | **83/100** | 2.611 | 2 | **Test-Tooling** (Fault-Injection Vfs, Conformance Mocks) |
| `contextra-infer-ollama`| **83/100** | 1.450 | 2 | **Inferenz-Baustein** (Ollama REST Client) |
| `contextra-infer-onnx` | **82/100** | 2.310 | 2 | **Inferenz-Baustein** (In-Process ONNX Runtime Embeddings) |
| `contextra-erasure` | **77/100** | 6.840 | 3 | **Spezial-Baustein** (Kryptographisches Shredding & Sagas) |
| `contextra-db` | **77/100** | 4.120 | 3 | **Fassaden-Baustein** (Multi-Collection Facade) |
| `contextra-py` | **77/100** | 2.235 | 3 | **Binding-Baustein** (PyO3 Python-Bridge) |
| `contextra-cognition` | **76/100** | 5.210 | 4 | **Agentik-Baustein** (Memory Consolidation & Compaction) |
| `contextra-durable-fs`| **75/100** | 978 | 3 | **Storage-Baustein** (Crash-Safe Atomic File Vfs) |
| `contextra-infer-candle`| **71/100**| 3.120 | 3 | **Inferenz-Baustein** (Candle GGUF/Quantized Llama/Qwen) |
| `contextra-core` | **66/100** | 1.840 | 4 | **Legacy Strangler** (Veralteter Integrator, Phase-Out) |
| `contextra-audit-export`| **64/100**| 1.150 | 3 | **Enterprise-Baustein** (Compliance Audit Exporter) |
| `contextra-avv-generator`| **64/100**| 980 | 3 | **Enterprise-Baustein** (Auftragsverarbeitungs-Vertrags-Gen) |
| `contextra-router` | **61/100** | 5.781 | 4 | **Network-Baustein** (Query Routing & Semantic Dispatch) |
| `contextra-crypto` | **58/100** | 5.926 | 2 | **Krypto-Baustein** (AEAD, KDF, Anti-Tamper) |
| `contextra-kvcache` | **57/100** | 4.230 | 4 | **LLM-Baustein** (Prefix-Radix Prompt- & KV-Cache) |
| `contextra-agent` | **54/100** | 3.840 | 5 | **Agentik-Baustein** (Tool-Calling, ReAct Planner) |
| `contextra-mcp` | **53/100** | 4.650 | 5 | **Protokoll-Baustein** (Model Context Protocol Server) |
| `contextra-text` | **52/100** | 5.890 | 2 | **RAG-Baustein** (BM25, WAND Scorer, Tokenizer) |
| `contextra-ports` | **50/100** | 4.890 | 1 | **Zentraler Port-Hub** (Muss durch lokale Traits ersetzt werden) |
| `contextra-graph` | **48/100** | 16.812 | 2 | **GraphRAG-Baustein** (CSR Graph, APPRH Diffusion) |
| `contextra` | **39/100** | 1.233 | 6 | **Monolith-Fassade** (Hauptbibliothek, historischer Root) |
| `contextra-store` | **35/100** | 23.090 | 5 | **Schwerer Storage-Baustein** (LSM-Tree KV-Engine mit WAL) |
| `contextra-vector` | **25/100** | 11.841 | 5 | **RAG-Baustein** (HNSW, DiskANN, SQ8/RaBitQ Quantisierung) |
| `contextra-engine` | **10/100** | 18.450 | 6 | **Monolithischer Orchestrator** (Aggregiert alle Module eng) |

---

## 3. Identifizierte Hauptprobleme (mit Priorität)

### Priorität 1 (Kritisch): Die „Hub-and-Spoke“-Falle (`types` und `ports`)
- **Befund:** Fast alle Crates binden `contextra-types` und `contextra-ports` ein. Das führte dazu, dass selbst ein rein mathematischer Baustein wie `contextra-simd` ein Crate mit über 5.600 Zeilen Typdefinitionen importiert hat, nur um ein einfaches 3-Varianten-Enum (`DistanceMetric`) zu nutzen.
- **Risiko bei Übernahme:** Will ein Entwickler `contextra-simd` oder `contextra-rank` in eine Robotik-Anwendung übernehmen, muss er Cargo-Abhängigkeiten auf das gesamte Contextra-Typsystem mitschleppen.
- **Lösung:** Ersetzung externer Typen durch Rust-Standardtypen (`&[f32]`, `u64`, `&str`) und crate-lokale Enums.

### Priorität 2 (Hoch): Verflechtung von Fachlogik und Löschketten/DSGVO
- **Befund:** Im Vektorindex (`contextra-vector`), Volltextindex (`contextra-text`) und LSM-Store (`contextra-store`) finden sich Relikte von Löschketten, `WitnessLayer`, `DeletionProof` und DSGVO-Referenzen.
- **Risiko bei Übernahme:** In Robotik- und Enterprise-Projekten ohne strikte DSGVO-Art.-17-Bedarfe stört dieser Ballast die Lesbarkeit, Performance und Testbarkeit.
- **Lösung:** Sämtliche Löschungs- und Nachweislogik ist strikt in `contextra-erasure` gekapselt. Indizes bieten rein neutrale Operationen (`insert`, `search`, `remove`).

### Priorität 3 (Mittel): Monolithische Integratoren (`engine`, `db`, `contextra`)
- **Befund:** `contextra-engine` (18k LOC) und `contextra-db` (4k LOC) fungierten als monolithische Nadelöhre. Sie zwingen Vektor, Text, Graph, MVCC und Store in ein starres Korsett.
- **Risiko bei Übernahme:** Unmöglichkeit, z. B. nur Textsuche + RRF ohne LSM-Storage zu betreiben.
- **Lösung:** Dependency Inversion via generische Backends (`trait Backends`) und schmale Adapter.

---

## 4. Risiken & Mitigationen

1. **Breaking Changes bei bestehenden Schnittstellen:**
   - *Mitigation:* Re-Exporte in Übergangs-Crates (`contextra`, `contextra-db`) bleiben erhalten; neue Baukasten-APIs werden schrittweise eingeführt, ohne bestehende Tests zu brechen.
2. **Performance-Regressionen durch Entkopplung:**
   - *Mitigation:* Verwendung von statischem Dispatch (Generics/Monomorphisierung) statt dynamischem Dispatch (`dyn Trait`), wo Performance kritisch ist (SIMD, HNSW, WAND).
3. **Inkonsistente Fehlerbehandlung:**
   - *Mitigation:* Jeder Baustein erhält seinen eigenen, typisierten Fehler (`thiserror`), der `std::error::Error` implementiert. Keine globalen Monster-Enums.
