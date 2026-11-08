# SPEC.md — `contextra-infer-ollama`

**Klasse:** Klasse A (Autarker Baustein)
**Ring:** Ring 2
**Normativitätsstatus:** Normativ im Rahmen der Bausteinspezifikation SPEC v4.0 §13.3

---

## 1. Zweck & Verantwortlichkeit
`contextra-infer-ollama` ist ein autarker REST-Client und Adapter für Ollama-Inferenzserver. Er bietet:
- Erzeugung von Vektoreinbettungen (Embeddings) über Ollama `/api/embeddings` und `/api/embed`.
- Textgenerierung und Streaming-Inferenz über `/api/generate` und `/api/chat`.
- Automatische dynamische Dimensionsvalidierung und Schutz gegen stille Dimensionsänderungen.
- Resiliente HTTP-Kommunikation mit konfigurierbarem Retry, Exponential-Backoff mit Jitter und Circuit-Breaker.
- Egress-Richtlinienprüfung (Egress Policy) mit Audit-Sink-Anbindung.
- Offline-Verarbeitung für XML-Prompt-Escaping, Strukturierte Prompt-Formattierung, Kontext-Präfixierung und Few-Shot Wichtigkeits-Scoring.

---

## 2. Baustein-Eigenschaften (B1–B12)
1. **B1 / B2 (Keine internen Abhängigkeiten):** Im regulären Standalone-Modus besitzt die Crate 0 Abhängigkeiten zu anderen Workspace-Crates. Interne Typen (`contextra-types`, `contextra-ports`) existieren nur hinter dem transitionalen Feature `ports-compat` (bis Welle 6).
2. **B3 (Standardtypen):** Öffentliche APIs verwenden Standardtypen (`String`, `&str`, `Vec<f32>`, `Duration`) und eigene Typen (`OllamaConfig`, `ImportanceAssessment`).
3. **B4 (Einheitliches Fehlermodell):** Alle Operationen nutzen `OllamaError` (`thiserror::Error`, `#[non_exhaustive]`, zero-panic).
4. **B5 (Isolierter Build):** Kompiliert und testet vollständig autark aus einem leeren Verzeichnis via `tools/export_crate.py`.
5. **B6 (Feature-Minimalismus):** `default = []`. Netzwerk- und Async-Abhängigkeiten (`reqwest`, `tokio`, `futures`) sind hinter dem Feature `http` gekapselt.
6. **B7 (Keine Fremdfachlogik):** Keine Löschlogik, keine Transaktionen (`TxId`), keine Mandantenkaskaden.
7. **B8 (Dokumentation & Benchmarks):** Vollständige `README.md`, `SPEC.md`, Moduldokumentation (`//!`) und Criterion-Benchmarks (`benches/ollama_bench.rs`).
8. **B9 (Zero-Panic):** Kein `unwrap()`, `expect()`, `panic!` oder unbedachtes Indexing in Produktivpfaden.
9. **B10 (Zero Unsafe):** `#![forbid(unsafe_code)]` strikt deklariert und eingehalten.
10. **B11 (Determinismus & Abstraktion):** Zeit (`Clock`) und Zufall (`Rng`) sind abstrahiert mit deterministischem `SeededRng` für Jitter.
11. **B12 (Lizenzen & Version):** Eigene Lizenzdateien (`LICENSE-MIT`, `LICENSE-APACHE`) und Spezifikation der Mindest-Rust-Version.

---

## 3. Verträge & Invarianten

- **INV-OLLAMA-NET-1 (Kein rohes HTTP):** Jeder Netzwerkaufruf läuft über `OllamaClient` mit Timeout, Retry, Circuit-Breaker und Backoff.
- **INV-OLLAMA-EGRESS-1 (Fail-Closed Egress):** Externe Netzwerkaufrufe validieren die `EndpointPolicy`. Nicht erlaubte Remotes oder fehlende Audit-Sinks brechen deterministisch mit Fehler ab.
- **INV-OLLAMA-DIM-1 (Dimensionssperre):** Die beobachtete Embedding-Dimension wird beim ersten Aufruf festgeschrieben (`observed_dimension`). Bei abweichenden Dimensionen in Folgeaufrufen wird sofort fail-closed abgebrochen (`OllamaError::DimensionMismatch`).
- **INV-OLLAMA-SAN-1 (Strukturelle Prompt-Isolation):** Sämtliche Nutzertiefen und Kontextabschnitte werden via `xml_escape` escaped, um XML-/Prompt-Injektionen zu verhindern.
- **INV-OLLAMA-ASYNC-1 (Async nur bei I/O):** Rechnerische und parsing-bezogene Algorithmen (Prompt-Building, XML-Escape, Response-Parsing, ModelInfo-Nachschlagen) sind vollständig synchron und kompilieren ohne `http`-Feature.

---

## 4. Features

| Feature | Zweck | Standard |
|---|---|---|
| `http` | Bindet `reqwest`, `tokio`, `futures`, `futures-util` für Netzwerk-Client | Nein |
| `ports-compat` | Transitionale Anbindung an `contextra-ports` und `contextra-types` (Deprecation bis Welle 6) | Nein |
| `types-compat` | Alias für `ports-compat` | Nein |
