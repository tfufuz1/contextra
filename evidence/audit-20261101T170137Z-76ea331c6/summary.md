# Contextra Voll-Audit

- Commit: `76ea331c6` (2026-11-01 18:01:29 +0100), Branch `main`, hinter origin: 0, ungespeicherte Aenderungen: False
- Start: 2026-11-01T17:01:37+00:00  Dauer: 96.8 min
- Toolchain-Pin: `1.89.0`; Preflight: **PASS** (cargo 1.89.0 (c24e10642 2025-06-23) | Pin 1.89.0)
- Host: Linux-7.2.7-1-liquorix-amd64-x86_64-with-glibc2.41 | 8 CPUs | MemTotal:       10121020 kB
- Nutzer: freddy

## Gesamt

- **FAIL**: 226
- **SKIPPED**: 19
- **INFO**: 6
- **PASS**: 69

Hinweis: Bei `befund_*`-Schritten bedeutet FAIL, dass der Befund **vorliegt**. `INFO` ist eine Messung oder ein Hinweis ohne Urteil.

## Rote und uebersprungene Schritte

| Schritt | Status | Sek | Detail | Log |
|---|---|---:|---|---|
| `befund_cfg_features_undefiniert` | FAIL | 0.4 | 1 Crates mit undefinierten Features | `logs/000_befund_cfg_features_undefiniert.log` |
| `befund_safe_fn_mit_unsafe_inhalt` | FAIL | 0.0 | 2 bekannte K7/K8-Stellen (mmap_readonly, reopen_and_dup2) | gesamt 13 Kandidaten | `logs/002_befund_safe_fn_mit_unsafe_inhalt.log` |
| `befund_ingest_verdrahtung` | FAIL | 0.0 | kein Workspace-Crate haengt von contextra-ingest ab; ingest haengt direkt an contextra-engine (Produktmenge darf den Kern nicht ziehen) | `logs/003_befund_ingest_verdrahtung.log` |
| `befund_default_members` | FAIL | 0.0 | members=37 default-members=27; Produkt nicht Mitglied: []; Produkt nicht in default-members: ['contextra-ingest', 'contextra-privacy', 'contextra-audit-export', 'contextra-avv-generator'] | `logs/004_befund_default_members.log` |
| `befund_from_entropy` | FAIL | 0.0 | 4 Produktivdateien mit from_entropy (K14) | `logs/006_befund_from_entropy.log` |
| `produkt_huelle_ohne_kern` | FAIL | 3.5 | 1 Produkt-Crates mit Befund | `logs/010_produkt_huelle_ohne_kern.log` |
| `fmt_check` | FAIL | 7.9 |  | `logs/017_fmt_check.log` |
| `check_alle_targets` | FAIL | 145.8 | errors=26 | `logs/018_check_alle_targets.log` |
| `doc` | FAIL | 11.9 |  | `logs/020_doc.log` |
| `clippy_lib__contextra` | FAIL | 32.0 | errors=303 warnings=1 | `logs/021_clippy_lib__contextra.log` |
| `clippy_alle_targets__contextra` | FAIL | 7.3 | errors=190 warnings=1 | `logs/022_clippy_alle_targets__contextra.log` |
| `clippy_lib__contextra-adapt` | FAIL | 11.1 | errors=190 warnings=0 | `logs/023_clippy_lib__contextra-adapt.log` |
| `clippy_alle_targets__contextra-adapt` | FAIL | 8.3 | errors=260 warnings=1 | `logs/024_clippy_alle_targets__contextra-adapt.log` |
| `clippy_lib__contextra-agent` | FAIL | 4.8 | errors=303 warnings=1 | `logs/025_clippy_lib__contextra-agent.log` |
| `clippy_alle_targets__contextra-agent` | FAIL | 5.5 | errors=303 warnings=1 | `logs/026_clippy_alle_targets__contextra-agent.log` |
| `clippy_lib__contextra-bench` | FAIL | 16.3 | errors=305 warnings=1 | `logs/033_clippy_lib__contextra-bench.log` |
| `clippy_alle_targets__contextra-bench` | FAIL | 7.6 | errors=305 warnings=1 | `logs/034_clippy_alle_targets__contextra-bench.log` |
| `clippy_alle_targets__contextra-checkpoint` | FAIL | 21.2 | errors=49 warnings=0 | `logs/036_clippy_alle_targets__contextra-checkpoint.log` |
| `clippy_lib__contextra-cognition` | FAIL | 3.5 | errors=303 warnings=1 | `logs/037_clippy_lib__contextra-cognition.log` |
| `clippy_alle_targets__contextra-cognition` | FAIL | 12.8 | errors=303 warnings=1 | `logs/038_clippy_alle_targets__contextra-cognition.log` |
| `clippy_lib__contextra-core` | FAIL | 1.2 | errors=2 warnings=0 | `logs/039_clippy_lib__contextra-core.log` |
| `clippy_alle_targets__contextra-core` | FAIL | 1.4 | errors=3 warnings=1 | `logs/040_clippy_alle_targets__contextra-core.log` |
| `clippy_lib__contextra-crypto` | FAIL | 7.1 | errors=49 warnings=0 | `logs/041_clippy_lib__contextra-crypto.log` |
| `clippy_alle_targets__contextra-crypto` | FAIL | 10.0 | errors=49 warnings=0 | `logs/042_clippy_alle_targets__contextra-crypto.log` |
| `clippy_lib__contextra-db` | FAIL | 3.9 | errors=303 warnings=1 | `logs/043_clippy_lib__contextra-db.log` |
| `clippy_alle_targets__contextra-db` | FAIL | 11.0 | errors=303 warnings=1 | `logs/044_clippy_alle_targets__contextra-db.log` |
| `clippy_lib__contextra-durable-fs` | FAIL | 1.1 | errors=9 warnings=0 | `logs/045_clippy_lib__contextra-durable-fs.log` |
| `clippy_alle_targets__contextra-durable-fs` | FAIL | 0.9 | errors=10 warnings=1 | `logs/046_clippy_alle_targets__contextra-durable-fs.log` |
| `clippy_lib__contextra-engine` | FAIL | 3.7 | errors=303 warnings=1 | `logs/047_clippy_lib__contextra-engine.log` |
| `clippy_alle_targets__contextra-engine` | FAIL | 13.1 | errors=303 warnings=1 | `logs/048_clippy_alle_targets__contextra-engine.log` |
| `clippy_lib__contextra-graph` | FAIL | 2.2 | errors=190 warnings=0 | `logs/049_clippy_lib__contextra-graph.log` |
| `clippy_alle_targets__contextra-graph` | FAIL | 5.1 | errors=239 warnings=1 | `logs/050_clippy_alle_targets__contextra-graph.log` |
| `clippy_lib__contextra-infer-candle` | FAIL | 24.7 | errors=6 warnings=0 | `logs/051_clippy_lib__contextra-infer-candle.log` |
| `clippy_alle_targets__contextra-infer-candle` | FAIL | 25.2 | errors=49 warnings=1 | `logs/052_clippy_alle_targets__contextra-infer-candle.log` |
| `clippy_alle_targets__contextra-infer-ollama` | FAIL | 12.5 | errors=39 warnings=1 | `logs/054_clippy_alle_targets__contextra-infer-ollama.log` |
| `clippy_alle_targets__contextra-infer-onnx` | FAIL | 21.3 | errors=20 warnings=0 | `logs/056_clippy_alle_targets__contextra-infer-onnx.log` |
| `clippy_lib__contextra-ingest` | FAIL | 4.0 | errors=303 warnings=1 | `logs/057_clippy_lib__contextra-ingest.log` |
| `clippy_alle_targets__contextra-ingest` | FAIL | 21.0 | errors=303 warnings=1 | `logs/058_clippy_alle_targets__contextra-ingest.log` |
| `clippy_lib__contextra-kvcache` | FAIL | 2.9 | errors=49 warnings=0 | `logs/059_clippy_lib__contextra-kvcache.log` |
| `clippy_alle_targets__contextra-kvcache` | FAIL | 7.2 | errors=49 warnings=0 | `logs/060_clippy_alle_targets__contextra-kvcache.log` |
| `clippy_lib__contextra-mcp` | FAIL | 5.8 | errors=303 warnings=1 | `logs/063_clippy_lib__contextra-mcp.log` |
| `clippy_alle_targets__contextra-mcp` | FAIL | 3.6 | errors=303 warnings=1 | `logs/064_clippy_alle_targets__contextra-mcp.log` |
| `clippy_alle_targets__contextra-privacy` | FAIL | 4.9 | errors=49 warnings=0 | `logs/070_clippy_alle_targets__contextra-privacy.log` |
| `clippy_lib__contextra-py` | FAIL | 4.4 | errors=303 warnings=1 | `logs/071_clippy_lib__contextra-py.log` |
| `clippy_alle_targets__contextra-py` | FAIL | 8.0 | errors=303 warnings=1 | `logs/072_clippy_alle_targets__contextra-py.log` |
| `clippy_alle_targets__contextra-rank` | FAIL | 3.0 | errors=190 warnings=1 | `logs/074_clippy_alle_targets__contextra-rank.log` |
| `clippy_lib__contextra-router` | FAIL | 1.9 | errors=190 warnings=0 | `logs/075_clippy_lib__contextra-router.log` |
| `clippy_alle_targets__contextra-router` | FAIL | 9.2 | errors=303 warnings=1 | `logs/076_clippy_alle_targets__contextra-router.log` |
| `clippy_lib__contextra-store` | FAIL | 6.7 | errors=49 warnings=0 | `logs/081_clippy_lib__contextra-store.log` |
| `clippy_alle_targets__contextra-store` | FAIL | 21.7 | errors=239 warnings=1 | `logs/082_clippy_alle_targets__contextra-store.log` |
| `clippy_lib__contextra-text` | FAIL | 7.1 | errors=64 warnings=0 | `logs/087_clippy_lib__contextra-text.log` |
| `clippy_alle_targets__contextra-text` | FAIL | 8.2 | errors=188 warnings=1 | `logs/088_clippy_alle_targets__contextra-text.log` |
| `clippy_lib__contextra-vector` | FAIL | 3.1 | errors=49 warnings=0 | `logs/091_clippy_lib__contextra-vector.log` |
| `clippy_alle_targets__contextra-vector` | FAIL | 4.7 | errors=49 warnings=0 | `logs/092_clippy_alle_targets__contextra-vector.log` |
| `feature_matrix_hack` | FAIL | 1062.1 |  | `logs/095_feature_matrix_hack.log` |
| `deny` | FAIL | 3.3 |  | `logs/096_deny.log` |
| `audit` | FAIL | 4.9 |  | `logs/097_audit.log` |
| `panic_flaeche_lib` | FAIL | 37.3 | errors=2 warnings=625 | `logs/098_panic_flaeche_lib.log` |
| `test_lib__contextra` | FAIL | 81.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/100_test_lib__contextra.log` |
| `test_integration__contextra` | FAIL | 2.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/101_test_integration__contextra.log` |
| `nextest__contextra` | FAIL | 51.5 | nextest: keine Summary-Zeile gefunden | `logs/103_nextest__contextra.log` |
| `nextest__contextra-agent` | FAIL | 120.8 | passed=120 failed=1 | FAILED: ( 89/121) contextra-agent context::tests::test_telemetry_100k_insertions_and_amortized_performance | `logs/111_nextest__contextra-agent.log` |
| `test_integration__contextra-arch-tests` | FAIL | 2.3 | passed=3 failed=2 ignored=0 | FAILED: ring_regeln_gelten, unsafe_nur_in_inseln | `logs/113_test_integration__contextra-arch-tests.log` |
| `nextest__contextra-arch-tests` | FAIL | 1.8 | passed=3 failed=2 | FAILED: (4/5) contextra-arch-tests::arch ring_regeln_gelten, (5/5) contextra-arch-tests::arch unsafe_nur_in_inseln | `logs/115_nextest__contextra-arch-tests.log` |
| `test_integration__contextra-audit-export` | FAIL | 4.0 | passed=31 failed=1 ignored=0 | FAILED: test_h10_gdpr_art30_processing_registry_and_canary_leak_check | `logs/117_test_integration__contextra-audit-export.log` |
| `nextest__contextra-audit-export` | FAIL | 1.1 | passed=31 failed=1 | FAILED: (21/32) contextra-audit-export::campaign_j27_compliance_test test_h10_gdpr_art30_processing_registry_and_canary_leak_check | `logs/119_nextest__contextra-audit-export.log` |
| `test_integration__contextra-crypto` | FAIL | 2316.4 | passed=304 failed=2 ignored=0 | FAILED: test_scenario_3_deletion_proof_v3_creation_and_exhaustive_mutation_test, test_scenario_4_independent_external_verification_and_tempfile | `logs/141_test_integration__contextra-crypto.log` |
| `test_doc__contextra-crypto` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/142_test_doc__contextra-crypto.log` |
| `nextest__contextra-crypto` | FAIL | 0.3 | nextest: keine Summary-Zeile gefunden | `logs/143_nextest__contextra-crypto.log` |
| `test_lib__contextra-db` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/144_test_lib__contextra-db.log` |
| `test_integration__contextra-db` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/145_test_integration__contextra-db.log` |
| `test_doc__contextra-db` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/146_test_doc__contextra-db.log` |
| `nextest__contextra-db` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/147_nextest__contextra-db.log` |
| `test_lib__contextra-durable-fs` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/148_test_lib__contextra-durable-fs.log` |
| `test_integration__contextra-durable-fs` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/149_test_integration__contextra-durable-fs.log` |
| `test_doc__contextra-durable-fs` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/150_test_doc__contextra-durable-fs.log` |
| `nextest__contextra-durable-fs` | FAIL | 0.3 | nextest: keine Summary-Zeile gefunden | `logs/151_nextest__contextra-durable-fs.log` |
| `test_lib__contextra-engine` | FAIL | 0.2 | passed=0 failed=0 ignored=0 | `logs/152_test_lib__contextra-engine.log` |
| `test_integration__contextra-engine` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/153_test_integration__contextra-engine.log` |
| `test_doc__contextra-engine` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/154_test_doc__contextra-engine.log` |
| `nextest__contextra-engine` | FAIL | 0.3 | nextest: keine Summary-Zeile gefunden | `logs/155_nextest__contextra-engine.log` |
| `test_lib__contextra-graph` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/156_test_lib__contextra-graph.log` |
| `test_integration__contextra-graph` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/157_test_integration__contextra-graph.log` |
| `test_doc__contextra-graph` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/158_test_doc__contextra-graph.log` |
| `nextest__contextra-graph` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/159_nextest__contextra-graph.log` |
| `test_lib__contextra-infer-candle` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/160_test_lib__contextra-infer-candle.log` |
| `test_integration__contextra-infer-candle` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/161_test_integration__contextra-infer-candle.log` |
| `test_doc__contextra-infer-candle` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/162_test_doc__contextra-infer-candle.log` |
| `nextest__contextra-infer-candle` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/163_nextest__contextra-infer-candle.log` |
| `test_lib__contextra-infer-ollama` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/164_test_lib__contextra-infer-ollama.log` |
| `test_integration__contextra-infer-ollama` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/165_test_integration__contextra-infer-ollama.log` |
| `test_doc__contextra-infer-ollama` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/166_test_doc__contextra-infer-ollama.log` |
| `nextest__contextra-infer-ollama` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/167_nextest__contextra-infer-ollama.log` |
| `test_lib__contextra-infer-onnx` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/168_test_lib__contextra-infer-onnx.log` |
| `test_integration__contextra-infer-onnx` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/169_test_integration__contextra-infer-onnx.log` |
| `test_doc__contextra-infer-onnx` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/170_test_doc__contextra-infer-onnx.log` |
| `nextest__contextra-infer-onnx` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/171_nextest__contextra-infer-onnx.log` |
| `test_lib__contextra-ingest` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/172_test_lib__contextra-ingest.log` |
| `test_integration__contextra-ingest` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/173_test_integration__contextra-ingest.log` |
| `test_doc__contextra-ingest` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/174_test_doc__contextra-ingest.log` |
| `nextest__contextra-ingest` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/175_nextest__contextra-ingest.log` |
| `test_lib__contextra-kvcache` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/176_test_lib__contextra-kvcache.log` |
| `test_integration__contextra-kvcache` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/177_test_integration__contextra-kvcache.log` |
| `test_doc__contextra-kvcache` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/178_test_doc__contextra-kvcache.log` |
| `nextest__contextra-kvcache` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/179_nextest__contextra-kvcache.log` |
| `test_lib__contextra-license` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/180_test_lib__contextra-license.log` |
| `test_integration__contextra-license` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/181_test_integration__contextra-license.log` |
| `test_doc__contextra-license` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/182_test_doc__contextra-license.log` |
| `nextest__contextra-license` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/183_nextest__contextra-license.log` |
| `test_lib__contextra-mcp` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/184_test_lib__contextra-mcp.log` |
| `test_integration__contextra-mcp` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/185_test_integration__contextra-mcp.log` |
| `test_doc__contextra-mcp` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/186_test_doc__contextra-mcp.log` |
| `nextest__contextra-mcp` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/187_nextest__contextra-mcp.log` |
| `test_lib__contextra-mvcc` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/188_test_lib__contextra-mvcc.log` |
| `test_integration__contextra-mvcc` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/189_test_integration__contextra-mvcc.log` |
| `test_doc__contextra-mvcc` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/190_test_doc__contextra-mvcc.log` |
| `nextest__contextra-mvcc` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/191_nextest__contextra-mvcc.log` |
| `test_lib__contextra-ports` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/192_test_lib__contextra-ports.log` |
| `test_integration__contextra-ports` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/193_test_integration__contextra-ports.log` |
| `test_doc__contextra-ports` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/194_test_doc__contextra-ports.log` |
| `nextest__contextra-ports` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/195_nextest__contextra-ports.log` |
| `test_lib__contextra-privacy` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/196_test_lib__contextra-privacy.log` |
| `test_integration__contextra-privacy` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/197_test_integration__contextra-privacy.log` |
| `test_doc__contextra-privacy` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/198_test_doc__contextra-privacy.log` |
| `nextest__contextra-privacy` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/199_nextest__contextra-privacy.log` |
| `test_lib__contextra-py` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/200_test_lib__contextra-py.log` |
| `test_integration__contextra-py` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/201_test_integration__contextra-py.log` |
| `test_doc__contextra-py` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/202_test_doc__contextra-py.log` |
| `nextest__contextra-py` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/203_nextest__contextra-py.log` |
| `test_lib__contextra-rank` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/204_test_lib__contextra-rank.log` |
| `test_integration__contextra-rank` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/205_test_integration__contextra-rank.log` |
| `test_doc__contextra-rank` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/206_test_doc__contextra-rank.log` |
| `nextest__contextra-rank` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/207_nextest__contextra-rank.log` |
| `test_lib__contextra-router` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/208_test_lib__contextra-router.log` |
| `test_integration__contextra-router` | FAIL | 0.4 | passed=0 failed=0 ignored=0 | `logs/209_test_integration__contextra-router.log` |
| `test_doc__contextra-router` | FAIL | 0.5 | passed=0 failed=0 ignored=0 | `logs/210_test_doc__contextra-router.log` |
| `nextest__contextra-router` | FAIL | 1.0 | nextest: keine Summary-Zeile gefunden | `logs/211_nextest__contextra-router.log` |
| `test_lib__contextra-sandbox` | FAIL | 0.6 | passed=0 failed=0 ignored=0 | `logs/212_test_lib__contextra-sandbox.log` |
| `test_integration__contextra-sandbox` | FAIL | 0.4 | passed=0 failed=0 ignored=0 | `logs/213_test_integration__contextra-sandbox.log` |
| `test_doc__contextra-sandbox` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/214_test_doc__contextra-sandbox.log` |
| `nextest__contextra-sandbox` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/215_nextest__contextra-sandbox.log` |
| `test_lib__contextra-simd` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/216_test_lib__contextra-simd.log` |
| `test_integration__contextra-simd` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/217_test_integration__contextra-simd.log` |
| `test_doc__contextra-simd` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/218_test_doc__contextra-simd.log` |
| `nextest__contextra-simd` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/219_nextest__contextra-simd.log` |
| `test_lib__contextra-store` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/220_test_lib__contextra-store.log` |
| `test_integration__contextra-store` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/221_test_integration__contextra-store.log` |
| `test_doc__contextra-store` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/222_test_doc__contextra-store.log` |
| `nextest__contextra-store` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/223_nextest__contextra-store.log` |
| `test_lib__contextra-sys` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/224_test_lib__contextra-sys.log` |
| `test_integration__contextra-sys` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/225_test_integration__contextra-sys.log` |
| `test_doc__contextra-sys` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/226_test_doc__contextra-sys.log` |
| `nextest__contextra-sys` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/227_nextest__contextra-sys.log` |
| `test_lib__contextra-testkit` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/228_test_lib__contextra-testkit.log` |
| `test_integration__contextra-testkit` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/229_test_integration__contextra-testkit.log` |
| `test_doc__contextra-testkit` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/230_test_doc__contextra-testkit.log` |
| `nextest__contextra-testkit` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/231_nextest__contextra-testkit.log` |
| `test_lib__contextra-text` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/232_test_lib__contextra-text.log` |
| `test_integration__contextra-text` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/233_test_integration__contextra-text.log` |
| `test_doc__contextra-text` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/234_test_doc__contextra-text.log` |
| `nextest__contextra-text` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/235_nextest__contextra-text.log` |
| `test_lib__contextra-types` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/236_test_lib__contextra-types.log` |
| `test_integration__contextra-types` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/237_test_integration__contextra-types.log` |
| `test_doc__contextra-types` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/238_test_doc__contextra-types.log` |
| `nextest__contextra-types` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/239_nextest__contextra-types.log` |
| `test_lib__contextra-vector` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/240_test_lib__contextra-vector.log` |
| `test_integration__contextra-vector` | FAIL | 0.2 | passed=0 failed=0 ignored=0 | `logs/241_test_integration__contextra-vector.log` |
| `test_doc__contextra-vector` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/242_test_doc__contextra-vector.log` |
| `nextest__contextra-vector` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/243_nextest__contextra-vector.log` |
| `test_lib__contextra-wire` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/244_test_lib__contextra-wire.log` |
| `test_integration__contextra-wire` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/245_test_integration__contextra-wire.log` |
| `test_doc__contextra-wire` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/246_test_doc__contextra-wire.log` |
| `nextest__contextra-wire` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden | `logs/247_nextest__contextra-wire.log` |
| `test_gesamt_debug` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/248_test_gesamt_debug.log` |
| `test_gesamt_release` | FAIL | 0.1 | passed=0 failed=0 ignored=0 | `logs/249_test_gesamt_release.log` |
| `python_bindings_pytest` | FAIL | 0.0 |  | `logs/250_python_bindings_pytest.log` |
| `bench__contextra-mvcc__mvcc_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/251_bench__contextra-mvcc__mvcc_bench.log` |
| `bench__contextra-checkpoint__checkpoint_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/252_bench__contextra-checkpoint__checkpoint_bench.log` |
| `bench__contextra-router__router_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/253_bench__contextra-router__router_bench.log` |
| `bench__contextra-router__bandit_latency_gate` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/254_bench__contextra-router__bandit_latency_gate.log` |
| `bench__contextra-router__cost_routing_savings_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/255_bench__contextra-router__cost_routing_savings_bench.log` |
| `bench__contextra-graph__csr_traversal_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/256_bench__contextra-graph__csr_traversal_bench.log` |
| `bench__contextra-graph__binary_edge_regression` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/257_bench__contextra-graph__binary_edge_regression.log` |
| `bench__contextra-crypto__crypto_benchmarks` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/258_bench__contextra-crypto__crypto_benchmarks.log` |
| `bench__contextra-crypto__deletion_proof_latency_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/259_bench__contextra-crypto__deletion_proof_latency_bench.log` |
| `bench__contextra-infer-candle__kv_bridge_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/260_bench__contextra-infer-candle__kv_bridge_bench.log` |
| `bench__contextra-vector__distance_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/261_bench__contextra-vector__distance_bench.log` |
| `bench__contextra-vector__sq8_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/262_bench__contextra-vector__sq8_bench.log` |
| `bench__contextra-vector__hnsw_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/263_bench__contextra-vector__hnsw_bench.log` |
| `bench__contextra-vector__hnsw_delete_search_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/264_bench__contextra-vector__hnsw_delete_search_bench.log` |
| `bench__contextra-vector__audit_benchmarks` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/265_bench__contextra-vector__audit_benchmarks.log` |
| `bench__contextra-vector__flush_threshold_amplification` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/266_bench__contextra-vector__flush_threshold_amplification.log` |
| `bench__contextra-vector__partial_rebuild_recall_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/267_bench__contextra-vector__partial_rebuild_recall_bench.log` |
| `bench__contextra-vector__distance_nan_impact_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/268_bench__contextra-vector__distance_nan_impact_bench.log` |
| `bench__contextra-vector__filtered_search_selectivity_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/269_bench__contextra-vector__filtered_search_selectivity_bench.log` |
| `bench__contextra-store__sstable_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/270_bench__contextra-store__sstable_bench.log` |
| `bench__contextra-store__memtable_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/271_bench__contextra-store__memtable_bench.log` |
| `bench__contextra-store__wal_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/272_bench__contextra-store__wal_bench.log` |
| `bench__contextra-store__lsm_concurrency_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/273_bench__contextra-store__lsm_concurrency_bench.log` |
| `bench__contextra-store__block_cache_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/274_bench__contextra-store__block_cache_bench.log` |
| `bench__contextra-store__redb_comparative_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/275_bench__contextra-store__redb_comparative_bench.log` |
| `bench__contextra-simd__simd_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/276_bench__contextra-simd__simd_bench.log` |
| `bench__contextra-bench__competitive_kv` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/277_bench__contextra-bench__competitive_kv.log` |
| `bench__contextra-bench__locomo` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/278_bench__contextra-bench__locomo.log` |
| `bench__contextra-bench__long_mem_eval` | FAIL | 0.1 | keine Criterion-Zeilen erkannt | `logs/279_bench__contextra-bench__long_mem_eval.log` |
| `fuzz__text__fuzz_bm25_tokenize` | FAIL | 0.3 |  | `logs/296_fuzz__text__fuzz_bm25_tokenize.log` |
| `fuzz__text__fuzz_tokenizer_smoke` | FAIL | 0.1 |  | `logs/297_fuzz__text__fuzz_tokenizer_smoke.log` |
| `fuzz__db__rrf_fusion` | FAIL | 0.1 |  | `logs/298_fuzz__db__rrf_fusion.log` |
| `fuzz__crypto__deletion_proof_tamper` | FAIL | 0.1 |  | `logs/299_fuzz__crypto__deletion_proof_tamper.log` |
| `fuzz__crypto__deletion_proof_verify_external_fuzz` | FAIL | 0.1 |  | `logs/300_fuzz__crypto__deletion_proof_verify_external_fuzz.log` |
| `fuzz__crypto__wal_hmac_chain_verify_fuzz` | FAIL | 0.1 |  | `logs/301_fuzz__crypto__wal_hmac_chain_verify_fuzz.log` |
| `fuzz__vector__fuzz_hnsw_persistence` | FAIL | 0.1 |  | `logs/302_fuzz__vector__fuzz_hnsw_persistence.log` |
| `fuzz__vector__hnsw_insert_search` | FAIL | 0.1 |  | `logs/303_fuzz__vector__hnsw_insert_search.log` |
| `fuzz__store__fuzz_compaction_interleave` | FAIL | 0.1 |  | `logs/304_fuzz__store__fuzz_compaction_interleave.log` |
| `fuzz__store__fuzz_manifest_load` | FAIL | 0.1 |  | `logs/305_fuzz__store__fuzz_manifest_load.log` |
| `fuzz__store__fuzz_memtable_concurrent` | FAIL | 0.1 |  | `logs/306_fuzz__store__fuzz_memtable_concurrent.log` |
| `fuzz__store__fuzz_recovery_arbitrary_state` | FAIL | 0.1 |  | `logs/307_fuzz__store__fuzz_recovery_arbitrary_state.log` |
| `fuzz__store__fuzz_sstable_binary_search` | FAIL | 0.1 |  | `logs/308_fuzz__store__fuzz_sstable_binary_search.log` |
| `fuzz__store__fuzz_wal_replay` | FAIL | 0.1 |  | `logs/309_fuzz__store__fuzz_wal_replay.log` |
| `fuzz__store__wal_mutation_chaos` | FAIL | 0.1 |  | `logs/310_fuzz__store__wal_mutation_chaos.log` |
| `fuzz__store__wal_roundtrip` | FAIL | 0.1 |  | `logs/311_fuzz__store__wal_roundtrip.log` |
| `mutation_tier0_1` | FAIL | 0.2 |  | `logs/315_mutation_tier0_1.log` |
| `loom_mvcc` | FAIL | 0.1 |  | `logs/316_loom_mvcc.log` |
| `miri_vector_scalar` | FAIL | 0.3 |  | `logs/317_miri_vector_scalar.log` |
| `asan_vector` | FAIL | 0.1 |  | `logs/318_asan_vector.log` |
| `coverage` | FAIL | 0.2 |  | `logs/319_coverage.log` |
| `fuzz__ingest__fuzz_detect` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/280_fuzz__ingest__fuzz_detect.log` |
| `fuzz__ingest__fuzz_docx` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/281_fuzz__ingest__fuzz_docx.log` |
| `fuzz__ingest__fuzz_email` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/282_fuzz__ingest__fuzz_email.log` |
| `fuzz__ingest__fuzz_odf` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/283_fuzz__ingest__fuzz_odf.log` |
| `fuzz__ingest__fuzz_ole` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/284_fuzz__ingest__fuzz_ole.log` |
| `fuzz__ingest__fuzz_pdf` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/285_fuzz__ingest__fuzz_pdf.log` |
| `fuzz__ingest__fuzz_pipeline` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/286_fuzz__ingest__fuzz_pipeline.log` |
| `fuzz__ingest__fuzz_pptx` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/287_fuzz__ingest__fuzz_pptx.log` |
| `fuzz__ingest__fuzz_safe_archive` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/288_fuzz__ingest__fuzz_safe_archive.log` |
| `fuzz__ingest__fuzz_safe_xml` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/289_fuzz__ingest__fuzz_safe_xml.log` |
| `fuzz__ingest__fuzz_sheets` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/290_fuzz__ingest__fuzz_sheets.log` |
| `fuzz__ingest__fuzz_structured` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/291_fuzz__ingest__fuzz_structured.log` |
| `fuzz__ingest__fuzz_text` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/292_fuzz__ingest__fuzz_text.log` |
| `fuzz__ingest__fuzz_web` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') | `logs/293_fuzz__ingest__fuzz_web.log` |
| `fuzz__mcp__fuzz_jsonrpc_parsing` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-mcp') | `logs/294_fuzz__mcp__fuzz_jsonrpc_parsing.log` |
| `fuzz__mcp__fuzz_prompt_injection_guard` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-mcp') | `logs/295_fuzz__mcp__fuzz_prompt_injection_guard.log` |
| `fuzz__wire__fuzz_flatbuffers_ipc` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-wire') | `logs/312_fuzz__wire__fuzz_flatbuffers_ipc.log` |
| `fuzz__wire__fuzz_jsonrpc` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-wire') | `logs/313_fuzz__wire__fuzz_jsonrpc.log` |
| `fuzz__wire__fuzz_verification` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-wire') | `logs/314_fuzz__wire__fuzz_verification.log` |

