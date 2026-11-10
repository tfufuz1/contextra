# Spezifikation: `contextra-infer-candle`

**Ring:** Ring 2 (External Adapters & Inferenz)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §6.2
**Klasse:** A (Autarker Inferenzbaustein)
**MSRV:** 1.89.0
**Code-Disziplin:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra-infer-candle` stellt In-Process Inferenz für Transformer- und Einbettungsmodelle via Hugging Face Candle bereit:
- **Scope:** Laden von GGUF- und SafeTensors-Gewichten, token-weise Textgenerierung, Vektoreinbettungen und Gasp-Grounding-Validierung.
- **Autarkie:** Candle-Abhängigkeiten (`candle-core`, `candle-nn`, `candle-transformers`, `tokenizers`, `half`) sind hinter dem Feature `candle` isoliert. Ohne `candle` baut der Baustein als schlanker Typen- und Fake-Engine-Kern.

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra-infer-candle`:*
- **LOC:** 5.150 LOC über 20 Rust-Dateien.
- **Typen-Inventar:** 6 Traits, 35 Structs, 3 Enums, 73 öffentliche Funktionen (`pub fn`).
- **Abhängigkeiten:** 4 interne Crates (`contextra-types`, `contextra-ports`, `contextra-rank`, `contextra-kvcache`).
- **Audit-Befund:** 0 Deletion-Leaks. Starke Kopplung an `candle-core` erfordert striktes Feature-Gating.

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Typen |
|---|---|---|
| `embedding.rs` | Vektoreinbettung via Bert/Transformer-Architekturen | `CandleEmbedder`, `EmbeddingConfig` |
| `inference.rs` | Auto-regressive Textgenerierung und Logit-Sampling | `CandleGenerator`, `GenerationParams` |
| `gguf_loader.rs` | Nullkopie-Laden von quantisierten GGUF-Modellen | `GgufModel`, `ModelFingerprint` |
| `gasp.rs` | Grounding-Aware Selective Pruning & Vertrauens-Kalibrierung | `GroundingValidator`, `ConfidenceScore` |
| `fake.rs` | Deterministische Test-Double ohne Modellgewichte | `FakeCandleEmbedder`, `FakeGenerator` |
| `error.rs` | Spezifischer Inferenz-Fehlertyp | `CandleError`, `Result<T>` |

---

## 4. Invarianten (INV-CND-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-CND-ZERO-UNSAFE` | `#![forbid(unsafe_code)]` wird im Crate strikt erzwungen. | `cargo check -p contextra-infer-candle` |
| `INV-CND-DETERMINISM-1` | Identische Eingabetexte und feste Random-Seeds liefern bit-identische Logits und Embeddings. | `tests/candle_determinism.rs` |
| `INV-CND-FEATURE-ISOLATION` | `--no-default-features` kompiliert vollständig ohne externe C-/CUDA-/ML-Abhängigkeiten. | `cargo check -p contextra-infer-candle --no-default-features` |
| `INV-CND-FINGERPRINT-1` | Geladene Modelle werden kryptografisch über `ModelFingerprint` verifiziert. | `tests/model_fingerprint_test.rs` |

---

## 5. Zielzustand & Migrationspfad
Gemäß `docs/SPEC.md` §6.2:
1. Feature-Gating aller ML-Bibliotheken hinter `candle`.
2. Etablierung des lokalen `ConfidenceCalibrator`-Traits zur Entkopplung von `contextra-rank`.
3. Isolierung der Brücken (`embedding_provider.rs`, `kv_bridge.rs`) im Modul `compat`.
