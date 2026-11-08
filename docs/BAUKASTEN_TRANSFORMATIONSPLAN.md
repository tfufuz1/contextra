# Strategischer Architektur- & Transformationsplan: Contextra Baukastensystem

> **Rolle:** Senior Rust Architekt & Principal Engineer
> **Kontext:** Transformation von einer monolithischen Produkt-Codebase in eine private, modulare Baustein-Bibliothek für Rust-LLM-, RAG- und Enterprise-Projekte.
> **Leitprinzipien:** Copy-Paste-Readiness, Privater Pragmatismus, Flache Hierarchien, Zero-Over-Engineering, Isolierte Testbarkeit.

---

## 1. Executive Summary & Architekturbefund

Contextra beherbergt ein Jahr hochentwickelter Rust-Algorithmen für LLM- und RAG-Ökosysteme:
- Extrem performante Vektordistanz-Routinen (SIMD AVX-512/NEON) und HNSW-/DiskANN-Indizes
- Skalierbare BM25- und WAND-Volltext-Suchalgorithmen mit morphologischer Stemming-Pipeline
- Modernste RRF- und DiBud-Rangfusions-Algorithmen mit Platt- und Isotonischer Kalibrierung
- Multimodale Inferenz-Treiber (Candle/GGUF, in-process ONNX, Ollama REST)
- Prefix-Radix Prompt- und KV-Cache (KIVI-Quantisierung)
- Robuste Agenten-Zustandsmaschinen, GraphRAG (CSR-Graph, APPRH-Diffusion), WASM-Sandboxing und MCP-Server

### Die Kernproblematik des bisherigen Monolithen
Trotz des Wunsches nach Modularität war die Codebase als monolithisches Gesamtsystem verdrahtet:
1. **Zentralisierte Hub-Kopplung:** 27 von 36 Crates hängen direkt an `contextra-types`, 21 an `contextra-ports`.
2. **Monolithische Sammelbecken:** `contextra-engine`, `contextra-db` und `contextra` fungierten als Nadelöhre, die 12 bis 14 Sub-Crates eng verflechten.
3. **Verstreute Compliance- und Löschketten-Logik:** Über 660 Stellen im Code waren mit DSGVO- und Deletion-Proof-Verdrahtung belastet (sogar in Vektor- und Volltext-Indizes).
4. **Schwere Portabilität:** Um einen Baustein (z. B. HNSW) in ein neues Projekt zu kopieren, mussten bisher bis zu 14 Crates mitgezogen werden.

---

## 2. Vollständige Crate-für-Crate Analyse (36 Crates)

Die 36 Crates werden nach ihrer Rolle im Baukastensystem für zukünftige LLM-Projekte strukturiert:

### Gruppe A: Autarke Basis- & System-Primitives (Sofort Copy-Paste-ready)
| Crate | LOC | Interne Deps | Baukasten-Rolle | Copy-Paste-Status & Handlungsempfehlung |
|---|---:|---:|---|---|
| `contextra-sys` | 1.183 | 0 | POSIX Low-Level Primitives (mmap, fsync, mlock, ACL) | **100 % autark.** Direkt in jedes Projekt kopierbar für sichere I/O- und Memory-Operationen. |
| `contextra-wire` | 2.038 | 0 | Zero-Copy Framing & Hardened JSON-RPC 2.0 Parser | **100 % autark.** Exzellenter Baustein für Microservices, IPC und Tool-APIs. |
| `contextra-sandbox` | 1.904 | 0 | WASM-Sandbox für isolierte Tool-/Code-Ausführung | **100 % autark.** Perfekt für sichere LLM-Code-Execution (Wasmtime-basiert). |
| `contextra-types` | 5.675 | 0 | Reine Typdefinitionen (IDs, Metadaten, Enums) | **100 % autark.** Als schlanke Typensammlung nutzbar; für maximale Entkopplung durch Standard-Rust-Typen substituierbar. |
| `contextra-simd` | 1.745 | 1 | SIMD Vektordistanzen (Kosinus, Euklid, Dot) | **Nahezu autark.** Hängt nur für `DistanceMetric` an `types`. Kann mit einem 5-Zeilen-Enum komplett isoliert werden. |