## Panic-Flaeche im Produktivcode (Clippy, lib, ohne Tests)

| Crate | unwrap | expect | panic | todo | index | Summe |
|---|---:|---:|---:|---:|---:|---:|
| `contextra-adapt` | 0 | 0 | 0 | 0 | 170 | 170 |
| `contextra-graph` | 0 | 0 | 0 | 0 | 74 | 74 |
| `contextra-store` | 0 | 2 | 0 | 0 | 62 | 64 |
| `contextra-kvcache` | 0 | 0 | 0 | 0 | 23 | 23 |
| `contextra-crypto` | 0 | 0 | 0 | 0 | 22 | 22 |
| `contextra-text` | 0 | 0 | 0 | 0 | 16 | 16 |
| `contextra-engine` | 0 | 0 | 0 | 0 | 9 | 9 |
| `contextra-durable-fs` | 0 | 0 | 0 | 0 | 7 | 7 |
| `contextra-infer-candle` | 0 | 0 | 0 | 0 | 3 | 3 |

## unsafe-Inventar gegen capabilities.toml

- `contextra-arch-tests`: 0 Zeilen, island=False, nicht in capabilities.toml
- `contextra-bench`: 0 Zeilen, island=False, nicht in capabilities.toml
- `contextra-durable-fs`: 0 Zeilen, island=False, nicht in capabilities.toml
- `contextra-simd`: 269 Zeilen, island=True, ok
- `contextra-sys`: 21 Zeilen, island=True, ok
- `contextra-wire`: 45 Zeilen, island=True, ok

