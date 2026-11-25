# Architecture & Code Audit: `contextra-infer-onnx`

**Datum:** 2026-09-27
**Auditor:** Principal Senior Rust Architect
**Crate:** `contextra-infer-onnx` (`crates/contextra-infer-onnx/`)
**Scope:** `crates/contextra-infer-onnx/src/` (`lib.rs`, `reranker/mod.rs`, `reranker/config.rs`, `reranker/cross_encoder.rs`, `reranker/onnx.rs`)
**Status:** Experimental / In-Process Local Inference Engine

---

## 1. Übersicht & Architekturrolle

Die Crate `contextra-infer-onnx` stellt den in-process ONNX Inferenz-Treiber für Text-Embeddings und Cross-Encoder Reranking bereit (Layer 3 im Contextra 5-Schichten-DAG). Sie implementiert den `EmbeddingProvider` Trait aus `contextra-ports`.

### Modul-Struktur & Experimental-Status
* `src/lib.rs`: Enthält `TextEmbedder`, `TextEmbedderConfig`, Modell-Download-Manager (`ensure_onnx_model_download`) sowie `#![forbid(unsafe_code)]`.
* `src/reranker/`: Enthält `CrossEncoderReranker` (Unified Public API), `OnnxReranker` (ONNX Runtime Backend), `RerankConfig` und `PlattScaledSigmoid` Kalibrierung.
* **Hinweis zur Dateistruktur:** Das Session-Pooling (`SessionPool`) ist direkt in `TextEmbedder` und `OnnxReranker` mittels `Arc<parking_lot::Mutex<ort::session::Session>>` kapselt und erfordert keine eigene `pool.rs` Datei.

---

## 2. Vorbereitung & AGENTS.md Konformität

1. **Workspace & Precheck:**
   - Multi-Agent-Pfade wurden gemäß Single-Agent-Betriebsmodus verifiziert.
   - `crates/contextra-infer-onnx/AGENTS.md` Invarianten wurden geprüft.
2. **Import-Grenzen (DAG):**
   - Erlaubte Imports: `contextra-types`, `contextra-ports`, `contextra-rank` (Ring 0 / Layer 0).
   - Verbotene Imports: `contextra-db` (L2), `contextra-infer-ollama` (Peer). All verbotenen Imports sind abwesend.

---

## 3. Auditing der Prüfpunkte (P1 - P4)

### P1 FEATURE-GATE ISOLATION
* **Befehl:** `grep "^default\|onnx" crates/contextra-infer-onnx/Cargo.toml`
* **Ergebnis:**
  ```toml
  default = []
  onnx = ["dep:ort", "dep:tokenizers", "dep:ndarray", "dep:reqwest"]
  ```
* **Bewertung:** **BESTANDEN (PASS)**.
  Das Feature `onnx` ist **nicht** im Default-Set enthalten (`default = []`). Dies garantiert, dass Contextra im standardmäßigen Pure-Rust Build ohne schwerfällige C++ ONNX Runtime C-FFI Bindings kompilieren kann (gemäß Sovereign Core Doctrine / ADR-005).

---

### P2 SESSION-POOL & THREAD-SAFETY
* **Analyse:** Wie werden ONNX-Sessions zwischen Threads geteilt und synchronisiert?
  1. **Thread-Safety der ONNX Session:**
     `ort::session::Session` wird in `TextEmbedder` und `OnnxReranker` in ein `Arc<parking_lot::Mutex<ort::session::Session>>` verpackt.
  2. **Vermeidung von Tokio Executor Starvation:**
     ONNX Inference-Aufrufe (`session.run()`) sind blockierend und CPU-intensiv. Alle Inferenz-Operationen werden strikt in `tokio::task::spawn_blocking` ausgelagert.
  3. **Backpressure & Concurrency Control:**
     `TextEmbedder` schützt den Tokio-Blocking-Threadpool durch ein `tokio::sync::Semaphore` (`max_concurrent_embeddings`, Default: 8).
  4. **Protokoll-Invariante (`pool_size`):**
     `TextEmbedderConfig::validate()` erzwingt `pool_size = 1`. Mehrere parallele Threads teilen sich die gegenseitig ausschließende Mutex-Sperre innerhalb von `spawn_blocking`. `parking_lot::Mutex` garantiert Panic-Safety ohne Poisoning-Risiko.
* **Bewertung:** **BESTANDEN (PASS)**.

---

