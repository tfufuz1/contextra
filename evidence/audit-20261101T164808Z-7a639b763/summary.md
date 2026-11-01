# Contextra Voll-Audit

- Commit: `7a639b763` (2026-11-01 17:23:48 +0100), Branch `main`, hinter origin: 0, ungespeicherte Aenderungen: False
- Start: 2026-11-01T16:48:08+00:00  Dauer: 6.4 min
- Toolchain: rustc 1.89.0 (29483883e 2025-08-04)
- Host: Linux-7.2.7-1-liquorix-amd64-x86_64-with-glibc2.41 | 8 CPUs | MemTotal:       10121020 kB
- Nutzer: freddy

## Gesamt

- **FAIL**: 8
- **INFO**: 2
- **PASS**: 6
- **PENDING**: 302

Hinweis: Bei `befund_*`-Schritten bedeutet FAIL, dass der Befund **vorliegt**. `INFO` ist eine Messung ohne Bestanden/Durchgefallen.

## Rote und uebersprungene Schritte

| Schritt | Status | Sek | Detail | Log |
|---|---|---:|---|---|
| `befund_cfg_features_undefiniert` | FAIL | 0.3 | 1 Crates mit undefinierten Features | `logs/000_befund_cfg_features_undefiniert.log` |
| `befund_wal_aead_tail_widerspruch` | FAIL | 0.0 | tolerierend Z.[479, 611], ablehnend Z.[531] | `logs/001_befund_wal_aead_tail_widerspruch.log` |
| `befund_safe_fn_mit_unsafe_inhalt` | FAIL | 0.0 | 3 safe-Signaturen | `logs/002_befund_safe_fn_mit_unsafe_inhalt.log` |
| `befund_ingest_verdrahtung` | FAIL | 0.0 | kein Crate haengt von contextra-ingest ab; ingest haengt an contextra-engine | `logs/003_befund_ingest_verdrahtung.log` |
| `befund_default_members` | FAIL | 0.0 | members=37 default-members=27; Produkt fehlt in default-members: ['contextra-ingest', 'contextra-privacy', 'contextra-audit-export', 'contextra-avv-generator']; Kern in default-members: 13 | `logs/004_befund_default_members.log` |
| `befund_from_entropy` | FAIL | 0.2 | 4 Dateien mit from_entropy (K14) | `logs/006_befund_from_entropy.log` |
| `produkt_huelle_ohne_kern` | FAIL | 4.3 | 1 Produkt-Crates mit verbotener Huelle | `logs/009_produkt_huelle_ohne_kern.log` |
| `fmt_check` | FAIL | 7.0 |  | `logs/015_fmt_check.log` |

## Alle Schritte