## Reife-Behauptung vs. Beleg (capabilities.toml gegen diesen Lauf)

| Crate | behauptet | lib-Tests | Clippy lib | Urteil |
|---|---|---|---|---|
| `contextra` | stable | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-adapt` | beta | PASS | FAIL | **Behauptung ohne Beleg** |
| `contextra-agent` | beta | PASS | FAIL | **Behauptung ohne Beleg** |
| `contextra-audit-export` | experimental | PASS | PASS | ok |
| `contextra-avv-generator` | experimental | PASS | PASS | ok |
| `contextra-checkpoint` | stable | PASS | PASS | ok |
| `contextra-cognition` | experimental | PASS | FAIL | - |
| `contextra-core` | deprecated | PASS | FAIL | - |
| `contextra-crypto` | beta | PASS | FAIL | **Behauptung ohne Beleg** |
| `contextra-db` | beta | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-engine` | beta | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-graph` | stable | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-infer-candle` | stable | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-infer-ollama` | beta | FAIL | PASS | **Behauptung ohne Beleg** |
| `contextra-infer-onnx` | experimental | FAIL | PASS | - |
| `contextra-ingest` | experimental | FAIL | FAIL | - |
| `contextra-kvcache` | beta | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-license` | experimental | FAIL | PASS | - |
| `contextra-mcp` | beta | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-mvcc` | stable | FAIL | PASS | **Behauptung ohne Beleg** |
| `contextra-ports` | stable | FAIL | PASS | **Behauptung ohne Beleg** |
| `contextra-privacy` | stable | FAIL | PASS | **Behauptung ohne Beleg** |
| `contextra-py` | stable | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-rank` | stable | FAIL | PASS | **Behauptung ohne Beleg** |
| `contextra-router` | stable | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-sandbox` | stable | FAIL | PASS | **Behauptung ohne Beleg** |
| `contextra-simd` | beta | FAIL | PASS | **Behauptung ohne Beleg** |
| `contextra-store` | beta | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-sys` | stable | FAIL | PASS | **Behauptung ohne Beleg** |
| `contextra-testkit` | stable | FAIL | PASS | **Behauptung ohne Beleg** |
| `contextra-text` | beta | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-types` | stable | FAIL | PASS | **Behauptung ohne Beleg** |
| `contextra-vector` | beta | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-wire` | stable | FAIL | PASS | **Behauptung ohne Beleg** |

