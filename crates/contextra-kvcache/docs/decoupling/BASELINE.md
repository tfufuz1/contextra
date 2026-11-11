# Baseline-Messung: contextra-kvcache (WP-KV-1.0)

Erfassungszeitpunkt: 2026-11-11
Branch: jules-5625781264283053937-7e8c9f5b
HEAD: ca103a3c7

## 1. Build & Check Baseline
- `cargo check -p contextra-kvcache --all-targets`: GRÜN (Dauer ca. 9.8s)
- `cargo check -p contextra-kvcache --no-default-features`: GRÜN (Warnungen bezüglich ungenutzter Imports / toter Funktionen im Zwischenstand)
- `cargo test -p contextra-kvcache`: GRÜN (41 Unit-Tests, 26 Integrationstest-Binaries, alle bestanden)
- Feature-Matrix mit `--no-default-features --features <f>`:
  - `std`: OK
  - `workspace`: OK
  - `ports-compat`: OK
  - `crypto-compat`: OK
  - `kv-encryption`: OK
  - `exp-kivi`: OK
  - `kivi-quantization`: OK
  - `kvcache-kivi-quant`: OK
  - `exp-content-addressed`: OK
  - `content-addressed-kv-cache`: OK

## 2. Clippy-Baseline
- `cargo clippy -p contextra-kvcache --test clm_pinned_base_preserved --no-deps -- -D warnings`:
  - Vorab-Befund: `indexing may panic` in Zeile 98 (`base_hit.blocks[0].data`).
  - Behebung: Test-Annotation `#![allow(clippy::indexing_slicing)]` im Test ergänzt; Assertion unverändert beibehalten.
  - Ergebnis nach Annotation: GRÜN.
- `cargo clippy -p contextra-kvcache --no-deps -- -D warnings`:
  - Lib-Check: GRÜN.

## 3. Golden-Test & Hashes
- Befehl: `cargo test -p contextra-kvcache --test golden_test --test golden_corruption_negative_tests`
  - Ergebnis: GRÜN (0 passed in corruption [Unit test empty], 1 passed in golden_test).
- Hashes aus `tests/golden/MANIFEST.sha256`:
  - `da6800457a2e3d8040315bd9db239522766a8506c07ca99cf342226a5ab0ba10  eviction_ranking.json`
  - `c306b506f4860cd3f2c53b5bab45e9f710f1ef188ad8cf8f3a756ff1f8a33951  kivi_dequantized.json`
  - `e74d8d91d870e48c4a4f4f38cc3a9eba27497c4ab7f582e29da7e4195fb148f9  kivi_meta.json`
  - `72d4770c116ff47163d061ed94ac3b00548c3a86c1044937755cc4ec28542821  radix_match.json`
  - `0f0c280a7b338be2c49ae7bd3db6972f4c61d16e90732886f2806fbf9c0a5fb8  kivi_packed.bin`

## 4. Determinismus & Zero-Panic Werkzeuge
- `python3 tools/determinism_guard.py --crate contextra-kvcache`:
  - Befunde Prod: 6 Stellen:
    - `src/store.rs`: Instant::now() in Zeilen 140, 208, 241
    - `src/eviction_worker.rs`: HashMap in Zeilen 23, 38, 46
  - Befunde Test: 16 Stellen
- `python3 tools/zero_panic_analyzer.py --crate contextra-kvcache`:
  - Prod-Verstöße: 0 (GRÜN)

## 5. Fremdnutzer-Inventar
- Befehl: `grep -rn "contextra-kvcache" --include=Cargo.toml crates benchmarks`
  - Treffer ausschließlich in `crates/contextra-kvcache/Cargo.toml:5`.
  - Keine Fremdnutzer im Workspace, die per Cargo direkt von `contextra-kvcache` abhängen.
