# Zielarchitektur & Modul-Katalog: Contextra Baukastensystem

**Dokument-ID:** ARCH-ZIEL-2026-11
**Rolle:** Principal Rust Engineer & Systemarchitekt
**Geltungsbereich:** Interne Baustein-Bibliothek für LLM-, Enterprise- und Robotik-Systeme

---

## 1. Architektur-Leitbild: Die Private Baustein-Bibliothek

Das Gesamtrepository wird von einem monolithisch verdrahteten Ökosystem in eine **Sammlung unabhängiger, modularer Bausteine (Lego-Prinzip)** überführt. Jeder Baustein ist:
- **Autark kompilierbar:** Kann ohne Kenntnis anderer Bausteine gebaut und getestet werden.
- **Abhängigkeitsarm:** Nutzt bevorzugt Rust-Standardtypen (`&[f32]`, `u64`, `&str`) anstelle globaler Crate-Abhängigkeiten.
- **Eindeutig fehlertypisiert:** Definiert eigene schlanke Fehlertypen mit `thiserror`.
- **Copy-Paste-Readiness:** Kann durch einfaches Kopieren der Quelldateien oder Referenzieren des Crates in Enterprise- und Robotik-Projekte übernommen werden.
- **Feature-Gated:** Schwere Abhängigkeiten (PyO3, ONNX, Wasmtime, Ingestion-Parser) liegen hinter präzisen Feature-Flags.

---

## 2. Modul-Katalog & Anwendungsdomänen

Die 36 Crates werden in 5 klare Fachdomänen strukturiert:

### Domäne 1: Low-Level Primitives & System-Bausteine (System & Robotics)
*Einsatz: Robuste Echtzeit-I/O, IPC, Speichersicherheit, Embedded & Robotik.*
- **`contextra-sys` (Score 100/100):** POSIX Low-Level Primitives (`mlock`, `fsync`, `fdatasync`, sichere ACLs, kein unkontrollierter Core-Dump).
- **`contextra-simd` (Score 94/100 -> 100/100):** Hardware-beschleunigte Vektordistanzen (AVX-512 VNNI, ARM NEON, Cosine, L2, Dot).
- **`contextra-wire` (Score 100/100):** Gehärtetes Zero-Copy-Framing und JSON-RPC 2.0 Parser mit Tiefenbomben-Schutz für Microservices und Robotik-Busse.
- **`contextra-durable-fs` (Score 75/100):** Crash-sicheres Vfs mit atomaren Renames und Fsync-Poisoning-Schutz.

### Domäne 2: LLM Search, RAG & Ranking (Search Toolbox)
*Einsatz: RAG-Pipelines, hybride Volltext-/Vektorsuche, Agent-Memory.*
- **`contextra-rank` (Score 88/100):** RRF (Reciprocal Rank Fusion), DiBud-Streaming-Fusion, Platt- und Isotonische Kalibrierung, Conformal Prediction.
- **`contextra-vector` (Score 25/100 -> entkoppelt):** In-Memory & DiskANN HNSW-Index mit SQ8- und RaBitQ-Quantisierung.
- **`contextra-text` (Score 52/100 -> entkoppelt):** BM25- und WAND-Volltext-Scorer mit morphologischem Tokenizer.
- **`contextra-graph` (Score 48/100):** Hochperformanter CSR-Graph (Compressed Sparse Row) für GraphRAG und APPRH-Diffusion.

### Domäne 3: LLM Inference, Caching & Sandboxing (Agent Tooling)
*Einsatz: Lokale Modell-Ausführung, Prompt-Caching und sichere Tool-Ausführung.*
- **`contextra-sandbox` (Score 100/100):** Wasmtime-basierte WASM/WASI-Sandbox zur sicheren Ausführung von durch LLMs generiertem Code.
- **`contextra-kvcache` (Score 57/100):** Prefix-Radix-Tree Prompt-Cache und KIVI-quantisierter KV-Cache für schnelle LLM-Antwortzeiten.
- **`contextra-infer-candle` (Score 71/100):** In-Process GGUF/Quantized Llama & Qwen Inferenz ohne Python.
- **`contextra-infer-onnx` (Score 82/100):** In-Process ONNX Runtime für Text-Embeddings.
- **`contextra-infer-ollama` (Score 83/100):** Asynchroner HTTP-Client für lokale Ollama-Instanzen.
- **`contextra-ingest` (Score 84/100):** Universeller Dokumenten-Parser (PDF, DOCX, XLSX, HTML, Markdown, JSONL, E-Mail).