### Gruppe B: RAG-, Such- & Indexierungs-Kern (LLM Search Toolbox)
| Crate | LOC | Interne Deps | Baukasten-Rolle | Refactoring- & Entkopplungsschritte |
|---|---:|---:|---|---|
| `contextra-vector` | 11.841 | 5 | HNSW & DiskANN Vektorindex, SQ8/RaBitQ | **Priorität Hoch.** Trennung von `contextra-crypto` (bereits auf keyed BLAKE3 umgestellt) und Isolation der Lösch-Ports. Kann mit `std::io` und `contextra-simd` eigenständig laufen. |
| `contextra-text` | 5.890 | 2 | BM25, WAND-Scorer, Morphologischer Tokenizer | **Priorität Hoch.** Entkopplung von der `StorageEngine`-Port-Abhängigkeit; Umstellung auf einfache In-Memory- oder Vfs-Speicherstrukturen. |
| `contextra-rank` | 3.456 | 2 | RRF- & DiBud-Fusion, Conformal / Platt / PAVA | **Priorität Hoch.** Pure-Rust-Funktionen ohne Zwang zu `types` oder `ports`. Direkt als Re-Ranking-Crate extrahierbar. |
| `contextra-graph` | 16.812 | 2 | CSR-Graph, PPR/APPRH-Diffusion, Hyperedges | **Priorität Mittel.** GraphRAG-Baustein. Benötigt nur Vektor-/Text-IDs (`u64`/`[u8;16]`), um autark als Wissensgraph zu arbeiten. |

### Gruppe C: LLM-Inferenz & Context-Caching
| Crate | LOC | Interne Deps | Baukasten-Rolle | Refactoring- & Entkopplungsschritte |
|---|---:|---:|---|---|
| `contextra-infer-candle` | 5.015 | 4 | Lokale GGUF-/Llama- und Embedding-Inferenz | **Priorität Mittel.** Unabhängiger Candle-Client für lokale SLMs/LLMs. Bereinigung der Re-Exporte. |
| `contextra-infer-onnx` | 2.053 | 3 | In-Process ONNX-Embeddings & Reranking | **Priorität Mittel.** Feature-gated pure-Rust/ort. Direkter Baustein für schnelle CPU-Embeddings. |
| `contextra-infer-ollama` | 4.973 | 2 | REST-Client für Ollama-Inferenz | **Sehr autark.** Benötigt lediglich HTTP (`reqwest`). `ports`-Traits können optional hinter ein Feature gelegt werden. |
| `contextra-kvcache` | 4.297 | 3 | Prefix-Radix KV-Cache mit KIVI-Quantisierung | **Priorität Hoch.** Trennung von `crypto` und `ports`. Autarker Token- und KV-Cache für LLM-Pipelines. |

### Gruppe D: Agenten, Ingestion & Protokolle
| Crate | LOC | Interne Deps | Baukasten-Rolle | Refactoring- & Entkopplungsschritte |
|---|---:|---:|---|---|
| `contextra-agent` | 4.297 | 6 | ReAct Workflow-Engine, Tool-Registry, Budget | **Priorität Mittel.** Entkopplung von `contextra-db` / `contextra-store`. Akzeptiert generische Tool-Traits. |
| `contextra-router` | 5.781 | 4 | Dynamisches Routing (Bandit, Thompson-Sampling) | **Priorität Mittel.** Schlanker Entscheidungsbaustein zur Modellauswahl (SLM vs. Frontier LLM). |
| `contextra-cognition` | 5.332 | 4 | Memory Consolidation & Synthese | **Priorität Mittel.** Aggregation und Dedup von Kontextfenstern. |
| `contextra-ingest` | 16.460 | 1 | Multi-Format Dokumentenextraktion & Chunking | **Sehr autark.** Extrahiert Text aus PDF, DOCX, XLSX, HTML. Muss lediglich von `contextra-engine` entkoppelt werden. |
| `contextra-mcp` | 5.284 | 11 | Model Context Protocol Server | **Priorität Hoch.** Entkopplung von der Monolith-Fassade `contextra`; Anbindung über neutrale Schnittstellen. |
| `contextra-py` | 2.235 | 3 | PyO3 Python Bindings | Standard FFI-Schicht. |

