# Contextra Voll-Audit

- Commit: `unknown` (), Branch ``, hinter origin: ?, ungespeicherte Aenderungen: False
- Start: 2026-11-25T07:56:06+00:00  Dauer: 66.9 min
- Toolchain-Pin: `1.89.0`; Preflight: **PASS** (cargo 1.89.0 (c24e10642 2025-06-23) | Pin 1.89.0)
- Host: Linux-7.3.3-1-liquorix-amd64-x86_64-with-glibc2.41 | 8 CPUs | MemTotal:       10120436 kB
- Nutzer: freddy

## Gesamt

- **FAIL**: 76
- **INFO**: 4
- **PASS**: 17
- **PENDING**: 213

Hinweis: Bei `befund_*`-Schritten bedeutet FAIL, dass der Befund **vorliegt**. `INFO` ist eine Messung oder ein Hinweis ohne Urteil.

## Rote und uebersprungene Schritte

| Schritt | Status | Sek | Detail | Log |
|---|---|---:|---|---|
| `befund_cfg_features_undefiniert` | FAIL | 0.1 | 1 Crates mit undefinierten Features | `logs/000_befund_cfg_features_undefiniert.log` |
| `fmt_check` | FAIL | 5.2 |  | `logs/008_fmt_check.log` |
| `check_alle_targets` | FAIL | 324.0 | errors=12 | `logs/009_check_alle_targets.log` |
| `doc` | FAIL | 22.1 |  | `logs/011_doc.log` |
| `clippy_lib__contextra` | FAIL | 10.2 | errors=3 warnings=1 | `logs/012_clippy_lib__contextra.log` |
| `clippy_alle_targets__contextra` | FAIL | 7.2 | errors=3 warnings=1 | `logs/013_clippy_alle_targets__contextra.log` |
| `clippy_lib__contextra-adapt` | FAIL | 10.3 | errors=3 warnings=1 | `logs/014_clippy_lib__contextra-adapt.log` |
| `clippy_alle_targets__contextra-adapt` | FAIL | 8.2 | errors=3 warnings=0 | `logs/015_clippy_alle_targets__contextra-adapt.log` |
| `clippy_lib__contextra-agent` | FAIL | 19.1 | errors=3 warnings=1 | `logs/016_clippy_lib__contextra-agent.log` |
| `clippy_alle_targets__contextra-agent` | FAIL | 1.8 | errors=3 warnings=0 | `logs/017_clippy_alle_targets__contextra-agent.log` |
| `clippy_lib__contextra-audit-export` | FAIL | 4.7 | errors=3 warnings=0 | `logs/018_clippy_lib__contextra-audit-export.log` |
| `clippy_alle_targets__contextra-audit-export` | FAIL | 4.6 | errors=3 warnings=0 | `logs/019_clippy_alle_targets__contextra-audit-export.log` |
| `clippy_alle_targets__contextra-avv-generator` | FAIL | 0.9 | errors=5 warnings=1 | `logs/021_clippy_alle_targets__contextra-avv-generator.log` |
| `clippy_lib__contextra-bench` | FAIL | 1.9 | errors=3 warnings=0 | `logs/022_clippy_lib__contextra-bench.log` |
| `clippy_alle_targets__contextra-bench` | FAIL | 13.9 | errors=3 warnings=1 | `logs/023_clippy_alle_targets__contextra-bench.log` |
| `clippy_lib__contextra-checkpoint` | FAIL | 4.6 | errors=3 warnings=0 | `logs/024_clippy_lib__contextra-checkpoint.log` |
| `clippy_alle_targets__contextra-checkpoint` | FAIL | 19.6 | errors=3 warnings=0 | `logs/025_clippy_alle_targets__contextra-checkpoint.log` |
| `clippy_lib__contextra-cognition` | FAIL | 1.6 | errors=3 warnings=0 | `logs/026_clippy_lib__contextra-cognition.log` |
| `clippy_alle_targets__contextra-cognition` | FAIL | 5.4 | errors=3 warnings=1 | `logs/027_clippy_alle_targets__contextra-cognition.log` |
| `clippy_lib__contextra-core` | FAIL | 1.4 | errors=3 warnings=0 | `logs/028_clippy_lib__contextra-core.log` |
| `clippy_alle_targets__contextra-core` | FAIL | 1.8 | errors=3 warnings=0 | `logs/029_clippy_alle_targets__contextra-core.log` |
| `clippy_lib__contextra-crypto` | FAIL | 4.3 | errors=3 warnings=0 | `logs/030_clippy_lib__contextra-crypto.log` |
| `clippy_alle_targets__contextra-crypto` | FAIL | 9.6 | errors=3 warnings=1 | `logs/031_clippy_alle_targets__contextra-crypto.log` |
| `clippy_lib__contextra-db` | FAIL | 1.7 | errors=3 warnings=0 | `logs/032_clippy_lib__contextra-db.log` |
| `clippy_alle_targets__contextra-db` | FAIL | 21.6 | errors=3 warnings=1 | `logs/033_clippy_alle_targets__contextra-db.log` |
| `clippy_lib__contextra-engine` | FAIL | 1.6 | errors=3 warnings=0 | `logs/034_clippy_lib__contextra-engine.log` |
| `clippy_alle_targets__contextra-engine` | FAIL | 19.2 | errors=3 warnings=1 | `logs/035_clippy_alle_targets__contextra-engine.log` |
| `clippy_lib__contextra-graph` | FAIL | 1.5 | errors=3 warnings=0 | `logs/036_clippy_lib__contextra-graph.log` |
| `clippy_alle_targets__contextra-graph` | FAIL | 13.1 | errors=3 warnings=1 | `logs/037_clippy_alle_targets__contextra-graph.log` |
| `clippy_lib__contextra-infer-candle` | FAIL | 4.9 | errors=3 warnings=0 | `logs/038_clippy_lib__contextra-infer-candle.log` |
| `clippy_alle_targets__contextra-infer-candle` | FAIL | 4.1 | errors=3 warnings=1 | `logs/039_clippy_alle_targets__contextra-infer-candle.log` |
| `clippy_lib__contextra-infer-ollama` | FAIL | 38.8 | errors=3 warnings=1 | `logs/040_clippy_lib__contextra-infer-ollama.log` |
| `clippy_alle_targets__contextra-infer-ollama` | FAIL | 10.8 | errors=3 warnings=0 | `logs/041_clippy_alle_targets__contextra-infer-ollama.log` |
| `clippy_lib__contextra-infer-onnx` | FAIL | 1.5 | errors=3 warnings=0 | `logs/042_clippy_lib__contextra-infer-onnx.log` |
| `clippy_alle_targets__contextra-infer-onnx` | FAIL | 22.7 | errors=3 warnings=0 | `logs/043_clippy_alle_targets__contextra-infer-onnx.log` |
| `clippy_lib__contextra-kvcache` | FAIL | 1.5 | errors=3 warnings=0 | `logs/044_clippy_lib__contextra-kvcache.log` |
| `clippy_alle_targets__contextra-kvcache` | FAIL | 4.5 | errors=3 warnings=0 | `logs/045_clippy_alle_targets__contextra-kvcache.log` |
| `clippy_lib__contextra-license` | FAIL | 6.1 | errors=3 warnings=1 | `logs/046_clippy_lib__contextra-license.log` |
| `clippy_alle_targets__contextra-license` | FAIL | 2.3 | errors=3 warnings=1 | `logs/047_clippy_alle_targets__contextra-license.log` |
| `clippy_lib__contextra-mcp` | FAIL | 2.0 | errors=3 warnings=0 | `logs/048_clippy_lib__contextra-mcp.log` |
| `clippy_alle_targets__contextra-mcp` | FAIL | 1.7 | errors=3 warnings=0 | `logs/049_clippy_alle_targets__contextra-mcp.log` |
| `clippy_lib__contextra-mvcc` | FAIL | 1.6 | errors=3 warnings=0 | `logs/050_clippy_lib__contextra-mvcc.log` |
| `clippy_alle_targets__contextra-mvcc` | FAIL | 3.5 | errors=3 warnings=1 | `logs/051_clippy_alle_targets__contextra-mvcc.log` |
| `clippy_lib__contextra-ports` | FAIL | 1.5 | errors=3 warnings=0 | `logs/052_clippy_lib__contextra-ports.log` |
| `clippy_alle_targets__contextra-ports` | FAIL | 7.4 | errors=4 warnings=1 | `logs/053_clippy_alle_targets__contextra-ports.log` |
| `clippy_lib__contextra-privacy` | FAIL | 2.2 | errors=3 warnings=0 | `logs/054_clippy_lib__contextra-privacy.log` |
| `clippy_alle_targets__contextra-privacy` | FAIL | 1.9 | errors=3 warnings=0 | `logs/055_clippy_alle_targets__contextra-privacy.log` |
| `clippy_lib__contextra-py` | FAIL | 8.5 | errors=3 warnings=1 | `logs/056_clippy_lib__contextra-py.log` |
| `clippy_alle_targets__contextra-py` | FAIL | 4.2 | errors=3 warnings=1 | `logs/057_clippy_alle_targets__contextra-py.log` |
| `clippy_lib__contextra-rank` | FAIL | 1.5 | errors=3 warnings=0 | `logs/058_clippy_lib__contextra-rank.log` |
| `clippy_alle_targets__contextra-rank` | FAIL | 1.6 | errors=3 warnings=0 | `logs/059_clippy_alle_targets__contextra-rank.log` |
| `clippy_lib__contextra-router` | FAIL | 1.8 | errors=3 warnings=0 | `logs/060_clippy_lib__contextra-router.log` |
| `clippy_alle_targets__contextra-router` | FAIL | 2.7 | errors=3 warnings=1 | `logs/061_clippy_alle_targets__contextra-router.log` |
| `clippy_lib__contextra-sandbox` | FAIL | 39.7 | errors=18 warnings=0 | `logs/062_clippy_lib__contextra-sandbox.log` |
| `clippy_alle_targets__contextra-sandbox` | FAIL | 12.5 | errors=19 warnings=1 | `logs/063_clippy_alle_targets__contextra-sandbox.log` |
| `clippy_lib__contextra-simd` | FAIL | 1.5 | errors=3 warnings=0 | `logs/064_clippy_lib__contextra-simd.log` |
| `clippy_alle_targets__contextra-simd` | FAIL | 3.1 | errors=3 warnings=1 | `logs/065_clippy_alle_targets__contextra-simd.log` |
| `clippy_lib__contextra-store` | FAIL | 1.9 | errors=3 warnings=0 | `logs/066_clippy_lib__contextra-store.log` |
| `clippy_alle_targets__contextra-store` | FAIL | 30.3 | errors=3 warnings=1 | `logs/067_clippy_alle_targets__contextra-store.log` |
| `clippy_lib__contextra-testkit` | FAIL | 2.1 | errors=3 warnings=0 | `logs/070_clippy_lib__contextra-testkit.log` |
| `clippy_alle_targets__contextra-testkit` | FAIL | 2.7 | errors=3 warnings=0 | `logs/071_clippy_alle_targets__contextra-testkit.log` |
| `clippy_lib__contextra-text` | FAIL | 1.9 | errors=3 warnings=0 | `logs/072_clippy_lib__contextra-text.log` |
| `clippy_alle_targets__contextra-text` | FAIL | 1.5 | errors=3 warnings=0 | `logs/073_clippy_alle_targets__contextra-text.log` |
| `clippy_alle_targets__contextra-types` | FAIL | 7.1 | errors=20 warnings=1 | `logs/075_clippy_alle_targets__contextra-types.log` |
| `clippy_lib__contextra-vector` | FAIL | 1.6 | errors=3 warnings=0 | `logs/076_clippy_lib__contextra-vector.log` |
| `clippy_alle_targets__contextra-vector` | FAIL | 3.9 | errors=3 warnings=1 | `logs/077_clippy_alle_targets__contextra-vector.log` |
| `clippy_lib__xtask` | FAIL | 42.3 | errors=12 warnings=0 | `logs/080_clippy_lib__xtask.log` |
| `clippy_alle_targets__xtask` | FAIL | 26.7 | errors=13 warnings=1 | `logs/081_clippy_alle_targets__xtask.log` |
| `clippy_alle_targets__xtask-heavy` | FAIL | 2.1 | errors=3 warnings=0 | `logs/082_clippy_alle_targets__xtask-heavy.log` |
| `feature_matrix_hack` | FAIL | 1212.9 |  | `logs/083_feature_matrix_hack.log` |
| `deny` | FAIL | 2.7 |  | `logs/084_deny.log` |
| `audit` | FAIL | 0.2 |  | `logs/085_audit.log` |
| `panic_flaeche_lib` | FAIL | 41.9 | errors=21 warnings=635 | `logs/086_panic_flaeche_lib.log` |
| `test_integration__contextra` | FAIL | 624.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/089_test_integration__contextra.log` |
| `test_integration__contextra-adapt` | FAIL | 7.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/093_test_integration__contextra-adapt.log` |
| `nextest__contextra-adapt` | FAIL | 3.2 | nextest: keine Summary-Zeile gefunden | `logs/095_nextest__contextra-adapt.log` |

