> Nicht normativ, widerlegt durch SPEC v4.0 §1

# Architektur & Transformationsplan: Contextra als modulare Baustein-Bibliothek für Rust-LLM-Lösungen

> **Status:** Strategisches Architektur- und Migrationskonzept
> **Ziel:** Transformation des ehemals monolithischen Contextra-Ökosystems in eine unkomplizierte, hochgradig entkoppelte Baustein-Bibliothek für moderne Rust-LLM-Werkzeuge und -Produkte.

---

## 1. Executive Summary & Strategische Neuausrichtung

### 1.1 Ausgangslage & Problemstellung
Contextra wurde über ein Jahr lang intensiv als umfassendes, vertikal integriertes Produkt-Ökosystem entwickelt. Im Zuge dieser Entwicklung entstanden wertvolle, performante und praxiserprobte Algorithmen für den Umgang mit Large Language Models (LLMs) – darunter Vektorindizes, hybride BM25-Suche, Re-Ranking, Inferenz-Anbindungen, Prompt-/KV-Caching, Dokumentenextraktion und Agenten-Workflows.

Gleichzeitig führte der ursprüngliche Produktfokus jedoch zu einer **extremen internen Kopplung (Monolith)**:
- **Künstliche Zersplitterung:** 36 einzelne Crates mit unübersichtlichen Querabhängigkeiten.
- **Monolithische Altlasten:** Mehr als 660 Fundstellen von DSGVO-Löschketten, Deletion-Proofs und Audit-Trails waren quer durch unbeteiligte Kernkomponenten (wie Vektor- oder Volltext-Indizes) verteilt.
- **Schwere Entnehmbarkeit:** Wollte ein Entwickler lediglich einen performanten HNSW-Index oder einen BM25-Tokenizer in ein neues Projekt kopieren, mussten bis zu 14 Workspace-Crates, Schemadateien und Trait-Hüllen mitkopiert werden.

### 1.2 Die neue Vision: Das Rust-LLM Baukastensystem
Aus diesem Repository wird **kein monolithisches Endprodukt** mehr gebaut. Stattdessen dient Contextra als **fundierte Baustein-Bibliothek (Building Blocks / Lego-System)**.
Entwickler können einzelne Crates oder Modul-Dateien direkt entnehmen, in eigene Projekte kopieren, Pfade/Namen umbenennen und sofort lauffähige Rust-LLM-Werkzeuge erstellen (z. B. eigene RAG-Pipelines, lokale Inferenz-Tools, MCP-Server oder Agenten-Frameworks).

---

## 2. Strukturierung nach LLM-Anwendungsdomänen