## Alle Schritte

| Gruppe | Schritt | Status | Sek | Detail |
|---|---|---|---:|---|
| B-befunde | `befund_cfg_features_undefiniert` | FAIL | 0.4 | 1 Crates mit undefinierten Features |
| B-befunde | `befund_wal_aead_tail_hinweis` | INFO | 0.0 | Toleranz (AEAD-Entschluesselungsfehler am Dateiende) Z.[479, 611]; Ablehnung (Parsefehler nach erfolgreicher Entschluesselung) Z.[531]. Kein Widerspruch, da verschiedene Fehlerklassen; Policy bitte bestaetigen. |
| B-befunde | `befund_safe_fn_mit_unsafe_inhalt` | FAIL | 0.0 | 2 bekannte K7/K8-Stellen (mmap_readonly, reopen_and_dup2) | gesamt 13 Kandidaten |
| B-befunde | `befund_ingest_verdrahtung` | FAIL | 0.0 | kein Workspace-Crate haengt von contextra-ingest ab; ingest haengt direkt an contextra-engine (Produktmenge darf den Kern nicht ziehen) |
| B-befunde | `befund_default_members` | FAIL | 0.0 | members=37 default-members=27; Produkt nicht Mitglied: []; Produkt nicht in default-members: ['contextra-ingest', 'contextra-privacy', 'contextra-audit-export', 'contextra-avv-generator'] |
| B-befunde | `befund_adr_status` | PASS | 0.0 | 1 ADR-Dateien in 1 Verzeichnissen; Proposed: keine |
| B-befunde | `befund_from_entropy` | FAIL | 0.0 | 4 Produktivdateien mit from_entropy (K14) |
| B-befunde | `befund_maturity_claims` | INFO | 0.0 | stable=15 beta=12 Eintraege ohne Crate=0 |
| B-befunde | `befund_windows_flush` | INFO | 0.0 | windows-Pfad vorhanden=True, compile_error!=False |
| B-befunde | `befund_bench_fuzz_dateien` | PASS | 0.0 | 0 fehlende Dateien/Verzeichnisse |
| T1-produkt | `produkt_huelle_ohne_kern` | FAIL | 3.5 | 1 Produkt-Crates mit Befund |
| T1-produkt | `produkt_sqlcipher_probe` | PASS | 3.0 | rusqlite=* cipher=Some("4.19.0 community") fts5=true leak=false wrong_key_rejected=true rows=1 |
| T1-produkt | `produkt_crates_io_versionen` | INFO | 5.6 | 8 Abfragen |
| 0-basis | `toolchain_preflight` | PASS | 0.1 | cargo 1.89.0 (c24e10642 2025-06-23) | Pin 1.89.0 |
| 0-basis | `toolchain_versionen` | INFO | 0.4 |  |
| 0-basis | `cargo_fetch` | PASS | 0.6 |  |
| 0-basis | `cargo_tree_duplicates` | INFO | 0.5 |  |
| 1-statisch | `fmt_check` | FAIL | 7.9 |  |
| 1-statisch | `check_alle_targets` | FAIL | 145.8 | errors=26 |
| 1-statisch | `check_no_default_features` | PASS | 6.7 | errors=0 |
| 1-statisch | `doc` | FAIL | 11.9 |  |
| 1-statisch | `clippy_lib__contextra` | FAIL | 32.0 | errors=303 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra` | FAIL | 7.3 | errors=190 warnings=1 |
| 1-statisch | `clippy_lib__contextra-adapt` | FAIL | 11.1 | errors=190 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-adapt` | FAIL | 8.3 | errors=260 warnings=1 |
| 1-statisch | `clippy_lib__contextra-agent` | FAIL | 4.8 | errors=303 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-agent` | FAIL | 5.5 | errors=303 warnings=1 |
| 1-statisch | `clippy_lib__contextra-arch-tests` | PASS | 6.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-arch-tests` | PASS | 0.7 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-audit-export` | PASS | 5.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-audit-export` | PASS | 5.6 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-avv-generator` | PASS | 10.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-avv-generator` | PASS | 0.9 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-bench` | FAIL | 16.3 | errors=305 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-bench` | FAIL | 7.6 | errors=305 warnings=1 |
| 1-statisch | `clippy_lib__contextra-checkpoint` | PASS | 7.7 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-checkpoint` | FAIL | 21.2 | errors=49 warnings=0 |
| 1-statisch | `clippy_lib__contextra-cognition` | FAIL | 3.5 | errors=303 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-cognition` | FAIL | 12.8 | errors=303 warnings=1 |
| 1-statisch | `clippy_lib__contextra-core` | FAIL | 1.2 | errors=2 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-core` | FAIL | 1.4 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-crypto` | FAIL | 7.1 | errors=49 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-crypto` | FAIL | 10.0 | errors=49 warnings=0 |
| 1-statisch | `clippy_lib__contextra-db` | FAIL | 3.9 | errors=303 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-db` | FAIL | 11.0 | errors=303 warnings=1 |
| 1-statisch | `clippy_lib__contextra-durable-fs` | FAIL | 1.1 | errors=9 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-durable-fs` | FAIL | 0.9 | errors=10 warnings=1 |
| 1-statisch | `clippy_lib__contextra-engine` | FAIL | 3.7 | errors=303 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-engine` | FAIL | 13.1 | errors=303 warnings=1 |
| 1-statisch | `clippy_lib__contextra-graph` | FAIL | 2.2 | errors=190 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-graph` | FAIL | 5.1 | errors=239 warnings=1 |
| 1-statisch | `clippy_lib__contextra-infer-candle` | FAIL | 24.7 | errors=6 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-infer-candle` | FAIL | 25.2 | errors=49 warnings=1 |
| 1-statisch | `clippy_lib__contextra-infer-ollama` | PASS | 26.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-infer-ollama` | FAIL | 12.5 | errors=39 warnings=1 |
| 1-statisch | `clippy_lib__contextra-infer-onnx` | PASS | 2.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-infer-onnx` | FAIL | 21.3 | errors=20 warnings=0 |
| 1-statisch | `clippy_lib__contextra-ingest` | FAIL | 4.0 | errors=303 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-ingest` | FAIL | 21.0 | errors=303 warnings=1 |
| 1-statisch | `clippy_lib__contextra-kvcache` | FAIL | 2.9 | errors=49 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-kvcache` | FAIL | 7.2 | errors=49 warnings=0 |
| 1-statisch | `clippy_lib__contextra-license` | PASS | 2.8 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-license` | PASS | 2.6 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-mcp` | FAIL | 5.8 | errors=303 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-mcp` | FAIL | 3.6 | errors=303 warnings=1 |
| 1-statisch | `clippy_lib__contextra-mvcc` | PASS | 0.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-mvcc` | PASS | 8.7 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-ports` | PASS | 0.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-ports` | PASS | 6.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-privacy` | PASS | 2.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-privacy` | FAIL | 4.9 | errors=49 warnings=0 |
| 1-statisch | `clippy_lib__contextra-py` | FAIL | 4.4 | errors=303 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-py` | FAIL | 8.0 | errors=303 warnings=1 |
| 1-statisch | `clippy_lib__contextra-rank` | PASS | 2.8 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-rank` | FAIL | 3.0 | errors=190 warnings=1 |
| 1-statisch | `clippy_lib__contextra-router` | FAIL | 1.9 | errors=190 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-router` | FAIL | 9.2 | errors=303 warnings=1 |
| 1-statisch | `clippy_lib__contextra-sandbox` | PASS | 56.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-sandbox` | PASS | 18.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-simd` | PASS | 3.8 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-simd` | PASS | 6.2 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-store` | FAIL | 6.7 | errors=49 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-store` | FAIL | 21.7 | errors=239 warnings=1 |
| 1-statisch | `clippy_lib__contextra-sys` | PASS | 0.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-sys` | PASS | 0.8 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-testkit` | PASS | 2.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-testkit` | PASS | 1.8 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-text` | FAIL | 7.1 | errors=64 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-text` | FAIL | 8.2 | errors=188 warnings=1 |
| 1-statisch | `clippy_lib__contextra-types` | PASS | 0.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-types` | PASS | 5.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-vector` | FAIL | 3.1 | errors=49 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-vector` | FAIL | 4.7 | errors=49 warnings=0 |
| 1-statisch | `clippy_lib__contextra-wire` | PASS | 8.6 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-wire` | PASS | 1.3 | errors=0 warnings=0 |
| 1-statisch | `feature_matrix_hack` | FAIL | 1062.1 |  |
| 1-statisch | `deny` | FAIL | 3.3 |  |
| 1-statisch | `audit` | FAIL | 4.9 |  |
| 1-statisch | `panic_flaeche_lib` | FAIL | 37.3 | errors=2 warnings=625 |
| 1-statisch | `unsafe_inventar` | PASS | 0.4 | keine Verstoesse |
| 2-tests | `test_lib__contextra` | FAIL | 81.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra` | FAIL | 2.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra` | PASS | 0.7 | passed=0 failed=0 ignored=2 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra` | FAIL | 51.5 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-adapt` | PASS | 17.2 | passed=74 failed=0 ignored=1 |
| 2-tests | `test_integration__contextra-adapt` | PASS | 19.2 | passed=134 failed=0 ignored=1 |
| 2-tests | `test_doc__contextra-adapt` | PASS | 1.5 | passed=3 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-adapt` | PASS | 13.7 | passed=134 failed=0 |
| 2-tests | `test_lib__contextra-agent` | PASS | 23.9 | passed=45 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-agent` | PASS | 373.0 | passed=121 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-agent` | PASS | 1.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-agent` | FAIL | 120.8 | passed=120 failed=1 | FAILED: ( 89/121) contextra-agent context::tests::test_telemetry_100k_insertions_and_amortized_performance |
| 2-tests | `test_lib__contextra-arch-tests` | PASS | 15.3 | passed=3 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-arch-tests` | FAIL | 2.3 | passed=3 failed=2 ignored=0 | FAILED: ring_regeln_gelten, unsafe_nur_in_inseln |
| 2-tests | `test_doc__contextra-arch-tests` | PASS | 0.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-arch-tests` | FAIL | 1.8 | passed=3 failed=2 | FAILED: (4/5) contextra-arch-tests::arch ring_regeln_gelten, (5/5) contextra-arch-tests::arch unsafe_nur_in_inseln |
| 2-tests | `test_lib__contextra-audit-export` | PASS | 18.1 | passed=2 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-audit-export` | FAIL | 4.0 | passed=31 failed=1 ignored=0 | FAILED: test_h10_gdpr_art30_processing_registry_and_canary_leak_check |
| 2-tests | `test_doc__contextra-audit-export` | PASS | 0.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-audit-export` | FAIL | 1.1 | passed=31 failed=1 | FAILED: (21/32) contextra-audit-export::campaign_j27_compliance_test test_h10_gdpr_art30_processing_registry_and_canary_leak_check |
| 2-tests | `test_lib__contextra-avv-generator` | PASS | 10.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-avv-generator` | PASS | 5.2 | passed=25 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-avv-generator` | PASS | 1.0 | passed=3 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-avv-generator` | PASS | 1.1 | passed=25 failed=0 |
| 2-tests | `test_lib__contextra-bench` | PASS | 170.0 | passed=18 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-bench` | PASS | 130.5 | passed=39 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-bench` | PASS | 4.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-bench` | PASS | 44.7 | passed=39 failed=0 |
| 2-tests | `test_lib__contextra-checkpoint` | PASS | 55.6 | passed=56 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-checkpoint` | PASS | 32.9 | passed=121 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-checkpoint` | PASS | 1.1 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-checkpoint` | PASS | 3.3 | passed=121 failed=0 |
| 2-tests | `test_lib__contextra-cognition` | PASS | 132.3 | passed=58 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-cognition` | PASS | 102.7 | passed=105 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-cognition` | PASS | 4.3 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-cognition` | PASS | 3.7 | passed=105 failed=0 |
| 2-tests | `test_lib__contextra-core` | PASS | 3.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-core` | PASS | 2.6 | passed=7 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-core` | PASS | 0.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-core` | PASS | 1.3 | passed=7 failed=0 |
| 2-tests | `test_lib__contextra-crypto` | PASS | 107.8 | passed=107 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-crypto` | FAIL | 2316.4 | passed=304 failed=2 ignored=0 | FAILED: test_scenario_3_deletion_proof_v3_creation_and_exhaustive_mutation_test, test_scenario_4_independent_external_verification_and_tempfile |
| 2-tests | `test_doc__contextra-crypto` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-crypto` | FAIL | 0.3 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-db` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-db` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-db` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-db` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-durable-fs` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-durable-fs` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-durable-fs` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-durable-fs` | FAIL | 0.3 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-engine` | FAIL | 0.2 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-engine` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-engine` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-engine` | FAIL | 0.3 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-graph` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-graph` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-graph` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-graph` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-infer-candle` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-infer-candle` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-infer-candle` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-infer-candle` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-infer-ollama` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-infer-ollama` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-infer-ollama` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-infer-ollama` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-infer-onnx` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-infer-onnx` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-infer-onnx` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-infer-onnx` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-ingest` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-ingest` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-ingest` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-ingest` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-kvcache` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-kvcache` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-kvcache` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-kvcache` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-license` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-license` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-license` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-license` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-mcp` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-mcp` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-mcp` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-mcp` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-mvcc` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-mvcc` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-mvcc` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-mvcc` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-ports` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-ports` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-ports` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-ports` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-privacy` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-privacy` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-privacy` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-privacy` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-py` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-py` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-py` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-py` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-rank` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-rank` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-rank` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-rank` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-router` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-router` | FAIL | 0.4 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-router` | FAIL | 0.5 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-router` | FAIL | 1.0 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-sandbox` | FAIL | 0.6 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-sandbox` | FAIL | 0.4 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-sandbox` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-sandbox` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-simd` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-simd` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-simd` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-simd` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-store` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-store` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-store` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-store` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-sys` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-sys` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-sys` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-sys` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-testkit` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-testkit` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-testkit` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-testkit` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-text` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-text` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-text` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-text` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-types` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-types` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-types` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-types` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-vector` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-vector` | FAIL | 0.2 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-vector` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-vector` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-wire` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-wire` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-wire` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-wire` | FAIL | 0.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_gesamt_debug` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_gesamt_release` | FAIL | 0.1 | passed=0 failed=0 ignored=0 |
| 2-tests | `python_bindings_pytest` | FAIL | 0.0 |  |
| 4-bench | `bench__contextra-mvcc__mvcc_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-checkpoint__checkpoint_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-router__router_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-router__bandit_latency_gate` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-router__cost_routing_savings_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-graph__csr_traversal_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-graph__binary_edge_regression` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-crypto__crypto_benchmarks` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-crypto__deletion_proof_latency_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-infer-candle__kv_bridge_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-vector__distance_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-vector__sq8_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-vector__hnsw_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-vector__hnsw_delete_search_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-vector__audit_benchmarks` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-vector__flush_threshold_amplification` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-vector__partial_rebuild_recall_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-vector__distance_nan_impact_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-vector__filtered_search_selectivity_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-store__sstable_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-store__memtable_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-store__wal_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-store__lsm_concurrency_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-store__block_cache_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-store__redb_comparative_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-simd__simd_bench` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-bench__competitive_kv` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-bench__locomo` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-bench__long_mem_eval` | FAIL | 0.1 | keine Criterion-Zeilen erkannt |
| 5-tief | `fuzz__ingest__fuzz_detect` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_docx` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_email` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_odf` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_ole` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_pdf` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_pipeline` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_pptx` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_safe_archive` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_safe_xml` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_sheets` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_structured` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_text` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__ingest__fuzz_web` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-ingest') |
| 5-tief | `fuzz__mcp__fuzz_jsonrpc_parsing` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-mcp') |
| 5-tief | `fuzz__mcp__fuzz_prompt_injection_guard` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-mcp') |
| 5-tief | `fuzz__text__fuzz_bm25_tokenize` | FAIL | 0.3 |  |
| 5-tief | `fuzz__text__fuzz_tokenizer_smoke` | FAIL | 0.1 |  |
| 5-tief | `fuzz__db__rrf_fusion` | FAIL | 0.1 |  |
| 5-tief | `fuzz__crypto__deletion_proof_tamper` | FAIL | 0.1 |  |
| 5-tief | `fuzz__crypto__deletion_proof_verify_external_fuzz` | FAIL | 0.1 |  |
| 5-tief | `fuzz__crypto__wal_hmac_chain_verify_fuzz` | FAIL | 0.1 |  |
| 5-tief | `fuzz__vector__fuzz_hnsw_persistence` | FAIL | 0.1 |  |
| 5-tief | `fuzz__vector__hnsw_insert_search` | FAIL | 0.1 |  |
| 5-tief | `fuzz__store__fuzz_compaction_interleave` | FAIL | 0.1 |  |
| 5-tief | `fuzz__store__fuzz_manifest_load` | FAIL | 0.1 |  |
| 5-tief | `fuzz__store__fuzz_memtable_concurrent` | FAIL | 0.1 |  |
| 5-tief | `fuzz__store__fuzz_recovery_arbitrary_state` | FAIL | 0.1 |  |
| 5-tief | `fuzz__store__fuzz_sstable_binary_search` | FAIL | 0.1 |  |
| 5-tief | `fuzz__store__fuzz_wal_replay` | FAIL | 0.1 |  |
| 5-tief | `fuzz__store__wal_mutation_chaos` | FAIL | 0.1 |  |
| 5-tief | `fuzz__store__wal_roundtrip` | FAIL | 0.1 |  |
| 5-tief | `fuzz__wire__fuzz_flatbuffers_ipc` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-wire') |
| 5-tief | `fuzz__wire__fuzz_jsonrpc` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-wire') |
| 5-tief | `fuzz__wire__fuzz_verification` | SKIPPED | 0.0 | Befehl nicht gefunden: [Errno 2] No such file or directory: PosixPath('/home/freddy/Projekte/contextra/crates/contextra-wire') |
| 5-tief | `mutation_tier0_1` | FAIL | 0.2 |  |
| 5-tief | `loom_mvcc` | FAIL | 0.1 |  |
| 5-tief | `miri_vector_scalar` | FAIL | 0.3 |  |
| 5-tief | `asan_vector` | FAIL | 0.1 |  |
| 5-tief | `coverage` | FAIL | 0.2 |  |
