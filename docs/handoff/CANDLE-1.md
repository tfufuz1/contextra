# CANDLE-1 Übergabebericht: Autarke Candle-Inferenz (GGUF, Quantisierung, Embeddings)

## 1. Soll/Ist-Delta
- **Feature-Isolation:** `candle-*`, `tokenizers`, `tokio`, GPU-Backends (`cuda`, `metal`) sind optional hinter Feature-Flag `candle` gekapselt. Standard-Features: `std`, `serde`, `tracing`, `experimental`. `default = []`.
- **Vertikale Autarkie (Klasse A / B1-B12):** Alle internen Workspace-Abhängigkeiten (`contextra-types`, `contextra-ports`, `contextra-rank`, `contextra-crypto`) wurden aus `Cargo.toml` entfernt.
- **Lokale Traits & Typen:**
  - `Embedder` und `TextGenerator` synchron/asynchron lokal abstrahiert.
  - `KvStateSink` und `KvStateSource` lokal definiert ohne `contextra-crypto`/`contextra-kvcache` Kopplung.
  - Primärer Fehlertyp `Error` (`thiserror`, `#[non_exhaustive]`) und `pub type Result<T>`.
  - Fake-Backends `FakeEmbedder` und `FakeTextGenerator` für pure-Rust Builds ohne externe C-Libs/Candle.
- **Defensiver GGUF-Loader:**
  - Defensives Parsen gegen Puffer- und Dimensionsgrenzen (`MAX_GGUF_TENSORS`, `MAX_GGUF_METADATA_KVS`, `MAX_GGUF_STRING_LEN`).
  - SHA-256 Prüfsummenvalidierung vor dem Laden (`parse_gguf_metadata_with_checksum`).
  - Zero-Panic: Fehlerpfade liefern stets strukturierte `Error`-Varianten.
- **Adapter-Übergabe (E-05):**
  - Vorherige Bridges zu `contextra_ports` und `contextra_types` (`kv_bridge.rs`, `gasp.rs`, `inference/core.rs`, diverse Test-Suites) wurden verbatim unter `docs/handoff/CANDLE-1/adapter_src/` archiviert.
- **Determinismus (B11):**
  - Nicht-deterministische `HashMap` in `quantized_llama.rs` durch `BTreeMap` ersetzt. `determinism_guard.py` meldet 0 Verstöße.

## 2. Messwerte & Gates
- `zero_panic_analyzer.py --crate contextra-infer-candle --fail-on-prod`: 0 Verstöße.
- `decouple_inspector.py -c contextra-infer-candle`: Score 100/100 (völlig autark).
- `erasure_leak_prober.py -c contextra-infer-candle`: 0 Leaks.
- `concurrency_lock_auditor.py -c contextra-infer-candle`: 0 Lock-across-await Gefahren.
- `export_crate.py contextra-infer-candle`: Gate G3 bestanden (Export nach `/tmp/export-contextra-infer-candle`, `cargo test --locked` 100 % bestanden).

## 3. Offene Punkte / Upstream Blocker
- **`crates/contextra-vector/Cargo.toml`:** `ports-compat` bindet `dep:contextra-ports`, während `contextra-ports` non-optional ist. Dies blockiert Workspace-Metadaten in CI-Workflows (`check`, `proof-tests`).
- **`crates/contextra-infer-ollama/Cargo.toml:22`:** Doppelte Deklaration `[features]` führt zu Syntaxfehlern beim Manifest-Laden in CI (`context-gates`, `benchmarks`, `cargo-deny`).
- **Production Unwrap Baseline:** `Production unwrap count increased from 1197 to 1310` in fremden Crates auf `origin/main`.
- Gemäß Regel *„Nur das eigene Crate ändern“* verbleiben diese Upstream-Fehler unangetastet.
