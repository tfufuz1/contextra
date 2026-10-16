# Audit-Bericht: contextra-infer-candle

**Datum:** 2026-10-04
**Auditor:** Principal Senior Rust Architect
**Crate:** `contextra-infer-candle` (Ring 2)
**Safety Status:** `#![forbid(unsafe_code)]` enforced

---

## 1. Zero-Panic-Inventar (P1)

Gemäß der Contextra Zero-Panic-Doktrin (P7 / AGENTS.md §6) darf Produktionscode keine `.unwrap()`, `.expect()` oder `panic!()` Aufrufe enthalten.

### Codebase Scan Resultate
Analyse der Suchtreffer aus `grep -rn "\.unwrap()\|\.expect(\|panic!" crates/contextra-infer-candle/src/`:

| Dateipfad | Zeile | Typ | Klassifikation | Begründung |
|---|---|---|---|---|
| `src/gguf_loader.rs` | 74, 90 | `panic!` | Test-Code | Innerhalb `#[cfg(test)] mod tests` |
| `src/kv_bridge.rs` | 319, 331, 356, 369, 379, 385, 386, 409, 442, 464, 477, 498, 510, 513, 514, 518, 519, 537, 553, 564, 567, 568, 572, 573, 594, 617, 629, 654, 665, 666, 689, 690 | `.unwrap()`, `.expect()` | Test-Code | Innerhalb `#[cfg(test)] mod tests` |
| `src/embedding.rs` | 377, 388, 408, 435, 455 | `.unwrap()`, `panic!` | Test-Code | Innerhalb `#[cfg(test)] mod tests` |
| `src/gasp.rs` | 348, 381, 405, 417, 437, 452, 463, 473, 545, 589, 611, 637, 646 | `.unwrap()`, `panic!` | Test-Code | Innerhalb `#[cfg(test)] mod tests` |
| `src/kv_state.rs` | 4 | Doc-Kommentar | Dokumentation | Invariante-Dokumentationszeile |
| `src/model/quantized_llama.rs` | 4 | Doc-Kommentar | Dokumentation | Invariante-Dokumentationszeile |
| `src/model/mod.rs` | 4 | Doc-Kommentar | Dokumentation | Invariante-Dokumentationszeile |

**EVIDENCE-P1-01:** Zero Panic Treffer in Nicht-Test-Code: **0**. Alle Treffer sind vollständig auf `#[cfg(test)]`-Module oder Modul-Kommentare beschränkt. Fehler im Produktionspfad werden sauber per `Result<T, ContextraError>` propagiert.

---

## 2. Semaphore-Backpressure-Nachweis (P2)

Inferenzoperationen auf CPU/GPU sind speicher- und rechenintensiv. Unbegrenzte `spawn_blocking`-Aufrufe würden Tokio-Threadpools erschöpfen.

### Implementation Details
1. **LLM Text Generation (`CandleLlmClient`)**:
   - In `crates/contextra-infer-candle/src/inference/core.rs`:
     - `pub semaphore: Arc<tokio::sync::Semaphore>`
     - Initialwert: `DEFAULT_MAX_CONCURRENT_INFERENCES = 4`.
     - Konfigurierbar via `.with_max_concurrent_inferences(limit: usize)` (untere Schranke `limit.max(1)`).
   - In `LlmTextGenerator::generate` & `LlmTextGeneratorStreaming::generate_stream`:
     - Akquiriert Permit vor Ausführung: `semaphore.acquire().await` bzw. `semaphore.acquire_owned().await`.
     - Ist die Semaphore voll, verharren aufrufende Futures im Async-Zustand (Backpressure / Await) anstatt neue Threads zu sperren oder Anfragen abzuweisen.

2. **Vector Embedding (`CandleEmbedClient`)**:
   - In `crates/contextra-infer-candle/src/embedding.rs` & `src/embedding_provider.rs`:
     - `pub semaphore: Arc<tokio::sync::Semaphore>`
     - Initialwert: `DEFAULT_MAX_CONCURRENT_EMBEDDINGS = 8`.
     - Konfigurierbar via `.with_max_concurrent_embeddings(limit: usize)` (untere Schranke `limit.max(1)`).
   - In `EmbeddingProvider::embed` & `embed_batch`:
     - Akquiriert Permit via `semaphore.acquire().await`.
     - Batch-Größe zusätzlich durch `MAX_CANDLE_EMBED_BATCH_SIZE = 256` begrenzt.

**EVIDENCE-P2-01:** Beide Inferenz-Clients begrenzen nebenläufige Aufgaben über konfigurierbare Async-Semaphoren (`Semaphore`). Bei Vollauslastung wird die Anfrage blockiert (asynchron gewartet), wodurch deterministische Backpressure ohne Threadpool-Exhaustion gewährleistet ist.

---

## 3. Embedding-Dimensionsvalidierungsnachweis (P3)

Inkorrekte Embedding-Dimensionen führen im HNSW-Index (`contextra-vector`) zu Speicher- und Distanzberechnungsfehlern.

### Implementation Details
- `CandleEmbedClient` speichert das Attribut `pub dim: usize`, das beim Konstruktor aus `model.dim()` ausgelesen wird.
- `BertEmbedModel` liest `config.hidden_size` ein und erzeugt durch L2-normalisiertes Mean-Pooling garantiert Vektoren der Dimension `hidden_size`.
- `DefaultCandleEmbedModel` erzeugt im Mock-Modus deterministische Vektoren der Länge `self.dim`.
- **Befund / Empfehlung:** `CandleEmbedClient` vertraut derzeit auf die Typsicherheit von `CandleEmbedInner::dim()`. Es erfolgt nach `model.embed(...)` keine explizite Laufzeitprüfung `if vec.len() != self.dim`. Es wird empfohlen, in einer zukünftigen Härtungs-Phase eine explizite Assertion/Error-Return einzufügen, falls Drittanbieter-`CandleEmbedInner`-Implementierungen integriert werden.

**EVIDENCE-P3-01:** Vektordimensionen sind im Modell-Typ statisch an `config.hidden_size` gebunden.

---

## 4. Ressourcen-Cleanup & Memory Management (P4)

- Candle Tensor-Operationen (`candle_core::Tensor`, `ModelWeights`, `BertModel`) nutzen Rust-RAII.
- Lokale Tensors innerhalb von `spawn_blocking`-Closures werden beim Verlassen des Gültigkeitsbereichs automatisch freigegeben.
- Bei KV-Cache-gestützten Generationen wird vor/nach Läufen `clear_kv_cache()` aufgerufen, um Speicherlecks über lange Sessions zu verhindern.

**EVIDENCE-P4-01:** Automatische RAII-Freigabe aller Candletensoren nach Inferenz-Abschluss; kein unkontrolliertes Speicherwachstum.

---

## 5. Feature-Gate & Default Backend Check (P5)

- In Workspace `Cargo.toml`: `crates/contextra-infer-candle` ist in `default-members` eingetragen.
- In `crates/contextra/Cargo.toml` (Facade): Standardfeatures sind `default = ["fast", "candle"]`.
- `contextra-infer-candle` ist somit das Standard-Inferenzbackend (Pure-Rust, zero external network services).

**EVIDENCE-P5-01:** Candle ist als Standard-Backend im Workspace und Facade crate konfiguriert.

---

## 6. VERDICT

```markdown
## VERDICT
- STATUS: APPROVED_WITH_RECOMMENDATIONS
- CRATE: contextra-infer-candle
- RING: 2
- UNSAFE: #![forbid(unsafe_code)]
- RECOMMENDATION: Runtime post-execution vector length check in CandleEmbedClient::embed to reinforce P3.
```
