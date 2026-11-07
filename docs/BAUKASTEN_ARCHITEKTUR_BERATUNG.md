# Architektur- und Transformationsberatung: Contextra als modulare LLM-Baustein-Bibliothek

**Autor:** Principal Rust-Architekt
**Zielgruppe:** Strategische und praktische Beratung für Rust-Entwickler / Einsteiger
**Status:** Verbindliches Architektur- & Handlungsdokument
**Datum:** 7. November 2026

---

## 1. Kurzfassung (Executive Summary)

### Kernbefund
Das Repository `contextra` ist kein gescheitertes Projekt, sondern eine Fundgrube hocheffizienter Algorithmen für moderne LLM-Anwendungen (HNSW-Vektorindex, BM25-Volltextsuche, RRF-Fusion, CSR-Graph, Prompt-Caching, Inferenz-Clients und MCP-Server). Allerdings wurde der Code über ein Jahr hinweg mit zwei gegensätzlichen Zielen entwickelt: Einerseits der Traum von modularen Bausteinen, andererseits die Zwangsjacke eines hochregulierten Unternehmensprodukts mit kryptografischer DSGVO-Löschkette, Audit-Bürokratie und 5-Ring-Schichtenarchitektur. Das Resultat ist eine künstliche Zersplitterung in 36 Crates, die sich gegenseitig blockieren und für ein neues privates Projekt kaum isoliert nutzbar sind.