### Gruppe E: Storage & Low-Level Infrastructure
| Crate | LOC | Interne Deps | Baukasten-Rolle | Refactoring- & Entkopplungsschritte |
|---|---:|---:|---|---|
| `contextra-durable-fs` | 978 | 3 | Crash-konsistentes atomares Vfs | **Sehr autark.** RealVfs mit atomarem Temp-Rename-Sync. Universell für robuste Datenhaltung. |
| `contextra-mvcc` | 2.909 | 2 | Sequence-Log & Snapshot-Registry | **Sehr autark.** Unabhängiges MVCC-Koordinations-Crate. |
| `contextra-store` | 23.090 | 5 | LSM-Tree Storage-Engine mit WAL | **Schwerer Baustein.** Für Projekte nutzbar, die eine eigene persistente KV-Engine benötigen. Bereinigung von Deletion-Hooks. |
| `contextra-checkpoint` | 3.584 | 2 | Snapshot- & Rollback-Koordinator | Spezieller Storage-Baustein für Time-Travel. |
| `contextra-crypto` | 5.926 | 2 | KeyManager, KDF, Verschlüsselung | Kann auf reine Krypto-Primitiven (ohne Deletion-Proofs) verschlankt werden. |

### Gruppe F: Monolithische Integratoren & Compliance-Ballast (Zurückbauen / Archivieren)
| Crate | Baukasten-Bewertung | Maßnahme für die Baukasten-Transformation |
|---|---|---|
| `contextra-engine` & `contextra-db` | Monolithische Sammelbecken mit Score 10–14 / 100 | In schlanke Integrations-Beispiele überführen oder nur noch als optionale High-Level-Fassade pflegen. |
| `contextra` & `contextra-core` | Reine Re-Export-Hüllen ohne Mehrwert | Überflüssig im Baukastensystem; auflösen. |
| `contextra-erasure`, `contextra-privacy`, `contextra-audit-export`, `contextra-avv-generator` | Spezifische DSGVO- und Deletion-Proof-Bürokratie | Vollständig aus den Kern-Bausteinen isolieren. In privaten Projekten irrelevant. |

---

## 3. Schnittstellen-Refactoring: Auflösung der Flaschenhälse

### 3.1 Das Hub-Problem: `contextra-types` und `contextra-ports`
Im Monolithen führte die Regel „Alle Typen nach `contextra-types`, alle Schnittstellen nach `contextra-ports`“ zu einer inversen Abhängigkeit: Um das kleinste Crate zu nutzen, musste das gesamte Typen-Universum importiert werden.

**Die Baukasten-Lösung:**
1. **Verwendung von Rust-Standardtypen:**
   - Vektoren: `&[f32]` oder `Vec<f32>` statt `contextra_types::Embedding`.
   - Dokument-/Node-IDs: Generischer Typparameter `K: Hash + Eq + Copy` mit Default `u64` statt `DocId`.
   - Distanzen: Einfaches lokales Enum:
     ```rust
     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
     pub enum DistanceMetric {
         Cosine,
         Euclidean,
         DotProduct,
     }
     ```
2. **Crate-lokale Fehler mit `thiserror`:**
   Jeder Baustein deklariert seinen eigenen minimalen Fehler (`RankError`, `VectorError`, `TextError`), anstatt `contextra_types::ContextraError` mit über 50 Varianten zu erzwingen.
3. **Flache Port-Definitionen:**
   Statt riesiger Traits mit Dutzenden Methoden werden schmale, fokussierte Traits direkt beim Konsumenten oder Baustein definiert (`trait Embedder { fn embed(&self, text: &str) -> Result<Vec<f32>, Error>; }`).