Statt der historischen Einteilung nach Rings (Ring 0 bis 4) gruppiert die neue Architektur alle Funktionen nach **konkreten Anwendungsfeldern in der LLM-Softwareentwicklung**:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                             LLM APPLICATION DOMAINS                              │
├─────────────────────┬──────────────────────┬─────────────────────────────────────┤
│ 1. RAG & Suche      │ 2. LLM-Inferenz      │ 3. Agenten & Kognition              │
│    - HNSW Vektor    │    - Candle (GGUF)   │    - Workflow-Engine                │
│    - BM25 Volltext  │    - ONNX Embed/Rank │    - Tool Router & Dispatch         │
│    - RRF Fusion     │    - Ollama API      │    - Memory Consolidation           │
│    - SIMD Distanzen │                      │    - WASM-Sandbox                   │
├─────────────────────┼──────────────────────┼─────────────────────────────────────┤
│ 4. Cache & Context  │ 5. GraphRAG          │ 6. Ingestion & ETL                  │
│    - Prefix-Radix   │    - CSR Graph       │    - Multi-Format Parser            │
│    - KV-State Cache │    - Entity Linking  │    - Chunking & Splitting           │
│    - Token-Pruning  │    - Path-Traversal  │    - Metadata Extraktion            │
├─────────────────────┴──────────────────────┴─────────────────────────────────────┤
│ 7. Protokolle & Interop                    │ 8. Durable Storage (Optional)       │
│    - Model Context Protocol (MCP)          │    - LSM-Tree Engine & WAL          │
│    - Hardened JSON-RPC 2.0                 │    - POSIX Fsync Durability         │
│    - PyO3 Python Bindings                  │    - MVCC Point-in-Time             │
└────────────────────────────────────────────┴─────────────────────────────────────┘
```

---

## 3. Crate-by-Crate Analyse & Transformationsplan

Nachfolgend werden alle 36 Crates des Repositories hinsichtlich **Reife**, **LLM-Relevanz**, **konkreten Funktionen**, **Entkopplungsmaßnahmen** und ihrer **Zukunft im Baukastensystem** auditiert:

| Crate | Reife | LLM-Relevanz | Kernfunktionen | Entkopplungs-Befund & Maßnahme | Zukunftsempfehlung |
|---|:---:|:---:|---|---|---|
| `contextra-simd` | 94/100 | Hoch | AVX2/AVX-512/NEON Distanzfunktionen (Cosine, Euclidean, Dot Product) | Autarkes Ring-0 Crate, fast keine internen Abhängigkeiten. | **Beibehalten** (autarker Low-Level Baustein) |
| `contextra-vector` | 19/100 | Sehr hoch | HNSW Vektorindex, Graph-Repair, dynamische Heuristiken | Stark verdrahtet mit Krypto- & Deletion-Relikten. Bereinigen! | **Fusionieren** in `rust-llm-index` |
| `contextra-text` | 52/100 | Sehr hoch | BM25 Inverted Index, Unicode Tokenizer, Stemming | Enthält Löschketten-Relikte. Reine Such- & Tokenizer-Logik isolieren. | **Fusionieren** in `rust-llm-index` |
| `contextra-rank` | 88/100 | Sehr hoch | Reciprocal Rank Fusion (RRF), Platt Scaling, Conformal Prediction | Sehr reif und sauber. Abhängigkeiten minimal halten. | **Fusionieren** in `rust-llm-index` |
| `contextra-infer-candle` | 71/100 | Sehr hoch | Lokale GGUF Inferenz, Candle-Backend, Model Registry | Eigenständige Inferenz-Logik; benötigt Standard-Schnittstelle. | **Fusionieren** in `rust-llm-infer` |
| `contextra-infer-onnx` | 82/100 | Sehr hoch | Text-Embedding & Cross-Encoder Re-Ranking via ONNX | Sauberes Crate, optional via Feature-Flag. | **Fusionieren** in `rust-llm-infer` |
| `contextra-infer-ollama` | 83/100 | Sehr hoch | Async Client für Ollama REST API, Query-Rewriter | Leichtgewichtig, benötigt keine schweren Workspace-Crates. | **Fusionieren** in `rust-llm-infer` |
| `contextra-kvcache` | 57/100 | Sehr hoch | Prefix-Radix Tree für Prompt Caching, Token Re-Use | Hoher Mehrwert für LLM Latenz- & Kostenreduktion. Krypto-Teile optional machen. | **Beibehalten** als autarker Cache-Baustein |
| `contextra-graph` | 48/100 | Hoch | CSR-Graph (Compressed Sparse Row), GraphRAG, Entity Traversals | Mächtige Graph-Strukturen, aktuell an interne Ports gekoppelt. Ports generifizieren. | **Beibehalten** als autarker GraphRAG-Baustein |
| `contextra-ingest` | 84/100 | Sehr hoch | Format-Parser: PDF, DOCX, XLSX, HTML, CSV, Markdown, XML | Extrem wertvoll für RAG Datenpipelines. Weitgehend autark. | **Beibehalten** als autarker Parser-Baustein |
| `contextra-agent` | 54/100 | Hoch | State-Machine Workflows, Checkpoint-Rollbacks, ReAct Loop | Relikte alter DB-Kopplung entfernen. Autarke Workflow-Engine bauen. | **Fusionieren** in `rust-llm-agent` |
| `contextra-cognition` | 56/100 | Hoch | Memory Consolidation, Kurzzeit-/Langzeitgedächtnis, Dekay | Eng an `contextra-engine` gekoppelt. Logik in Agenten-Modul überführen. | **Fusionieren** in `rust-llm-agent` |
| `contextra-router` | 61/100 | Hoch | Intent-Routing, Model Cascades, Semantischer Dispatcher | Gute Heuristiken für Multi-Model Setups. | **Fusionieren** in `rust-llm-agent` |
| `contextra-sandbox` | 100/100 | Hoch | WASM Runtime Boundary für sichere Tool-Ausführung | Perfekt autark (100/100 Score). Keine internen Workspace-Abhängigkeiten. | **Beibehalten / Beistellen** in Tool-Execution |
| `contextra-adapt` | 88/100 | Mittel | Adaptive Controller, Bandits, dynamische Batching-Parameter | Nützliche Heuristiken zur Laufzeitoptimierung. | **Zusammenlegen** mit Inferenz oder Agent |
| `contextra-mcp` | 53/100 | Sehr hoch | Model Context Protocol Server & Client (Tools, Resources, Prompts) | Industriestandard (Anthropic MCP). Sehr stark mit Monolith verwebt. Entflechten! | **Beibehalten / Bereinigen** als `rust-llm-mcp` |
| `contextra-wire` | 100/100 | Hoch | FlatBuffers Zero-Copy Framing & Hardened JSON-RPC 2.0 Parser | Autark und performant. Dient als Fundament für RPC und MCP. | **Verschmelzen** mit MCP oder als Proto-Lib nutzen |
| `contextra-py` | 77/100 | Mittel | PyO3 Python Bindings für Vektor- & Suchoperationen | Nur relevant für hybride Rust/Python RAG Stacks. | **Optional** als Beispiel-Binding erhalten |
| `contextra-types` | 90/100 | Fundament | Basis-Typen (DocId, Distanzen, Embeddings, Context) | Enthält 20 Dead-Code Stellen und alte Löschtypen. Bereinigen auf schlanke Core-Typen. | **Beibehalten** als schlanke Typ-Basis |
| `contextra-ports` | 50/100 | Fundament | Dyn-kompatible Trait-Schnittstellen (Storage, Index, Inferenz) | Sehr viele ungenutzte Traits und Lösch-Altlasten. Auf LLM-Kern-Traits reduzieren. | **Schlank halten** als reine Schnittstellen-Crate |
| `contextra-store` | 35/100 | Mittel | LSM-Tree Storage Engine, WAL, SSTable, Compaction | Riesiger Codeumfang (23k LOC). Für viele reine LLM-Tools überdimensioniert. | **Beibehalten** für Projekte mit On-Disk KV-Bedarf |
| `contextra-mvcc` | 83/100 | Mittel | Lock-free Snapshot Registry, Point-in-Time Sequence Log | Für transaktionale Speicher-Engines relevant. | **Zusammenlegen** mit `contextra-store` |
| `contextra-checkpoint` | 88/100 | Mittel | Snapshot-Management, RAII Transaction Guards | Schlankes Crate, gehört funktional zum Speicher. | **Zusammenlegen** mit `contextra-store` |
| `contextra-durable-fs` | 75/100 | Mittel | Crash-konsistentes atomares Dateischreiben, POSIX fsync | Wichtig für zuverlässige On-Disk Persistenz. | **Zusammenlegen** mit `contextra-store` |
| `contextra-sys` | 100/100 | Mittel | Unsafe OS Primitives (memlock, sync, permissions) | Autark und gehärtet (< 700 LOC). | **Zusammenlegen** mit Storage / Low-Level |
| `contextra-crypto` | 58/100 | Mittel | Verschlüsselung (ChaCha20-Poly1305, HKDF, BLAKE3-Chains) | 106 Löschketten-Stellen. Bereinigen auf Standard-Krypto-Primitive. | **Beibehalten** als optionale Security-Lib |
| `contextra-privacy` | 88/100 | Nische | Cloud Egress Filter, DLP, PII-Maskierung für LLM-Prompts | Hoher Nutzwert für Datenschutz-konforme LLM-Anwendungen (PII-Stripping). | **Beibehalten** als Prompt-Sanitizer |
| `contextra-erasure` | 77/100 | Nische | Saga-Löschung, Vault, Witness, DSGVO-Beweise | Hochgradig domänenspezifisch für Revisionssicherheit. | **Separieren** in Compliance-Spezialpaket |
| `contextra-audit-export` | 64/100 | Nische | Art. 30 Verarbeitungsverzeichnis-Export | Spezialisierte DSGVO-Logik. | **Separieren / Archivieren** |
| `contextra-avv-generator` | 64/100 | Nische | Generator für Auftragsverarbeitungsverträge (AVV) | Spezialisierte Dokumenten-Logik. | **Separieren / Archivieren** |
| `contextra-license` | 88/100 | Unnötig | Lizenz-Schlüssel-Prüfung | Proprietäre Produkt-Logik (2 Dateien). | **Entfernen / Archivieren** |
| `contextra-core` | 66/100 | Unnötig | Deprecated Strangler Facade | 1 Datei, reine Re-Exports. | **Vollständig entfernen** |
| `contextra-engine` | 10/100 | Unnötig | Monolithischer Glue-Code (verbindet alle 36 Crates) | Das Herz des alten Monolithen. Verhindert modulare Entnahme. | **Auflösen** zugunsten modularer Zusammensetzung |
| `contextra-db` | 77/100 | Unnötig | Monolithische Datenbank-Fassade | Verdrahtet die Engine. | **Ersetzen** durch schlanke `examples/` |
| `contextra` | 39/100 | Unnötig | Oberste All-In-One Fassade | Reine Produkt-Fassade. | **Ersetzen** durch Metapackage oder auflösen |
| `contextra-testkit` | 83/100 | Dev | Referenzmodelle, Benchmark-Harness, Mutant-Testing | Exzellentes Testwerkzeug für Validierung der Bausteine. | **Beibehalten** als internes Test-Kit |

---

## 4. Die Ziel-Bausteine (Von 36 Crates zu den 7 wesentlichen LLM-Modulen)

Für zukünftige Rust-LLM-Projekte wird das Repository auf **7 primäre, voneinander entkoppelte Kernbausteine** konsolidiert:

### 4.1 `rust-llm-index` (Hybride Suche & RAG)
* **Bestehend aus:** `contextra-vector` + `contextra-simd` + `contextra-text` + `contextra-rank`
* **Zweck:** Schnelle Vektorsuche (HNSW) und Volltextsuche (BM25) mit deterministischer Score-Fusion (RRF).
* **Entnahme-Faktor:** Sehr einfach. Reine In-Memory-Datenstrukturen, optional mit Disk-Persistenz. Null DSGVO- oder Krypto-Ballast.

### 4.2 `rust-llm-infer` (Einheitliche LLM-Inferenz)
* **Bestehend aus:** `contextra-infer-candle` + `contextra-infer-onnx` + `contextra-infer-ollama`
* **Zweck:** Einheitliches Trait `LlmInferenceEngine` und `EmbeddingEngine`.
* **Features:**
  * `candle`: Für eigenständige Rust-Binaries mit GGUF-Modellen (Llama-3, Mistral, Qwen, Phi).
  * `onnx`: Für hocheffiziente Einbettungen (BGE, MiniLM) und Cross-Encoder Re-Ranking.
  * `ollama`: Für Anbindung an bestehende Ollama-Server im Netzwerk.

### 4.3 `rust-llm-cache` (Prompt- & KV-Cache)
* **Bestehend aus:** `contextra-kvcache`
* **Zweck:** Prefix-Radix Baum zur Erkennung und Wiederverwendung identischer Prompt-Präfixe (z. B. System-Prompts, lange Kontext-Dokumente).
* **Benefit:** Massive Einsparung von Berechnungszeit und Token-Kosten.

### 4.4 `rust-llm-graph` (GraphRAG & Knowledge Graphs)
* **Bestehend aus:** `contextra-graph`
* **Zweck:** Effiziente Speicherung von Wissensgraphen (CSR-Format) zur Abfrage von Entitäten und Beziehungen in Kombination mit Vektorsuche (GraphRAG).

### 4.5 `rust-llm-agent` (Agenten-Loops & Werkzeuge)
* **Bestehend aus:** `contextra-agent` + `contextra-cognition` + `contextra-router` + `contextra-sandbox`
* **Zweck:** ReAct-Agenten, Multi-Model Router, Memory-Hierarchien und sandboxed WASM-Toolausführung.

### 4.6 `rust-llm-ingest` (Dokument-Extraktion)
* **Bestehend aus:** `contextra-ingest`
* **Zweck:** Parsen und Normalisieren von PDFs, Office-Dateien, Webseiten und strukturierten Daten in sauberen Markdown/Text für LLM-Pipelines.

### 4.7 `rust-llm-mcp` (Model Context Protocol Integration)
* **Bestehend aus:** `contextra-mcp` + `contextra-wire`
* **Zweck:** Bereitstellung von Tools, Ressourcen und Kontext für Claude, Cursor und andere MCP-fähige LLM-Hosts.

---

## 5. Entwickler-Leitfaden: Wie entnehme ich einen Baustein?

Wenn du als Junior-Entwickler eine Funktion für ein neues Projekt benötigst, gehe wie folgt vor:

### Szenario A: Du brauchst nur den HNSW-Vektorindex
1. Kopiere das Crate-Verzeichnis `crates/contextra-vector` (oder nach Fusion `rust-llm-index/src/vector`).
2. Kopiere `crates/contextra-simd` mit (wird für SIMD-Beschleunigung benötigt).
3. In der `Cargo.toml` deines neuen Projekts:
   ```toml
   [dependencies]
   my-vector-index = { path = "../bausteine/contextra-vector" }
   ```
4. Der Vektorindex funktioniert unabhängig von Datenbanken oder Servern direkt im Speicher!

### Szenario B: Du willst Dokumente für ein LLM vorbereiten (Ingestion)
1. Kopiere `crates/contextra-ingest`.
2. Das Crate benötigt fast keine Workspace-Abhängigkeiten (nur Basis-Typen, die leicht durch Standardtypen ersetzt werden können).
3. Rufe einfach `extract_text(&path)` auf, um sofort sauberen Text aus PDFs, DOCX oder Excel zu erhalten.

### Szenario C: Du willst einen lokalen MCP-Server für dein eigenes Tool bauen
1. Kopiere `crates/contextra-mcp` und `crates/contextra-wire`.
2. Definiere deine Rust-Funktion als MCP-Tool.
3. Starte den Server über Stdio oder HTTP-SSE. Schon kann jedes LLM (wie Claude) dein Rust-Tool ausführen.

---

## 6. Konkreter Umsetzungsfahrplan für dieses Repository

1. **Phase 1: Altlasten entsorgen (Dead Weight Deletion)**
   - Entfernen von `contextra-core` (bereits dekommissioniert).
   - Auslagern von `contextra-license`, `contextra-audit-export` und `contextra-avv-generator`.
2. **Phase 2: Entkopplung der RAG- & Inferenz-Kerne**
   - Entfernen aller Deletion-Proofs und DSGVO-Imports aus `contextra-vector`, `contextra-text` und `contextra-store`.
   - Vereinheitlichung der Inferenz-Schnittstellen (`candle`, `onnx`, `ollama`).
3. **Phase 3: Konsolidierung der Bausteine**
   - Zusammenführung in die 7 autarken Zielmodule.
   - Bereitstellung von minimalen, selbsterklärenden `examples/` für jeden Baustein.
4. **Phase 4: Baustein-Dokumentation & Vorlagen**
   - Erstellung kompakter "Quickstart-Templates", damit jedes Modul mit wenigen Zeilen Code in Drittprojekten instanziiert werden kann.
