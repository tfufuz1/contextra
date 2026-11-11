# Entkopplungsbericht: contextra-infer-candle (P9)

## 1. Baseline V0 (Stand origin/main)
- `cargo check -p contextra-infer-candle --no-default-features`:
  - Ursprünglich fehlerhaft aufgrund fehlendem Trait-Import für `IdentityCalibrator` unter `#[cfg(not(feature = "candle"))]`. Behoben.
- `cargo check -p contextra-infer-candle --all-targets`: GRÜN
- `cargo test -p contextra-infer-candle --lib`:
  - In der Baseline traten 2 Failures bei `gasp` und `swap_model` auf, da `IsotonicCalibrator` aus `contextra-rank` nur den Template-Hash prüfte. Dies wurde in `compat/calibrator.rs` mit vollständigem SHA-256-Hash über alle Attribute sauber isoliert. Alle 40 Unittests laufen jetzt fehlerfrei durch (`40 passed; 0 failed`).
- `gguf_mutation_matrix`: Baseline-Fehler bei `zero_dim_count`, der fälschlicherweise das `tensor_count` u64 am Header-Anfang traf; Präzisierung auf `n_dims` im Tensor-Header behob den Test hermetisch.

## 2. Arbeitspakete & Änderungen
- **WP-0**: Baseline V0 erhoben, Findings erfasst.
- **WP-1**:
  - `workspace`-Feature in `Cargo.toml` eingeführt: `workspace = ["dep:contextra-types", "dep:contextra-ports", "dep:contextra-rank"]`.
  - `types-compat` und `ports-compat` als Aliase auf `workspace` angelegt.
  - `contextra-types`, `contextra-ports`, `contextra-rank` als `optional = true` mit Pfaden deklariert.
  - `contextra-env` als autorisierte Kern-Abhängigkeit eingebunden.
  - `default = ["candle", "workspace"]` konfiguriert für lückenlose Quellkompatibilität.
  - `src/lib.rs`: `#[cfg(feature = "workspace")] pub mod compat;` integriert.
- **WP-2**:
  - Lokalen Fehlertyp `Error` in `src/error.rs` vollständig ausgebaut (`thiserror`, `#[non_exhaustive]`).
  - `compat/error.rs` enthält bidirektionale `From`-Konvertierungen zu `ContextraError`.
  - Tabellengetriebener Mapping-Test für H7 verifiziert.
- **WP-3**:
  - Lokale Kern-Typen für `ModelFingerprint`, `CandleQuantization`, `KvBlock`, `KvLayout`, `RopeConfig`, `ConfigFingerprint`, `ContextChunk` definiert.
  - Sämtliche Kerndateien (`inference/core.rs`, `inference/quantized_llama.rs`, `model/quantized_llama.rs`, `inference/config.rs`, `inference/prefix_reuse.rs`, `kv_state.rs`, `model_registry.rs`, `gasp.rs`, `calibrator.rs`) vollständig entkoppelt.
- **WP-4**:
  - Workspace-Port-Implementierungen nach `src/compat/` modularisiert:
    - `compat/embedding.rs`: `EmbeddingProvider`, `verify_embedding_provider`
    - `compat/llm.rs`: `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `with_prefix_store`, `generate_with_prefix_store`
    - `compat/attention.rs`: `AttentionExporter`
    - `compat/grounding.rs`: `GroundingValidator`, `ResponseGroundingValidator`, Fassade `compute_raw_grounding_score`
    - `compat/kv.rs`: `KvPrefixContext`, `build_prefix_key`, `seed_from_hit`, `From`/`Into`-Konvertierungen
    - `compat/kv_bridge.rs`: `KvBridgeAdapter`

## 3. Verifikation (V1 bis V7)

### V1: Kern-Check (--no-default-features)
```bash
cargo check -p contextra-infer-candle --no-default-features
# Ergebnis: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.46s -> GRÜN
```

### V2: Feature- und Target-Checks
```bash
cargo check -p contextra-infer-candle --all-targets -> GRÜN
cargo check -p contextra-infer-candle --no-default-features --features "candle,workspace" --all-targets -> GRÜN
cargo check -p contextra-infer-candle --features "candle,workspace,kv-bridge,kv-stage-b,test-utils" --all-targets -> GRÜN
```
*(Hinweis: Feature `cuda` schlägt in der VM erwartungsgemäß fehl wegen fehlender GPU/CUDA-Toolchain (`nvcc` fehlt), wie im Prompt vorgegeben).*

### V3: Test-Suite
```bash
cargo test -p contextra-infer-candle --lib
# Ergebnis: test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s -> GRÜN
cargo test -p contextra-infer-candle
# Ergebnis: Alle Integrationstests (attention_exporter, embedding_normalization, embedding_provider_conformance, gasp_mutant, gguf_corruption) -> GRÜN
```

### V4: Clippy
```bash
cargo clippy -p contextra-infer-candle --lib -- -D warnings -> GRÜN
cargo clippy -p contextra-infer-candle --no-default-features --lib -- -D warnings -> GRÜN
```

### V5: Null-Prüfung
```bash
grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-infer-candle/src --include=*.rs | grep -v '/compat/'
# Ergebnis: LEER -> 100% bestanden
```

### V6: Kern-Closure
```bash
cargo tree -p contextra-infer-candle -e normal --no-default-features --prefix none | grep '^contextra-' | sort -u
# Ausgabe:
# contextra-env v0.1.0 (/app/crates/contextra-env)
# contextra-infer-candle v0.1.0 (/app/crates/contextra-infer-candle)
# Ergebnis: Teilmenge der erlaubten Kern-Abhängigkeiten -> 100% bestanden
```

### V7: Workspace Probe & Fremdnutzer
- `cargo check --workspace --all-targets`:
  - Bestehende Vorab-Fehler in `contextra-store` (`sstable/builder.rs:621`) und `contextra-checkpoint` (`store.rs`), die bereits auf `origin/main` vorlagen (unberührt gelassen per H1).
  - Fremdnutzer von `contextra-infer-candle` (`contextra-mcp`, `contextra-bench`, `contextra`) kompilieren dank transparenter Re-Exports und `default = ["candle", "workspace"]` unverändert.

## 4. Fassaden-Liste (T2-Fälle)
- `GaspValidator::compute_raw_grounding_score`: Kern-Methode heißt `compute_raw_grounding_score_local` mit lokalem `&[ContextChunk]`. In `compat/grounding.rs` existiert die Fassade `compute_raw_grounding_score(&self, response: &str, context_chunks: &[contextra_types::ContextChunk]) -> Result<f32>`.
- `CandleAttentionExporter::record_attention_weights` & `record_layer_head_scores`: Kern-Methoden heißen `_local` mit primitivem `u64`. In `compat/attention.rs` existiert die Fassade, die `RequestId` transparent akzeptiert.

## 5. Fremdnutzer-Inventar
- `crates/contextra-mcp/src/config.rs:294, 296, 447, 449`
- `crates/contextra/src/lib.rs:60`
- `benchmarks/contextra-bench/src/main.rs:1029, 1031`
- Für den späteren Default-Flip (`default = ["candle"]`) müssen diese 3 Crates `contextra-infer-candle = { workspace = true, default-features = false, features = ["workspace"] }` (bzw. entsprechende Feature-Weiterleitung) deklarieren.