### 3.2 Bereinigung der Löschketten-Relikte (D-E1)
In den Kern-Crates (`contextra-vector`, `contextra-text`, `contextra-store`) werden alle Schnittstellen auf neutrale Datenstrukturen zurückgeführt:
- Keine `HnswPurger`, `TextPurger`, `WitnessLayer`, `DeletionProof` in den Core-Modulen.
- Standard-Operationen: `index.insert(id, vector)`, `index.remove(id)`, `index.search(query, k)`.

---

## 4. Konkrete Refactoring-Schritte für maximale Copy-Paste-Readiness

### Schritt 1: Autarkie der RAG-Basis (`contextra-simd` & `contextra-rank`)
- `contextra-simd`: Vollständige Loslösung von `contextra-types`. Lokales `DistanceMetric`-Enum und `SimdError`.
- `contextra-rank`: Reine mathematische Fusion (RRF, DiBud) ohne Typ-Overhead. Bereinigung ungenutzter Test-Kopplungen.

### Schritt 2: Vektor- & Volltext-Kern isolieren (`contextra-vector` & `contextra-text`)
- Bereinigung der `Cargo.toml`: Entfernung nicht-essentieller Abhängigkeiten.
- Bereitstellung autarker Standalone-Beispiele in `examples/standalone_vector.rs` und `examples/standalone_text.rs`.
- Verifikation über `python3 tools/isolation_proof.py`.

### Schritt 3: Inferenz & Caching (`contextra-infer-*` & `contextra-kvcache`)
- Bereinigung von Re-Export-Lecks.
- Klare Trennung von Candle (GGUF), ONNX und Ollama, sodass jedes Crate einzeln ohne ML-Toolchain-Konflikte gebaut werden kann.

### Schritt 4: Agenten, Ingestion & MCP
- `contextra-agent`: Entkopplung von der Datenbank. Der Agent interagiert nur noch mit In-Memory-Context oder generischen Storage-Traits.
- `contextra-ingest`: Entfernung des `chunker`-Imports aus `contextra-engine`; Bereitstellung als autarkes Parsing-Crate.
- `contextra-mcp`: Direkte Anbindung an Vektor- und Text-Indizes ohne `contextra-db`-Monolith.

---

## 5. Das Copy-Paste-Kochbuch (How-To für neue Projekte)

Wenn in einem zukünftigen Projekt eine bestimmte Fähigkeit benötigt wird:

1. **Schnelle Vektorsuche & Distanzen:**
   - Kopiere `crates/contextra-simd` und `crates/contextra-vector/src/hnsw/`
   - Abhängigkeiten: nur `rand`, `parking_lot`, `rayon` (optional).
   - Ergebnis: In 2 Minuten ein hochperformanter In-Memory HNSW Vektorindex.

2. **Hybride Suche (Vektor + BM25 Fusion):**
   - Kopiere `crates/contextra-vector`, `crates/contextra-text/src/bm25/` und `crates/contextra-rank/src/fusion/`
   - Ergebnis: Vollwertige RRF-Hybrid-RAG-Pipeline ohne Datenbank-Server.

3. **Multi-Format Dokumenten-Parser:**
   - Kopiere `crates/contextra-ingest`
   - Ergebnis: Extrahiert Text aus PDF, Office-Dateien und Webseiten für LLM-Prompts.

4. **Sichere Tool-Ausführung:**
   - Kopiere `crates/contextra-sandbox`
   - Ergebnis: Sichere WASM-Sandbox zur Ausführung von durch LLMs generiertem Code.

5. **Lokale Inferenz:**
   - Kopiere `crates/contextra-infer-onnx` (für CPU/GPU-Embeddings) oder `crates/contextra-infer-candle` (für GGUF-Modelle).

---

## 6. Zusammenfassung & Nächste Schritte

Mit diesem Plan wandelt sich Contextra vom unübersichtlichen Monolithen zum **persönlichen High-End-Werkzeugkasten**. Jeder Baustein beweist seine Funktionsfähigkeit isoliert über Unit-Tests und Standalone-Beispiele. Das System wird damit wartungsarm, pragmatisch und sofort einsatzbereit für jede neue Enterprise- und LLM-Aufgabe.