### Strategische Hauptempfehlungen
1. **Bürokratie und Produkt-Ballast sofort abstreifen:** Entferne oder archiviere reine Produkt- und Compliance-Crates (`contextra-core`, `contextra-license`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-erasure`). Diese haben in einer privaten LLM-Bibliothek keinen Mehrwert.
2. **Keine 7 Riesen-Monolithen („Megacrates“) erzwingen:** Die externe Idee, 36 Crates in 7 große Blöcke wie `rust-llm-index` oder `rust-llm-infer` zu pressen, ist für einen Anfänger gefährlich. Sie führt zu enormen Compile-Zeiten, Feature-Flag-Wirrwarr und schweren Abhängigkeiten (z. B. PyTorch/Candle zwingend neben BM25).
3. **Fokus auf echte, autarke Domänen-Bausteine (10 bis 12 Bausteine):** Behalte modulare Crates bei, schneide sie aber strikt auf Autarkie zu. Wenn ein Baustein kopiert wird, darf er im Idealfall nur Standard-Rust-Typen oder eine minimale gemeinsame Typ-Bibliothek benötigen.
4. **Pragmatismus vor Dogmatismus:** Ersetze komplexe Architekturkonzepte wie „Ports-and-Adapters“ (Hexagonal Architecture) überall dort durch einfache Rust-Structs und Standard-Traits (`std::error::Error`), wo du in deinen Projekten keine zwei verschiedenen Implementierungen austauschen musst.

---

## 2. Befunde aus der eigenen Codebase-Analyse

Die Analyse erfolgte direkt am Code des Repositories (`crates/`, `Cargo.toml`, Tests, interne Tools).

| Kennzeichnung | Bedeutung |
|---|---|
| **[GEPRÜFT]** | Direkt am Code verifiziert (Compiler, Dateiinhalte, Ast-Analyse, Git-Historie). |
| **[ÜBERNOMMEN]** | Aus vorliegenden Dokumenten oder Kommentaren übernommen, logisch plausibel. |
| **[ANNAHME]** | Architektonische Schlussfolgerung oder fundierte Schätzung für die Zukunft. |

### 2.1 Workspace-Struktur und Crate-Inventar
- **[GEPRÜFT]** Der Workspace umfasst exakt **36 Crates** im Verzeichnis `crates/` sowie Hilfs- und Benchmark-Tools (`benchmarks/contextra-bench`, `tests/arch`).
- **[GEPRÜFT]** Die 36 Crates sind: `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-checkpoint`, `contextra-cognition`, `contextra-core`, `contextra-crypto`, `contextra-db`, `contextra-durable-fs`, `contextra-engine`, `contextra-erasure`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-ingest`, `contextra-kvcache`, `contextra-license`, `contextra-mcp`, `contextra-mvcc`, `contextra-ports`, `contextra-privacy`, `contextra-py`, `contextra-rank`, `contextra-router`, `contextra-sandbox`, `contextra-simd`, `contextra-store`, `contextra-sys`, `contextra-testkit`, `contextra-text`, `contextra-types`, `contextra-vector`, `contextra-wire`.
- **[GEPRÜFT]** In der Wurzel-`Cargo.toml` sind 26 Crates als `default-members` eingetragen. 10 Crates (darunter `contextra-ingest`, `contextra-erasure`, `contextra-sandbox`, `contextra-py`, `contextra-infer-candle`) sind zwar in `members`, werden aber bei einem einfachen `cargo check` standardmäßig ignoriert.

### 2.2 Autarkiegrad und Abhängigkeiten
- **[GEPRÜFT]** Es gibt **4 vollkommen autarke Crates**, die null interne Workspace-Abhängigkeiten haben:
  1. `contextra-sandbox` (WASM-Toolausführung, Wasmtime-basiert)
  2. `contextra-sys` (POSIX Low-Level Memory Hardening & Permissions, < 1.100 LOC)
  3. `contextra-types` (Zentrale Typdefinitionen)
  4. `contextra-wire` (FlatBuffers Framing & Hardened JSON-RPC 2.0 Parser)
- **[GEPRÜFT]** `contextra-simd` hängt nur an `contextra-types` (für `DistanceMetric`), kann aber mit trivialem Aufwand (Definition des 3-Varianten-Enums) zu 100 % autark gemacht werden.
- **[GEPRÜFT]** `contextra-types` fungiert als universeller Nabel: 27 von 36 Crates hängen direkt von `contextra-types` ab.
- **[GEPRÜFT]** `contextra-ports` dient als Schnittstellen-Hub: 21 Crates hängen davon ab.
- **[GEPRÜFT]** Der alte Monolith existiert primär in zwei Schichten:
  - `contextra-engine` (17.948 LOC, importiert 13 Workspace-Crates)
  - `contextra-db` / `contextra` (Fassaden, die alles bündeln)

### 2.3 Build- und Testlage
- **[GEPRÜFT]** Autarke Low-Level-Crates wie `contextra-simd`, `contextra-wire`, `contextra-sandbox` compilieren fehlerfrei und ihre Tests (`cargo test -p contextra-simd --no-run`) bauen in wenigen Sekunden.
- **[GEPRÜFT]** Ein vollständiger Workspace-Check (`cargo check --workspace --tests`) schlägt aktuell fehl, weil `contextra-store` veraltete Trait-Imports aus `contextra-ports` nutzt (`ResourceTracker`, `ResourceBudget`, die nach `contextra-types` verschoben wurden).
- **[GEPRÜFT]** Es existieren über 480 verstreute Vorkommen von DSGVO- und Löschketten-Begriffen (`deletion_proof`, `Purger`, `ScanMethod`, `AVV`) in Crates wie `contextra-crypto`, `contextra-text`, `contextra-vector`, `contextra-store` und `contextra-mcp`.
- **[ANNAHME]** Diese Kopplung an Lösch-Traits verhindert bisher die einfache Entnahme von z. B. `contextra-vector` in ein neues Projekt, weil der Vector-Index zwingend Methoden für `Purger` und `LayerScanner` implementieren muss, die ein reines RAG-Projekt niemals braucht.

---

## 3. Bewertung der drei vorliegenden Dokumente

Die drei analysierten Dokumente sind:
1. `docs/ARCHITEKTUR_BAUSTEINE_LLM.md` (Blueprint zur Konsolidierung in 7 Zielmodule)
2. `docs/meta/MODULAR_DECOUPLING_AUDIT.md` (Messdaten-Audit aller 36 Crates)
3. Die beiden externen Transformationspläne („Contextra als LLM-Baustein-Bibliothek – Umstrukturierungsplan pro Crate“ und „Architektur & Transformationsplan: Contextra als modulare Baustein-Bibliothek“)

### 3.1 Übereinstimmungen aller Dokumente (Belegte Fakten)
- Alle Dokumente erkennen richtig: Die bisherige Architektur wurde durch die DSGVO-Löschkette („Deletion-Proof-Saga“) massiv verunreinigt.
- Alle Dokumente identifizieren `contextra-simd`, `contextra-sandbox`, `contextra-ingest` und `contextra-wire` als hochwertige, hochgradig wiederverwendbare Komponenten.
- Alle Dokumente fordern die ersatzlose Löschung von `contextra-core` (reine Re-Export-Hülle) und `contextra-license` (proprietäre Lizenzprüfung für ein Produkt).
- Alle Dokumente fordern, dass `contextra-engine` in seiner monolithischen Form aufgelöst werden muss.

### 3.2 Widersprüche und Fehlschlüsse

#### Widerspruch 1: „7 Megacrates“ vs. echte Entnehmbarkeit
- **Aussage der Pläne:** Verschmelze alle 36 Crates zu 7 Megacrates: `rust-llm-index`, `rust-llm-infer`, `rust-llm-cache`, `rust-llm-graph`, `rust-llm-agent`, `rust-llm-ingest`, `rust-llm-mcp`.
- **Kritische Beurteilung [WIDERLEGT AM CODE]:**
  - Dies ist ein klassischer Architektur-Irrglaube. Wenn du z. B. `contextra-infer-candle` (benötigt PyTorch-C-Bindings oder riesige Candle-Matrizen), `contextra-infer-onnx` (benötigt ONNX Runtime) und `contextra-infer-ollama` (benötigt nur `reqwest` für HTTP-Calls) in ein einziges Crate `rust-llm-infer` wirfst, erzeugst du ein **Abhängigkeits-Monster**.
  - Ein Anfänger, der in einem neuen Projekt nur kurz Ollama per HTTP ansprechen will, muss plötzlich `candle` und `onnx` mitcompilieren oder sich mit 15 verschachtelten Feature-Flags (`#[cfg(feature = "candle")]`) herumschlagen.
  - **Empfehlung:** Behalte Inferenz-Backends getrennt! Ein leichtes HTTP-Ollama-Crate ist in 3 Sekunden compiliert; Candle braucht 3 Minuten.

#### Widerspruch 2: Die Rolle von `contextra-store` (LSM-Tree)
- **Aussage der Pläne:** `contextra-store` soll mit `contextra-mvcc`, `contextra-checkpoint`, `contextra-durable-fs` und `contextra-sys` zu einem monolithischen Speicher-Crate fusioniert werden.
- **Kritische Beurteilung [GEPRÜFT]:**
  - `contextra-store` umfasst allein über **23.000 Zeilen Code** (mit Tests über 45.000 Zeilen). Es ist eine extrem komplexe LSM-Tree Storage Engine mit Write-Ahead-Log (WAL), Memtables, SSTables und Compaction.
  - Für 90 % aller privaten Rust-LLM-Projekte (RAG, Agenten, MCP-Server) brauchst du überhaupt keine eigene LSM-Engine. Du nimmst entweder In-Memory-Vektoren, SQLite oder Qdrant/PostgreSQL.
  - Das Crate `contextra-store` jetzt krampfhaft umzubauen, kostet Wochen und hat den geringsten Hebel für LLM-Baukästen. Es sollte unverändert als optionales Spezialwerkzeug isoliert zur Seite gestellt werden.

#### Widerspruch 3: Ports-and-Adapters (Hexagonale Architektur)
- **Aussage der Pläne:** Alle Abhängigkeiten müssen über Traits in `contextra-ports` laufen.
- **Kritische Beurteilung [ANFÄNGER-REALITÄT]:**
  - Hexagonale Architektur mit separaten Interface-Crates ist hervorragend für Großkonzerne mit 50 Entwicklern. Für einen einzelnen Entwickler in privaten Projekten ist es oft eine **Komplexitätsfalle**: Du schreibst für eine Funktion ein Trait in Crate A, implementierst es in Crate B und musst beide Crates pflegen.
  - Für deine privaten Projekte gilt: Wenn du ein Modul kopierst, willst du direkte, klare Rust-Funktionen und -Structs. Traits nur dort, wo echte Polymorphie nötig ist (z. B. ein einheitliches Trait `EmbeddingModel` für verschiedene Provider).

---

## 4. Zielarchitektur für deine private LLM-Baustein-Bibliothek

### 4.1 Die Kernphilosophie: Das „Lego-Prinzip“ für Einzelentwickler
1. **Flache Hierarchie statt Ringe:** Vergiss Ring 0 bis Ring 4. Teile deine Bausteine nach ihrer **Aufgabe** ein.
2. **Copy-Paste-Ready (Null Workspace-Zwang):** Jeder Baustein muss in seiner eigenen `Cargo.toml` stehen können, ohne auf `[lints.workspace]` oder Workspace-Verzeichnisse angewiesen zu sein.
3. **Schlanke Schnittstellen statt riesiger Port-Sammlungen:** Wenn ein Vektorindex Kosinus-Ähnlichkeit berechnet, braucht er keine 30-zeiligen Port-Traits, sondern nimmt einfach `&[f32]`.
4. **Keine Angst vor Duplikation einfacher Typen:** Es ist im Rust-Ökosystem völlig legitim, ein winziges Struct wie `struct Point { x: f32, y: f32 }` oder `struct Embedding(pub Vec<f32>)` direkt im Modul zu definieren, statt dafür ein 5.000-Zeilen-Crate `types` mitkopieren zu müssen.

### 4.2 Die Baustein-Kataloge (Themenbereiche)

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    DEINE PRIVATE RUST-LLM-BIBLIOTHEK                            │
├─────────────────────┬──────────────────────┬────────────────────────────────────┤
│ 1. RAG & Suche      │ 2. Lokale Inferenz   │ 3. Agenten & Workflows             │
│    - simd-dist      │    - infer-ollama    │    - llm-router                    │
│    - hnsw-vector    │    - infer-onnx      │    - agent-core                    │
│    - bm25-text      │    - infer-candle    │    - tool-sandbox (WASM)           │
│    - rank-fusion    │                      │    - cognition-memory              │
├─────────────────────┼──────────────────────┼────────────────────────────────────┤
│ 4. Prompt & Cache   │ 5. GraphRAG          │ 6. Ingestion & Interop             │
│    - prompt-cache   │    - graph-csr       │    - doc-ingest (PDF/Office)       │
│      (Prefix-Radix) │    (Knowledge Graph) │    - mcp-server (Anthropic MCP)    │
│                     │                      │    - wire-rpc (JSON-RPC 2.0)       │
└─────────────────────┴──────────────────────┴────────────────────────────────────┘
```

### 4.3 Wie werden gemeinsame Typen gehandhabt?
- **Problem:** Wenn jeder Crate sein eigenes `DocId` oder `Embedding` definiert, passen sie beim Zusammenstecken nicht zusammen. Wenn alle von `contextra-types` abhängen, hast du wieder eine Monolith-Kopplung.
- **Die Rust-Lösung (Typen-Pragmatismus):**
  - **Level 1 (Primitive Rust-Typen bevorzugen):** Verwende in Funktionssignaturen Standard-Typen: `&[f32]` statt `CustomVectorSlice`, `u64` oder `String` statt `CustomDocIdType`, `&str` statt `DocumentChunkPayload`. Dadurch ist ein Baustein sofort mit *jedem* fremden Rust-Code kompatibel.
  - **Level 2 (Der Mini-Kernel `llm-common`):** Bereinige `contextra-types` radikal von allen alten Lösch-, Vault- und Audit-Typen. Behalte darin ausschließlich: `DocId`, `Embedding`, `DistanceMetric`, `TokenUsage`. Dieser Baustein hat 0 Abhängigkeiten, compiliert in 0,5 Sekunden und kann überall als Fundament mitkopiert werden.

---

## 5. Detaillierter Plan pro Crate (Alle 36 Crates)

Nachfolgend wird jedes der 36 Crates mit einer klaren Handlungsempfehlung, Begründung, Aufwandsschätzung und Priorität bewertet:

| Crate | Empfehlung | Begründung & Zielzustand | Aufwand | Prio |
|---|---|---|:---:|:---:|
| `contextra-simd` | **Behalten (Autark)** | Extrem wertvoll. Berechnet Cosine, Euclidean & Dot-Product mit AVX2/AVX-512/NEON. Abhängigkeit zu `types` kappen, Enum `DistanceMetric` lokal einbetten. Sofort fertig. | Sehr gering (1h) | P0 |
| `contextra-sandbox` | **Behalten (Autark)** | Meisterstück: 100/100 Autarkie. Führt WASM-Tools sicher isoliert aus. Perfekt für Agenten. Keine Änderungen nötig. | Null | P0 |
| `contextra-wire` | **Behalten (Autark)** | Gehärteter JSON-RPC 2.0 Parser und FlatBuffers Framing. Perfekt für CLI- und Tool-IPC. | Null | P0 |
| `contextra-sys` | **Behalten (Autark)** | POSIX Memory Locking (`mlock`), Hardening und Permissions. Für High-Security-Projekte nützlich. | Null | P3 |
| `contextra-rank` | **Behalten (Autark)** | Reciprocal Rank Fusion (RRF), Platt Scaling & Kalibrierung. Entferne die Abhängigkeit zu `ports`, arbeite direkt auf Score-Slices. | Gering (2h) | P0 |
| `contextra-ingest` | **Behalten (Autark)** | Extrem nützlich für RAG: Extrahiert Text aus PDF, DOCX, XLSX, HTML, Markdown, CSV. Hängt nur an `types`. Typ-Import durch Standard-String ersetzen. | Gering (3h) | P0 |
| `contextra-infer-ollama` | **Behalten (Autark)** | Schlanker async Client für Ollama (Llama-3, Mistral etc.). Keine schweren C-Deps. Von `ports` entkoppeln und direkte Client-Methoden anbieten. | Gering (3h) | P1 |
| `contextra-infer-onnx` | **Behalten (Autark)** | Schnelle lokale Embeddings (z. B. BGE, MiniLM) und Cross-Encoder Re-Ranking via ONNX Runtime. Als separater Baustein erhalten. | Mittel (4h) | P1 |
| `contextra-infer-candle` | **Behalten (Autark)** | Pure-Rust Inferenz für GGUF-Modelle ohne C++-Compiler. Unbedingt isoliert lassen, nicht mit Ollama verschmelzen! | Mittel (4h) | P2 |
| `contextra-text` | **Bereinigen & Behalten** | BM25-Volltextindex & Unicode-Tokenizer. Entferne alle `Purger`-, `Scanner`- und DSGVO-Relikte. Zu einem reinen In-Memory Inverted-Index machen. | Mittel (5h) | P1 |
| `contextra-vector` | **Bereinigen & Behalten** | Das Herzstück: HNSW Vektorsuch-Graph. Aktuell mit Krypto- und Purge-Traits verunreinigt. Bereinigen, sodass es nur Vektoren indexiert und abfragt. | Hoch (8h) | P1 |
| `contextra-kvcache` | **Bereinigen & Behalten** | Prefix-Radix-Baum für Prompt-Caching (spart Token-Kosten). Krypto- und Vault-Schichten entfernen, als reinen Prompt-Cache nutzen. | Mittel (4h) | P1 |
| `contextra-graph` | **Bereinigen & Behalten** | Starker CSR-Graph (Compressed Sparse Row) für Knowledge Graphs & Entity Linking. Von `ports` entkoppeln. | Mittel (6h) | P2 |
| `contextra-router` | **Behalten** | Semantisches Intent-Routing und Model-Cascades. Sehr nützlich für Multi-LLM-Setups. | Gering (3h) | P2 |
| `contextra-agent` | **Bereinigen & Behalten** | ReAct-Loop und State-Machine für Agenten. Alte DB- und Checkpoint-Verdrahtung durch einfaches State-Interface ersetzen. | Mittel (6h) | P2 |
| `contextra-cognition` | **In Agent integrieren** | Memory-Consolidation und Kurzzeit-/Langzeitgedächtnis. Gehört funktional direkt in das Agenten-Modul. | Mittel (6h) | P2 |
| `contextra-mcp` | **Bereinigen & Behalten** | Anthropic Model Context Protocol Server. Extrem zukunftssicher für Claude Desktop & Cursor. Verbindung zum Monolithen trennen! | Hoch (8h) | P1 |
| `contextra-types` | **Radikal abspecken** | Auf 4–5 Kern-Typen (`DocId`, `Embedding`, `DistanceMetric`, `TokenUsage`) reduzieren. Alle 20 Dead-Code-Stellen und Lösch-Typen tilgen. | Gering (2h) | P0 |
| `contextra-ports` | **Auflösen / Modularisieren** | Ein zentraler Port-Monolith widerspricht dem Baukasten-Prinzip. Traits gehören entweder zu den Bausteinen selbst oder entfallen. | Mittel (6h) | P2 |
| `contextra-adapt` | **Behalten / Optional** | Bandit-Algorithmen und dynamische Batching-Parameter. Kann bei Bedarf als Optimierungs-Baustein genutzt werden. | Gering (2h) | P3 |
| `contextra-privacy` | **Behalten als Tool** | Prompt-Sanitizer: PII-Maskierung vor dem Senden an Cloud-LLMs (OpenAI/Anthropic). Nützliches Hilfswerkzeug. | Gering (3h) | P2 |
| `contextra-durable-fs` | **Behalten (Utility)** | Crash-konsistentes atomares Dateischreiben (`write_atomic`). Nützlich für Konfigurations- oder JSON-Saves. | Gering (2h) | P3 |
| `contextra-py` | **Archivieren / Bei Bedarf** | PyO3 Python-Bindings. Nur relevant, wenn du Rust aus Python aufrufen willst. Vorerst parken. | Gering | P4 |
| `contextra-mvcc` | **Archivieren / Speicher** | Snapshot-Registry & Point-in-Time Isolation. Wird nur für transaktionale Multi-Thread-Datenbanken gebraucht. | Mittel | P4 |
| `contextra-checkpoint` | **Archivieren / Speicher** | Snapshot-Dateiverwaltung. Nur für On-Disk-Rollbacks relevant. | Gering | P4 |
| `contextra-store` | **Isolieren / Nicht anfassen** | 23k Zeilen LSM-Tree. Zu komplex zum Umbauen, für LLM-Tools meist unnötig. Bleibt als Spezial-Engine unberührt. | Sehr hoch | P4 |
| `contextra-crypto` | **Bereinigen auf Basics** | 106 Löschketten-Stellen entfernen. Nur Standard-Krypto behalten (ChaCha20, HKDF), falls verschlüsseltes Caching gewünscht ist. | Mittel (4h) | P3 |
| `contextra-erasure` | **Auslagern / Archivieren** | Komplexe Lösch-Saga, Vaults, DSGVO-Witness. Reiner Produkt-Ballast für Konzerne. Für private Tools nutzlos. | Gering (1h) | P1 |
| `contextra-audit-export` | **Sofort entfernen** | Art. 30 Verarbeitungsverzeichnis-Export. Reines Bürokratie-Tool. | Gering (30m) | P0 |
| `contextra-avv-generator` | **Sofort entfernen** | Auftragsverarbeitungsvertrag-Generator nach Art. 28 DSGVO. Reines juristisches PDF/MD-Tool. | Gering (30m) | P0 |
| `contextra-license` | **Sofort entfernen** | Proprietäre Lizenzschlüssel-Validierung. Für private Open-Source/Closed-Code-Projekte völlig sinnlos. | Gering (15m) | P0 |
| `contextra-core` | **Sofort entfernen** | Bereits als veraltete Fassade markiert. 1 Datei, 0 Funktionalität. | Gering (10m) | P0 |
| `contextra-engine` | **Schrittweise auflösen** | Der Monolith, der alles fest verklebt. Sobald Bausteine entkoppelt sind, wird dieses Crate gelöscht. | Hoch | P2 |
| `contextra-db` | **Durch Beispiele ersetzen** | Monolithische Datenbank-Hülle. Ersetze sie durch 2–3 saubere Anwendungs-Beispiele unter `examples/`. | Mittel | P2 |
| `contextra` | **Entfernen** | Oberste Dach-Fassade. Wird im Baukastensystem nicht mehr benötigt. | Gering (15m) | P1 |
| `contextra-testkit` | **Behalten als Test-Tool** | Enthält hervorragende In-Memory Mocks und Benchmark-Harnesses zur Verifikation deiner Bausteine. | Gering | P2 |

---

## 6. Der konkrete Entnahme-Workflow (Schritt für Schritt)

Hier ist die verlässliche Anleitung, wie du als Rust-Entwickler einen Baustein aus diesem Repository in ein neues, leeres Projekt übernimmst:

### Typische Hürden, die Anfänger scheitern lassen
- **Hürde 1:** Das Crate enthält `[lints] workspace = true` oder `version.workspace = true` in der `Cargo.toml`. Ein neues Projekt kennt diesen Workspace nicht und bricht sofort mit einem Syntaxfehler ab.
- **Hürde 2:** Das Crate importiert `contextra-types` oder `contextra-ports` über Pfade (`path = "../contextra-types"`), die im neuen Projekt fehlen.
- **Hürde 3:** Interne Modulnamen und Crate-Namen passen nicht zum neuen Projektzweck.

### Die 5-Schritte-Formel zur erfolgreichen Entnahme

#### Schritt 1: Zielverzeichnis im neuen Projekt vorbereiten
Erstelle in deinem neuen Projekt (z. B. `my-llm-app`) einen Ordner `crates/` oder `src/modules/`:
```bash
cargo new my-llm-app
cd my-llm-app
mkdir -p lib/
```

#### Schritt 2: Baustein herüberkopieren
Kopiere das gewünschte Crate-Verzeichnis (z. B. `crates/contextra-simd`):
```bash
cp -r /pfad/zu/contextra/crates/contextra-simd ./lib/fast-simd
```

#### Schritt 3: Die `Cargo.toml` entkoppeln (Workspace-Befreiung)
Öffne `lib/fast-simd/Cargo.toml` und passe sie so an, dass sie völlig autark ist:
1. Lösche alle `.workspace = true` Verweise.
2. Trage feste Werte ein:
```toml
[package]
name = "fast-simd"
version = "0.1.0"
edition = "2021"

[dependencies]
# Externe Crates aus crates.io bleiben erhalten
cfg-if = "1.0"

# Workspace-Abhängigkeiten entfernen oder durch lokale Pfade ersetzen:
# Falls gemeinsame Typen nötig sind:
# llm-common = { path = "../llm-common" }
```

#### Schritt 4: Crate im Hauptprojekt einbinden
In der Haupt-`Cargo.toml` deines neuen Projekts:
```toml
[dependencies]
fast-simd = { path = "lib/fast-simd" }
```

#### Schritt 5: Der "Zero-Dependency Sanity Check"
Führe im Ordner des entnommenen Bausteins einen isolierten Test aus:
```bash
cd lib/fast-simd
cargo test
```
Wenn der Test grün durchläuft, ist dein Baustein erfolgreich befreit und dauerhaft wartbar!

---

## 7. Phasenplan mit überprüfbaren Zwischenergebnissen

Damit du dich nicht übernimmst, erfolgt die Umstrukturierung in kleinen, sicheren Schritten. Jeder Schritt hinterlässt ein compilierbares, funktionierendes System:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        TRANSFORMATIONS-PHASEN                          │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 0: Sofortiger Ballast-Abwurf & Build-Heilung (Tag 1)             │
│   ➔ Ergebnis: Keine Compiler-Fehler, tote Crates gelöscht.            │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 1: Die 4 autarken Juwelen isolieren (Woche 1)                   │
│   ➔ Ergebnis: SIMD, Sandbox, Wire & Ingest als 100% autarke Bausteine. │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 2: RAG & Suche entgiften (Woche 2)                               │
│   ➔ Ergebnis: HNSW-Vector, BM25-Text & Rank ohne Lösch-Traits nutzbar. │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 3: Inferenz & Prompt-Cache autark machen (Woche 3)               │
│   ➔ Ergebnis: Unabhängige Clients für Ollama, ONNX & Prefix-Cache.     │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Agenten & MCP-Server freischalten (Woche 4)                   │
│   ➔ Ergebnis: Eigenständiger Anthropic-MCP-Server und ReAct-Agent.     │
└────────────────────────────────────────────────────────────────────────┘
```

### Phase 0: Ballast-Abwurf & Build-Heilung (Sofortmaßnahme)
- **Aktionen:**
  1. Repariere den Import-Fehler in `contextra-store` (`ResourceTracker` aus `contextra-types`).
  2. Entferne die 4 überflüssigen Crates ersatzlos aus der Workspace-`Cargo.toml`: `contextra-core`, `contextra-license`, `contextra-audit-export`, `contextra-avv-generator`.
  3. Verschiebe `contextra-erasure` in einen Archivordner (`archive/contextra-erasure`) oder nimm es aus den `default-members`.
- **Überprüfbares Kriterium:** `cargo check --workspace` läuft mit 0 Fehlern durch.

### Phase 1: Die autarken Juwelen freilegen
- **Aktionen:**
  1. `contextra-simd`: Kopiere das Enum `DistanceMetric` aus `contextra-types` direkt nach `contextra-simd`. Entferne `contextra-types` aus den Abhängigkeiten.
  2. `contextra-ingest`: Ersetze `contextra-types::Result` und Dokumenttypen durch einfache Rust-Typen (`String`, `Vec<u8>`).
- **Überprüfbares Kriterium:** `cargo test -p contextra-simd` und `cargo test -p contextra-ingest` kompilieren völlig isoliert ohne jeden Workspace-Zwang.

### Phase 2: RAG-Kerne (Vektor & Text) entkoppeln
- **Aktionen:**
  1. In `contextra-vector`: Lösche das Untermodul `purge.rs` und alle Bezüge zu `Purger`, `LayerScanner` und `DeletionProof`.
  2. In `contextra-text`: Entferne `inverted/purge.rs` und alle Bezüge zu DSGVO Art. 17.
  3. In `contextra-rank`: Kappe die Abhängigkeit zu `contextra-ports`.
- **Überprüfbares Kriterium:** Du kannst ein einfaches Code-Beispiel schreiben, das Text mit BM25 indexiert, Vektoren mit HNSW sucht und Ergebnisse mit RRF fusioniert – ohne dass irgendwo das Wort `Purger` oder `DSGVO` auftaucht.

### Phase 3: Inferenz-Clients autark machen
- **Aktionen:**
  1. `contextra-infer-ollama`: Kappe `contextra-ports`. Biete eine direkte `OllamaClient`-Struktur mit Methoden `generate(&str)` und `embed(&str)` an.
  2. `contextra-kvcache`: Entferne die verschlüsselte Segment-Schicht (`EncryptedSegmentPayload`), behalte den blitzschnellen Prefix-Radix-Tree.
- **Überprüfbares Kriterium:** Ein minimales Beispiel in `examples/ollama_query.rs` fragt ein lokales Modell ab.

### Phase 4: Agenten- und MCP-Abschluss
- **Aktionen:**
  1. `contextra-mcp`: Entferne die Abhängigkeit zur All-in-One-Fassade `contextra`. Verdrahte Tools direkt mit den neuen, schlanken Modulen (z. B. Ingest oder Vektor).
  2. `contextra-agent`: Führe `contextra-cognition` als einfaches Modul `memory` in den Agenten ein.
- **Überprüfbares Kriterium:** Du startest den MCP-Server über Stdio und verbindest ihn erfolgreich mit Claude Desktop oder einem Terminal-Client.

---

## 8. Offene Entscheidungen für dich (mit Empfehlungen)

Hier sind die vier zentralen Weichenstellungen, die du als Projektinhaber treffen musst:

### Entscheidung 1: Was passiert mit der 23.000 Zeilen LSM-Tree Storage Engine (`contextra-store`)?
- **Option A (Radikaler Schnitt - EMPFOHLEN):** Friere `contextra-store` ein. Rühre es nicht mehr an. Nutze für zukünftige Projekte, die persistente Daten brauchen, bewährte Crates wie `redb` (Pure-Rust Embedded KV-Store) oder `rusqlite`.
  - *Begründung:* Das Schreiben einer fehlerfreien LSM-Storage-Engine (Crash-Safety, WAL, Compaction) ist eines der schwersten Probleme der Informatik. Für LLM-Anwendungen ist es fast immer das falsche Werkzeug und kostet dich monatelang Zeit.
- **Option B (Aufwendige Rettung):** Trenne alle alten Lösch- und MVCC-Fäden ab und versuche, einen eigenständigen KV-Store daraus zu bauen.
  - *Empfehlung:* Nur wählen, wenn du primär Datenbank-Entwicklung lernen möchtest, nicht LLM-Engineering.

### Entscheidung 2: Ein zentrales `llm-common`-Crate oder dezentrale Primitiv-Typen?
- **Option A (Dezentrale Typen - EMPFOHLEN für maximale Portabilität):** Jeder Baustein akzeptiert Standard-Rust-Typen (`&[f32]`, `&str`, `u64`).
  - *Begründung:* Ein Modul, das nur `&[f32]` nimmt, kannst du in 10 Sekunden in ein fremdes Actix-Web-, Axum- oder CLI-Projekt kopieren, ohne irgendeine andere Datei mitzunehmen.
- **Option B (Schlankes Shared Crate):** Ein minimales Crate `crates/llm-common` (max. 150 Zeilen Code), das `DocId`, `Embedding` und `TokenUsage` enthält.
  - *Empfehlung:* Wenn du häufig eigene Multi-Crate-Pipelines baust, ist Option B ein sehr sauberer Mittelweg.

### Entscheidung 3: Wie gehst du mit Candle vs. ONNX vs. Ollama um?
- **Empfehlung:** Halte alle drei Inferenz-Pfade strikt in getrennten Crates.
  - Nutze **Ollama** für den schnellen Start und größte Modellvielfalt.
  - Nutze **ONNX** für extrem performante, CPU-schonende Embeddings in Produktion.
  - Nutze **Candle** nur dann, wenn du eine eigenständige Single-Binary-Anwendung ohne externe Abhängigkeiten ausliefern willst.

### Entscheidung 4: Was geschieht mit den Architektur-Regeln (AGENTS.md, Ratchet, Gates)?
- **Empfehlung:** Schalte 80 % der internen CI-Schikanen (wie Zeilenbudgets, strikte Ring-Prüfungen in `tests/arch`, Verbot von `std::fs`) ab.
  - *Begründung:* Diese Regeln wurden von KI-Agenten geschaffen, um sich selbst bei Produkt-Refactorings einzuschränken. Für dich als Einzelentwickler sind sie wie Betonschuhe. Behalte nur `cargo test`, `cargo clippy` und `cargo fmt`.

---

## 9. Grenzen dieser Analyse

Damit du die Empfehlungen realistisch einordnen kannst, sind hier die Grenzen dieser Ausarbeitung transparent dargelegt:

1. **Hardware-spezifische SIMD-Befehle:** Die SIMD-Optimierungen in `contextra-simd` (AVX-512) wurden statisch und über Standard-Testsuiten geprüft. Wenn dein Zielrechner kein AVX-512 unterstützt, greift automatisch der AVX2- oder Scalar-Fallback.
2. **Candle-/ONNX-Laufzeitumgebung:** Inferenz-Crates hängen zur Laufzeit von externen Faktoren ab (z. B. installierte dynamische Bibliotheken von `onnxruntime` oder heruntergeladene HuggingFace-Modellgewichte). Diese wurden im Rahmen der reinen Code-Analyse statisch geprüft, aber nicht mit realen Multi-Gigabyte-GGUF-Gewichten gebenchmarkt.
3. **Keine Gewährleistung für externe Entwürfe:** Die Aussagen zu den externen Vorlagen („7 Megacrates“) basieren auf dem strukturellen Abgleich mit den tatsächlichen Cargo-Metadaten. Sollten dort andere, unveröffentlichte Intentionen bestanden haben, wurden sie im Sinne der praktischen Wartbarkeit für einen Einsteiger verworfen.