### Domäne 4: Storage & Concurrency (Persistence Engine)
*Einsatz: Transaktionale Speicherung, Snapshot-Isolation, ACID KV-Store.*
- **`contextra-mvcc` (Score 83/100):** Lock-freies Sequenzlog und Snapshot-Registry mit Serializable Snapshot Isolation (SSI).
- **`contextra-store` (Score 35/100):** Persistenter LSM-Tree mit Write-Ahead-Log (WAL) und Group-Commit.
- **`contextra-checkpoint` (Score 88/100):** Snapshot-Koordination und Point-in-Time Recovery.
- **`contextra-crypto` (Score 58/100):** AEAD (AES-256-GCM-SIV), Argon2id/HKDF-KDF und WAL-HMAC-Chaining.

### Domäne 5: Enterprise Governance & Spezialschnittstellen
*Einsatz: Compliance, DSGVO-Art.-17-Löschung, MCP-Protokoll.*
- **`contextra-erasure` (Score 77/100):** Unabhängige Saga-Zustandsmaschine für kryptographisches Shredding.
- **`contextra-privacy` (Score 88/100):** Regelbasierte PII-/DLP-Erkennung (IBAN, E-Mail, Kreditkarte, Steuer-ID).
- **`contextra-license` (Score 88/100):** Ed25519-signierte Feature-Ring-Gates.
- **`contextra-mcp` (Score 53/100):** Model Context Protocol Server (Stdio-basiert) für LLM-Agenten.
- **`contextra-py` (Score 77/100):** Optionale PyO3 Python-Bindings.

---

## 3. Ziel-Abhängigkeitsgraph & Entkopplungsprinzipien

### 3.1 Auflösung der inversen Abhängigkeit
```
[Vorher: Monolithische Hub-Kopplung]
Alle 36 Crates ──► contextra-types (5.6k LOC)
               ──► contextra-ports (4.8k LOC)
               ──► contextra-engine (18k LOC)

[Nachher: Horizontale, entkoppelte Bausteine]
contextra-simd    ──► (Standard Rust: &[f32], usize)
contextra-rank    ──► (Standard Rust: &[T], f32 / Scores)
contextra-sandbox ──► (Wasmtime, autark)
contextra-wire    ──► (FlatBuffers / Serde, autark)
contextra-sys     ──► (libc, autark)
contextra-kvcache ──► (Generischer Trie & Cache)
```

### 3.2 Feature-Matrix für schlanke Builds

| Crate | Default-Features | Optionale Features | Zweck |
|---|---|---|---|
| `contextra-simd` | `std` | `avx512`, `neon`, `types-compat` | Erlaubt reine Standard-Nutzung ohne `contextra-types` |
| `contextra-rank` | `std` | `conformal`, `types-compat` | Trennt RRF-Kern von Port-Abstraktionen |
| `contextra-kvcache` | `std` | `kivi-quant`, `persistence` | Leichtgewichtiger Prompt-Cache ohne LSM |
| `contextra-sandbox` | `std` | `wasi-http`, `async-runtime` | Wasm-Sandbox bedarfsgerecht konfigurierbar |
| `contextra-py` | `[]` | `pyo3`, `numpy`, `pythonize` | Zero Overhead bei reiner Rust-Nutzung |

---

## 4. Begründung der Architekturentscheidungen (Trade-Offs)

1. **Beibehaltung der Crates als separate Verzeichnisse statt Komplett-Löschung:**
   - *Vorteil:* Erhalt des über ein Jahr entwickelten Algorithmen-Schatzes; jedes Crate behält seine Git-Historie.
   - *Entscheidung:* Keine Zerstörung von Funktionalität, sondern schrittweise Isolation und Entkopplung.
2. **Crate-lokale Typen vs. Shared-Type-Library:**
   - *Vorteil:* Sofortige Copy-Paste-Fähigkeit in Fremdprojekte ohne Abhängigkeitskaskade.
   - *Nachteil:* Geringfügige Duplikation trivialer Enums (`DistanceMetric`).
   - *Entscheidung:* Die Portabilität überwiegt geringe Duplikationen bei Weitem.