| Gruppe | Schritt | Status | Sek | Detail |
|---|---|---|---:|---|
| B-befunde | `befund_cfg_features_undefiniert` | FAIL | 0.3 | 1 Crates mit undefinierten Features |
| B-befunde | `befund_wal_aead_tail_widerspruch` | FAIL | 0.0 | tolerierend Z.[479, 611], ablehnend Z.[531] |
| B-befunde | `befund_safe_fn_mit_unsafe_inhalt` | FAIL | 0.0 | 3 safe-Signaturen |
| B-befunde | `befund_ingest_verdrahtung` | FAIL | 0.0 | kein Crate haengt von contextra-ingest ab; ingest haengt an contextra-engine |
| B-befunde | `befund_default_members` | FAIL | 0.0 | members=37 default-members=27; Produkt fehlt in default-members: ['contextra-ingest', 'contextra-privacy', 'contextra-audit-export', 'contextra-avv-generator']; Kern in default-members: 13 |
| B-befunde | `befund_adr_status` | PASS | 0.0 | ADRs=[]; Proposed=[]; naechste freie Nummer<1000: 0001 |
| B-befunde | `befund_from_entropy` | FAIL | 0.2 | 4 Dateien mit from_entropy (K14) |
| B-befunde | `befund_maturity_claims` | INFO | 0.0 | stable=15 beta=12 |
| B-befunde | `befund_windows_flush` | INFO | 0.0 | windows-Pfad vorhanden=True, compile_error!=False |
| T1-produkt | `produkt_huelle_ohne_kern` | FAIL | 4.3 | 1 Produkt-Crates mit verbotener Huelle |
| T1-produkt | `produkt_sqlcipher_probe` | PASS | 363.6 | cipher=Some("4.19.0 community") fts5=true leak=false wrong_key_rejected=true rows=1 |
| T1-produkt | `produkt_crates_io_versionen` | PASS | 3.2 |  |
| 0-basis | `toolchain_versionen` | PASS | 0.9 |  |
| 0-basis | `cargo_fetch` | PASS | 0.9 |  |
| 0-basis | `cargo_tree_duplicates` | PASS | 0.7 |  |
| 1-statisch | `fmt_check` | FAIL | 7.0 |  |
| 1-statisch | `check_alle_targets` | PENDING | 0.0 |  |
| 1-statisch | `check_no_default_features` | PENDING | 0.0 |  |
| 1-statisch | `doc` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-adapt` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-adapt` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-agent` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-agent` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-arch-tests` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-arch-tests` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-audit-export` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-audit-export` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-avv-generator` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-avv-generator` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-bench` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-bench` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-checkpoint` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-checkpoint` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-cognition` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-cognition` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-core` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-core` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-crypto` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-crypto` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-db` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-db` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-durable-fs` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-durable-fs` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-engine` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-engine` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-graph` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-graph` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-infer-candle` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-infer-candle` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-infer-ollama` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-infer-ollama` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-infer-onnx` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-infer-onnx` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-ingest` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-ingest` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-kvcache` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-kvcache` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-license` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-license` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-mcp` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-mcp` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-mvcc` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-mvcc` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-ports` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-ports` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-privacy` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-privacy` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-py` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-py` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-rank` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-rank` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-router` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-router` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-sandbox` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-sandbox` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-simd` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-simd` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-store` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-store` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-sys` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-sys` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-testkit` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-testkit` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-text` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-text` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-types` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-types` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-vector` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-vector` | PENDING | 0.0 |  |
| 1-statisch | `clippy_lib__contextra-wire` | PENDING | 0.0 |  |
| 1-statisch | `clippy_alle_targets__contextra-wire` | PENDING | 0.0 |  |
| 1-statisch | `feature_matrix_hack` | PENDING | 0.0 |  |
| 1-statisch | `deny` | PENDING | 0.0 |  |
| 1-statisch | `audit` | PENDING | 0.0 |  |
| 1-statisch | `panic_flaeche_lib` | PENDING | 0.0 |  |
| 1-statisch | `unsafe_inventar` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-adapt` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-adapt` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-adapt` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-adapt` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-agent` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-agent` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-agent` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-agent` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-arch-tests` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-arch-tests` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-arch-tests` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-arch-tests` | PENDING | 0.0 |  |
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
| 2-tests | `test_lib__contextra-durable-fs` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-durable-fs` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-durable-fs` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-durable-fs` | PENDING | 0.0 |  |
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
| 2-tests | `test_lib__contextra-ingest` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-ingest` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-ingest` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-ingest` | PENDING | 0.0 |  |
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
| 2-tests | `test_gesamt_debug` | PENDING | 0.0 |  |
| 2-tests | `test_gesamt_release` | PENDING | 0.0 |  |
| 2-tests | `python_bindings_pytest` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-mvcc__mvcc_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-checkpoint__checkpoint_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-router__router_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-router__bandit_latency_gate` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-router__cost_routing_savings_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-graph__csr_traversal_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-graph__binary_edge_regression` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-crypto__crypto_benchmarks` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-crypto__deletion_proof_latency_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-infer-candle__kv_bridge_bench` | PENDING | 0.0 |  |
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
| 4-bench | `bench__contextra-bench__competitive_kv` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-bench__locomo` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-bench__long_mem_eval` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_detect` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_docx` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_email` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_odf` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_ole` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_pdf` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_pipeline` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_pptx` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_safe_archive` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_safe_xml` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_sheets` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_structured` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_text` | PENDING | 0.0 |  |
| 5-tief | `fuzz__ingest__fuzz_web` | PENDING | 0.0 |  |
| 5-tief | `fuzz__mcp__fuzz_jsonrpc_parsing` | PENDING | 0.0 |  |
| 5-tief | `fuzz__mcp__fuzz_prompt_injection_guard` | PENDING | 0.0 |  |
| 5-tief | `fuzz__text__fuzz_bm25_tokenize` | PENDING | 0.0 |  |
| 5-tief | `fuzz__text__fuzz_tokenizer_smoke` | PENDING | 0.0 |  |
| 5-tief | `fuzz__db__rrf_fusion` | PENDING | 0.0 |  |
| 5-tief | `fuzz__crypto__deletion_proof_tamper` | PENDING | 0.0 |  |
| 5-tief | `fuzz__crypto__deletion_proof_verify_external_fuzz` | PENDING | 0.0 |  |
| 5-tief | `fuzz__crypto__wal_hmac_chain_verify_fuzz` | PENDING | 0.0 |  |
| 5-tief | `fuzz__vector__fuzz_hnsw_persistence` | PENDING | 0.0 |  |
| 5-tief | `fuzz__vector__hnsw_insert_search` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_compaction_interleave` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_manifest_load` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_memtable_concurrent` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_recovery_arbitrary_state` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_sstable_binary_search` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_wal_replay` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__wal_mutation_chaos` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__wal_roundtrip` | PENDING | 0.0 |  |
| 5-tief | `fuzz__wire__fuzz_flatbuffers_ipc` | PENDING | 0.0 |  |
| 5-tief | `fuzz__wire__fuzz_jsonrpc` | PENDING | 0.0 |  |
| 5-tief | `fuzz__wire__fuzz_verification` | PENDING | 0.0 |  |
| 5-tief | `mutation_tier0_1` | PENDING | 0.0 |  |
| 5-tief | `loom_mvcc` | PENDING | 0.0 |  |
| 5-tief | `miri_vector_scalar` | PENDING | 0.0 |  |
| 5-tief | `asan_vector` | PENDING | 0.0 |  |
| 5-tief | `coverage` | PENDING | 0.0 |  |
