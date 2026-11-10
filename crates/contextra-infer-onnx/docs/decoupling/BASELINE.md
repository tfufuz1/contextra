# BASELINE — contextra-infer-onnx (Vor Beginn der Änderungen)

## 1. Ausgangsbasis
- **Commit:** HEAD (9305c5d04)
- **Crate-Pfad:** `crates/contextra-infer-onnx/`
- **Datum:** 2026-11-10

## 2. Baseline Status der Tests & Builds
- `cargo check -p contextra-infer-onnx --no-default-features --offline`: GRÜN (7.65s)
- `cargo test -p contextra-infer-onnx --offline`: GRÜN (8 Tests: 2 Unit-Tests, 3 Adversarial-Tests, 1 Batch-Equiv, 2 Model-Security; 0 Fehler)
- `cargo clippy -p contextra-infer-onnx --all-targets --offline -- -D warnings`: GRÜN
- `tools/zero_panic_analyzer.py --crate contextra-infer-onnx`: 0 Prod-Verstöße
- `tools/determinism_guard.py --crate contextra-infer-onnx`: 0 Verstöße

## 3. Bestehende Features
- `default = []`
- `std = []`
- `serde = ["dep:serde", "dep:serde_json"]`
- `tracing = ["dep:tracing"]`
- `inspect = []`
- `experimental = []`
- `download = ["dep:reqwest"]`
- `reranking = ["onnx"]`
- `onnx = ["dep:ort", "dep:tokenizers", "dep:ndarray", "dep:sha2"]`

## 4. Nutzung der Fakes
- Außerhalb `crates/contextra-infer-onnx`: 0 Treffer
- Innerhalb: `examples/fake_reranker.rs`, `benches/infer_bench.rs`, `tests/batch_equiv.rs`, `tests/adversarial.rs`, `src/reranker/mod.rs` (`CrossEncoderReranker::Fake`)
