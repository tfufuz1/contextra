# Extraktion fuer `acta-text`

**Aufgabe:** Segmentierter Volltextindex (BM25 Festkomma), DE/EN-Normalisierung  
**Level:** L3 · **Budget:** 5000 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.2  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-gate, acta-vault, acta-seg  
**Umfang des Archivs:** 60 Dateien, 14272 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `text` | BM25 (9.2): f32/f64 durch Q32.32 ersetzen; DE/EN-Stemmer, Sichtklassen-Statistik neu |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 131 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 36 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 49 | tokio/async (G-NOTOKIO: Kern synchron) |
| unsafe | 6 | unsafe (G-UNSAFE: nur acta-sys) |
| zeit_zufall | 3 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 106 | unwrap/expect/panic (G-LINT) |
| std_fs | 1 | std::fs direkt (G-FS: nur acta-fs) |
| hr6 | 8 | include!/#[deprecated] (HR-6) |
| hub | 44 | Hub-Import ex types/ports (HR-2/HR-3) |
| alt_spec | 1 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |
| liste_a | 1 | verbotene Formulierung (1.6, Liste A) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `bincode`, `blake3`, `bytes`, `futures`, `itoa`, `parking_lot`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-text/`) | LOC | Befunde |
|---|---:|---|
| `ports/src/text_index.rs` | 276 | panic=3, async=26, serde=1 |
| `text/fuzz/fuzz_targets/fuzz_bm25_tokenize.rs` | 21 |  |
| `text/fuzz/fuzz_targets/fuzz_tokenizer_smoke.rs` | 36 |  |
| `text/src/bm25.rs` | 686 | panic=5, float=49 |
| `text/src/compat/purge.rs` | 507 | async=15, float=3, serde=4, alt_spec=3 |
| `text/src/compat/scanner.rs` | 309 | async=6, alt_spec=3 |
| `text/src/domain/legal_de.rs` | 60 |  |
| `text/src/domain/medical_de.rs` | 60 |  |
| `text/src/domain/mod.rs` | 140 |  |
| `text/src/error.rs` | 44 |  |
| `text/src/inverted.rs` | 32 |  |
| `text/src/inverted/index_struct.rs` | 670 | async=15, float=6, serde=10 |
| `text/src/inverted/types.rs` | 54 | serde=1 |
| `text/src/lib.rs` | 90 | async=1, serde=2 |
| `text/src/memory_index.rs` | 507 | panic=7, float=8 |
| `text/src/morphology.rs` | 33 |  |
| `text/src/morphology/compound_splitter.rs` | 315 |  |
| `text/src/morphology/metrics.rs` | 32 | float=3 |
| `text/src/morphology/passthrough.rs` | 33 |  |
| `text/src/morphology/stopwords.rs` | 63 |  |
| `text/src/morphology/tests.rs` | 764 | panic=2, float=2, unsafe=6 |
| `text/src/morphology/trie.rs` | 64 |  |
| `text/src/posting_list.rs` | 584 | panic=15, serde=6 |
| `text/src/storage.rs` | 64 |  |
| `text/src/stream.rs` | 156 | async=3, float=2 |
| `text/src/tokenizer.rs` | 1024 | panic=12, serde=8 |
| `text/src/traits.rs` | 32 |  |
| `text/src/types.rs` | 80 | float=1, serde=1 |
| `text/src/wand.rs` | 817 | async=14, float=12 |
| `text/tests/bm25_candidate_stream.rs` | 387 | async=31 |
| `text/tests/bm25_edge_cases.rs` | 170 | float=3 |
| `text/tests/bm25_reference_values.rs` | 252 | float=33 |
| `text/tests/bm25f_field_weighting.rs` | 167 | panic=3, float=1 |
| `text/tests/concurrent_metadata.rs` | 215 | panic=2, async=24 |
| `text/tests/core_smoke.rs` | 180 | async=13 |
| `text/tests/docid_128_tombstone_resolution.rs` | 263 | async=18 |
| `text/tests/domain_compound_integration.rs` | 142 |  |
| `text/tests/domain_vocab_quality.rs` | 457 |  |
| `text/tests/fault_injection_limits_test.rs` | 82 | panic=3 |
| `text/tests/golden/bm25_ranking.json` | 0 |  |
| `text/tests/golden/clock_reference.json` | 0 |  |
| `text/tests/golden/doc_id.json` | 0 |  |
| `text/tests/golden/tokenizer_default.json` | 0 |  |
| `text/tests/golden/tokenizer_german.json` | 0 |  |
| `text/tests/golden_oracle_tests.rs` | 292 | panic=38, float=2, serde=4, std_fs=1 |
| `text/tests/index_purge_proptests.rs` | 424 | panic=3, async=22 |
| `text/tests/lib_scorer_lifecycle_test.rs` | 171 | panic=4, async=18 |
| `text/tests/pipeline_analyzer_tests.rs` | 159 | panic=4 |
| `text/tests/pipeline_identity_regression.rs` | 312 | panic=1, async=22 |
| `text/tests/posting_roundtrip_tests.rs` | 145 | panic=7 |
| `text/tests/proptest_bm25_invariants.rs` | 368 | panic=9, async=18 |
| `text/tests/purge_error_injection_tests.rs` | 257 | panic=1, async=22 |
| `text/tests/snapshot_concurrency_test.rs` | 102 | panic=6 |
| `text/tests/snapshot_oracle_proptest.rs` | 106 | panic=4 |
| `text/tests/text_scanner_purger_tests.rs` | 372 | async=33 |
| `text/tests/text_search_at_wrappers.rs` | 383 | async=24 |
| `text/tests/tombstone_read_your_writes.rs` | 305 | async=20 |
| `text/tests/tombstone_update.rs` | 250 | async=22 |
| `text/tests/wand_differential.rs` | 487 | panic=3, async=18, float=7 |
| `text/tests/wand_tie_break_threshold.rs` | 271 | async=20 |