## Panic-Flaeche im Produktivcode (Clippy, lib, ohne Tests)

| Crate | unwrap | expect | panic | todo | index | Summe |
|---|---:|---:|---:|---:|---:|---:|
| `` | 31 | 0 | 0 | 0 | 289 | 320 |
| `xtask` | 43 | 7 | 0 | 0 | 239 | 289 |
| `contextra-sandbox` | 0 | 0 | 0 | 0 | 8 | 8 |
| `contextra-types` | 0 | 0 | 0 | 0 | 3 | 3 |
| `contextra-ports` | 0 | 0 | 0 | 0 | 3 | 3 |

## unsafe-Inventar gegen capabilities.toml

- `contextra-bench`: 0 Zeilen, island=False, nicht in capabilities.toml
- `contextra-simd`: 310 Zeilen, island=True, ok
- `contextra-sys`: 33 Zeilen, island=True, ok
- `contextra-wire`: 45 Zeilen, island=True, ok
- `xtask`: 6 Zeilen, island=False, nicht in capabilities.toml
- `xtask-heavy`: 0 Zeilen, island=False, nicht in capabilities.toml

## Reife-Behauptung vs. Beleg (capabilities.toml gegen diesen Lauf)

| Crate | behauptet | lib-Tests | Clippy lib | Urteil |
|---|---|---|---|---|
| `contextra` | stable | PASS | FAIL | **Behauptung ohne Beleg** |
| `contextra-adapt` | experimental | PASS | FAIL | - |
| `contextra-agent` | stable | PASS | FAIL | **Behauptung ohne Beleg** |
| `contextra-audit-export` | experimental | PENDING | FAIL | - |
| `contextra-avv-generator` | experimental | PENDING | PASS | - |
| `contextra-checkpoint` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-cognition` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-core` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-crypto` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-db` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-engine` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-graph` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-infer-candle` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-infer-ollama` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-infer-onnx` | experimental | PENDING | FAIL | - |
| `contextra-kvcache` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-license` | experimental | PENDING | FAIL | - |
| `contextra-mcp` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-mvcc` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-ports` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-privacy` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-py` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-rank` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-router` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-sandbox` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-simd` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-store` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-sys` | stable | PENDING | PASS | **Behauptung ohne Beleg** |
| `contextra-testkit` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-text` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-types` | stable | PENDING | PASS | **Behauptung ohne Beleg** |
| `contextra-vector` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-wire` | stable | PENDING | PASS | **Behauptung ohne Beleg** |

## Alle Schritte

| Gruppe | Schritt | Status | Sek | Detail |
|---|---|---|---:|---|
| B-befunde | `befund_cfg_features_undefiniert` | FAIL | 0.1 | 1 Crates mit undefinierten Features |
| B-befunde | `befund_default_members` | INFO | 0.0 | members=36 default-members=26; nicht in default-members (von `cargo test` ohne --workspace nicht erfasst): ['contextra-audit-export', 'contextra-avv-generator', 'contextra-bench', 'contextra-infer-ollama', 'contextra-infer-onnx', 'contextra-license', 'contextra-py', 'contextra-sandbox', 'xtask', 'xtask-heavy'] |
| B-befunde | `befund_maturity_claims` | INFO | 0.0 | stable=28 beta=0 Eintraege ohne Crate=0 |
| B-befunde | `befund_bench_fuzz_dateien` | PASS | 0.0 | 0 fehlende Dateien/Verzeichnisse |
| 0-basis | `toolchain_preflight` | PASS | 0.1 | cargo 1.89.0 (c24e10642 2025-06-23) | Pin 1.89.0 |
| 0-basis | `toolchain_versionen` | INFO | 0.4 |  |
| 0-basis | `cargo_fetch` | PASS | 0.7 |  |
| 0-basis | `cargo_tree_duplicates` | INFO | 0.5 |  |
| 1-statisch | `fmt_check` | FAIL | 5.2 |  |
| 1-statisch | `check_alle_targets` | FAIL | 324.0 | errors=12 |
| 1-statisch | `check_no_default_features` | PASS | 72.4 | errors=0 |
| 1-statisch | `doc` | FAIL | 22.1 |  |
| 1-statisch | `clippy_lib__contextra` | FAIL | 10.2 | errors=3 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra` | FAIL | 7.2 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-adapt` | FAIL | 10.3 | errors=3 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-adapt` | FAIL | 8.2 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-agent` | FAIL | 19.1 | errors=3 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-agent` | FAIL | 1.8 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-audit-export` | FAIL | 4.7 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-audit-export` | FAIL | 4.6 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-avv-generator` | PASS | 14.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-avv-generator` | FAIL | 0.9 | errors=5 warnings=1 |
| 1-statisch | `clippy_lib__contextra-bench` | FAIL | 1.9 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-bench` | FAIL | 13.9 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-checkpoint` | FAIL | 4.6 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-checkpoint` | FAIL | 19.6 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-cognition` | FAIL | 1.6 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-cognition` | FAIL | 5.4 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-core` | FAIL | 1.4 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-core` | FAIL | 1.8 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-crypto` | FAIL | 4.3 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-crypto` | FAIL | 9.6 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-db` | FAIL | 1.7 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-db` | FAIL | 21.6 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-engine` | FAIL | 1.6 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-engine` | FAIL | 19.2 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-graph` | FAIL | 1.5 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-graph` | FAIL | 13.1 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-infer-candle` | FAIL | 4.9 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-infer-candle` | FAIL | 4.1 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-infer-ollama` | FAIL | 38.8 | errors=3 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-infer-ollama` | FAIL | 10.8 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-infer-onnx` | FAIL | 1.5 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-infer-onnx` | FAIL | 22.7 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-kvcache` | FAIL | 1.5 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-kvcache` | FAIL | 4.5 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-license` | FAIL | 6.1 | errors=3 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-license` | FAIL | 2.3 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-mcp` | FAIL | 2.0 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-mcp` | FAIL | 1.7 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-mvcc` | FAIL | 1.6 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-mvcc` | FAIL | 3.5 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-ports` | FAIL | 1.5 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-ports` | FAIL | 7.4 | errors=4 warnings=1 |
| 1-statisch | `clippy_lib__contextra-privacy` | FAIL | 2.2 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-privacy` | FAIL | 1.9 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-py` | FAIL | 8.5 | errors=3 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-py` | FAIL | 4.2 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-rank` | FAIL | 1.5 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-rank` | FAIL | 1.6 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-router` | FAIL | 1.8 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-router` | FAIL | 2.7 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-sandbox` | FAIL | 39.7 | errors=18 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-sandbox` | FAIL | 12.5 | errors=19 warnings=1 |
| 1-statisch | `clippy_lib__contextra-simd` | FAIL | 1.5 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-simd` | FAIL | 3.1 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-store` | FAIL | 1.9 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-store` | FAIL | 30.3 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-sys` | PASS | 0.7 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-sys` | PASS | 0.8 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-testkit` | FAIL | 2.1 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-testkit` | FAIL | 2.7 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-text` | FAIL | 1.9 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-text` | FAIL | 1.5 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-types` | PASS | 0.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-types` | FAIL | 7.1 | errors=20 warnings=1 |
| 1-statisch | `clippy_lib__contextra-vector` | FAIL | 1.6 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-vector` | FAIL | 3.9 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-wire` | PASS | 14.8 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-wire` | PASS | 1.2 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__xtask` | FAIL | 42.3 | errors=12 warnings=0 |
| 1-statisch | `clippy_alle_targets__xtask` | FAIL | 26.7 | errors=13 warnings=1 |
| 1-statisch | `clippy_alle_targets__xtask-heavy` | FAIL | 2.1 | errors=3 warnings=0 |
| 1-statisch | `feature_matrix_hack` | FAIL | 1212.9 |  |
| 1-statisch | `deny` | FAIL | 2.7 |  |
| 1-statisch | `audit` | FAIL | 0.2 |  |
| 1-statisch | `panic_flaeche_lib` | FAIL | 41.9 | errors=21 warnings=635 |
| 1-statisch | `unsafe_inventar` | PASS | 0.7 | keine Verstoesse |
| 2-tests | `test_lib__contextra` | PASS | 225.5 | passed=6 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra` | FAIL | 624.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra` | PASS | 2.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra` | PASS | 124.3 | passed=66 failed=0 |
| 2-tests | `test_lib__contextra-adapt` | PASS | 17.3 | passed=73 failed=0 ignored=1 |
| 2-tests | `test_integration__contextra-adapt` | FAIL | 7.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-adapt` | PASS | 0.9 | passed=1 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-adapt` | FAIL | 3.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-agent` | PASS | 47.2 | passed=39 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-agent` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-agent` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-agent` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-audit-export` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-audit-export` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-audit-export` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-audit-export` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-avv-generator` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-avv-generator` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-avv-generator` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-avv-generator` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-bench` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-bench` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-bench` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-bench` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-checkpoint` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-checkpoint` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-checkpoint` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-checkpoint` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-cognition` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-cognition` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-cognition` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-cognition` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-core` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-core` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-core` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-core` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-crypto` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-crypto` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-crypto` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-crypto` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-db` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-db` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-db` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-db` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-graph` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-graph` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-graph` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-graph` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-infer-candle` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-infer-candle` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-infer-candle` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-infer-candle` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-infer-ollama` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-infer-ollama` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-infer-ollama` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-infer-ollama` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-infer-onnx` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-infer-onnx` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-infer-onnx` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-infer-onnx` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-kvcache` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-kvcache` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-kvcache` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-kvcache` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-license` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-license` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-license` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-license` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-mcp` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-mcp` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-mcp` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-mcp` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-privacy` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-privacy` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-privacy` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-privacy` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-py` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-py` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-py` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-py` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-router` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-router` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-router` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-router` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-sandbox` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-sandbox` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-sandbox` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-sandbox` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-store` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-store` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-store` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-store` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-wire` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-wire` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-wire` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-wire` | PENDING | 0.0 |  |
| 2-tests | `test_lib__xtask` | PENDING | 0.0 |  |
| 2-tests | `test_integration__xtask` | PENDING | 0.0 |  |
| 2-tests | `test_doc__xtask` | PENDING | 0.0 |  |
| 2-tests | `nextest__xtask` | PENDING | 0.0 |  |
| 2-tests | `test_integration__xtask-heavy` | PENDING | 0.0 |  |
| 2-tests | `nextest__xtask-heavy` | PENDING | 0.0 |  |
| 2-tests | `test_gesamt_debug` | PENDING | 0.0 |  |
| 2-tests | `test_gesamt_release` | PENDING | 0.0 |  |
| 2-tests | `python_bindings_pytest` | PENDING | 0.0 |  |
| 3-testbarkeit | `testbarkeit_crates` | PENDING | 0.0 |  |
| 3-testbarkeit | `testbarkeit_funktionen` | PENDING | 0.0 |  |
| 3-testbarkeit | `testbarkeit_verwaiste_dateien` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-adapt` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-agent` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-audit-export` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-avv-generator` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-bench` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-checkpoint` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-cognition` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-core` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-crypto` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-db` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-engine` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-graph` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-infer-candle` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-infer-ollama` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-infer-onnx` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-kvcache` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-license` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-mcp` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-mvcc` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-ports` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-privacy` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-py` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-rank` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-router` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-sandbox` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-simd` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-store` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-sys` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-testkit` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-text` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-types` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-vector` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-wire` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__xtask` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__xtask-heavy` | PENDING | 0.0 |  |
| 3-testbarkeit | `coverage` | PENDING | 0.0 |  |
| 3-testbarkeit | `coverage_funktionen` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-mvcc__mvcc_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-crypto__crypto_benchmarks` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__distance_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__sq8_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__hnsw_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__hnsw_delete_search_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__audit_benchmarks` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__flush_threshold_amplification` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__partial_rebuild_recall_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__distance_nan_impact_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__filtered_search_selectivity_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__sstable_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__memtable_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__wal_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__lsm_concurrency_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__block_cache_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__redb_comparative_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-simd__simd_bench` | PENDING | 0.0 |  |
| 5-tief | `fuzz__crypto__deletion_proof_tamper` | PENDING | 0.0 |  |
| 5-tief | `fuzz__crypto__deletion_proof_verify_external_fuzz` | PENDING | 0.0 |  |
| 5-tief | `fuzz__crypto__wal_hmac_chain_verify_fuzz` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_compaction_interleave` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_manifest_load` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_memtable_concurrent` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_recovery_arbitrary_state` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_sstable_binary_search` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_wal_replay` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__wal_mutation_chaos` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__wal_roundtrip` | PENDING | 0.0 |  |
| 5-tief | `fuzz__text__fuzz_bm25_tokenize` | PENDING | 0.0 |  |
| 5-tief | `fuzz__vector__fuzz_hnsw_persistence` | PENDING | 0.0 |  |
| 5-tief | `fuzz__vector__hnsw_insert_search` | PENDING | 0.0 |  |
| 5-tief | `mutation_tier0_1` | PENDING | 0.0 |  |
| 5-tief | `loom_mvcc` | PENDING | 0.0 |  |
| 5-tief | `miri_vector_scalar` | PENDING | 0.0 |  |
| 5-tief | `asan_vector` | PENDING | 0.0 |  |