### P3 CROSS-ENCODER KORREKTHEIT & SCORE-NORMIERUNG
* **Analyse (`reranker.rs` / `OnnxReranker`):**
  1. **Top-k Exaktheit:**
     `CrossEncoderReranker` berechnet für jedes übergebene Paar `(query, document)` den Cross-Encoder Relevance Logit über ONNX. Es werden alle $N$ Kandidaten vollständig gescored (kein Approximations-Pruning) und absteigend nach Score sortiert (`results.sort_by(...)`). Ein Abgreifen der Top-$k$ Elemente aus dem Ergebnis liefert deterministisch und exakt dieselben Top-$k$ Dokumente wie ein Brute-Force Scoring.
  2. **Tensor-Dimensionen & Logit-Extraktion:**
     `extract_scores_from_tensor_calibrated` unterstützt:
     - 1D Tensors `[b_size]`: Direkte Logits.
     - 2D Tensors `[b_size, 1]`: Einreihige Logits.
     - 2D Tensors `[b_size, 2]`: Binary Classification Logits ($l_1 - l_0$, Differenz zwischen Relevanz- und Nicht-Relevanz-Klasse).
  3. **Score-Normierung & Kalibrierung:**
     Die Inferenz-Scores werden mittels `PlattScaler` kalibriert. Standardmäßig liefert `PlattScaler::identity()` die unkalibrierte Sigmoid-Transformation $\sigma(x) = \frac{1}{1 + e^{-x}} \in (0, 1)$. Online-Feedback via `record_outcome` / `record_implicit_feedback` passt die Platt-Parameter $A, B$ nach Warmup ($N \ge 50$) an und reduziert nachweislich die Expected Calibration Error (ECE).
* **Bewertung:** **BESTANDEN (PASS)**.

---

### P4 ZERO-PANIC IN PRODUKTIONSCODE
* **Befehl:** `grep -rn "\.unwrap()\|\.expect(" crates/contextra-infer-onnx/src/ | grep -v test`
* **Ergebnis:**
  * `reranker/onnx.rs:36`: Einzige Trefferstelle in einer Kommentarzeile (`// Poison-Mechanismus hat. Kein .unwrap()/.map_err() nötig.`).
  * In Nicht-Test-Produktionscode existieren **0** `.unwrap()` und **0** `.expect()` Aufrufe.
  * Alle Fehlermöglichkeiten (fehlende Modelldateien, Tokenisierungsfehler, ONNX Runtime Fehler, Dimension-Mismatches) werden sauber über `Result<T, ContextraError>` behandelt.
  * Das Crate erzwingt `#![forbid(unsafe_code)]`.
* **Bewertung:** **BESTANDEN (PASS)**.

---

## 4. Testergebnisse & Linter-Prüfung

### 1. `cargo test -p contextra-infer-onnx --locked`
```text
running 23 tests
test reranker::tests::test_calibration_extreme_logits_and_non_finite_inputs ... ok
test reranker::tests::test_calibration_fitted_platt_not_identity ... ok
test reranker::tests::test_calibration_is_calibrated_after_warmup ... ok
test reranker::tests::test_concurrent_rerank_load_no_panic ... ok
test reranker::tests::test_cross_encoder_online_calibration_and_invalidation ... ok
test reranker::tests::test_implicit_feedback_passthrough_skipped ... ok
test reranker::tests::test_platt_scaled_sigmoid_config_and_reranker_builder ... ok
test reranker::tests::test_platt_scaled_sigmoid_fit_noisy_unseparable_and_nan ... ok
test reranker::tests::test_platt_scaled_sigmoid_fit_separable_dataset ... ok
test reranker::tests::test_platt_scaled_sigmoid_identity_matches_uncalibrated_sigmoid ... ok
test reranker::tests::test_record_implicit_feedback_top_k_marked_relevant ... ok
test reranker::tests::test_rerank_empty_candidates ... ok
test reranker::tests::test_rerank_exact_boundary_candidates ... ok
test reranker::tests::test_rerank_oversized_candidate_batch_rejected ... ok
test reranker::tests::test_rerank_passthrough_preserves_order ... ok
test reranker::tests::test_rerank_sorted_by_score_descending ... ok
test tests::test_default_model_cache_dir_resolution ... ok
test tests::test_embed_batch_ordering_and_fallback ... ok
test tests::test_ensure_onnx_model_download_disabled_feature ... ok
test tests::test_formatting_safety ... ok
test tests::test_mock_embedding_engine ... ok
test reranker::tests::test_ece_reduction_after_platt_calibration ... ok
test reranker::tests::prop_platt_calibration_reduces_ece ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

### 2. `cargo clippy -p contextra-infer-onnx --no-deps --all-targets -- -D warnings`
* **Ergebnis:** 0 Fehler, 0 Warnungen.

---

## 5. Fazit & Architektur-Empfehlungen

1. **Freigabe-Empfehlung:**
   Die Crate `contextra-infer-onnx` erfüllt alle Sicherheits-, Concurrency-, Thread-Safety- und Panic-Free Invarianten von Contextra. Sie ist bereit für den Experimental-Einsatz mit optional aktiviertem Feature-Flag `onnx`.
2. **Pflege-Hinweis:**
   Die Dokumentation in `AGENTS.md` sollte beibehalten werden, um zukünftigen Entwicklern zu veranschaulichen, dass Inferenz-Aufrufe stets hinter `spawn_blocking` gekapselt bleiben müssen.
