# Extraktion fuer `acta-text`

**Aufgabe:** Segmentierter Volltextindex (BM25 Festkomma), DE/EN-Normalisierung  
**Level:** L3 · **Budget:** 5000 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.2  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-gate, acta-vault, acta-seg  
**Umfang des Archivs:** 47 Dateien, 10850 Zeilen (inkl. Tests)  

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
| `text/src/bm25.rs` | 687 | float=49, panic=5 |
| `text/src/compat/purge.rs` | 508 | float=3, serde=4, hr6=3, hub=3, liste_a=1 |
| `text/src/compat/scanner.rs` | 310 | hr6=3, hub=3 |
| `text/src/domain/legal_de.rs` | 61 |  |
| `text/src/domain/medical_de.rs` | 61 |  |
| `text/src/domain/mod.rs` | 141 |  |
| `text/src/error.rs` | 45 |  |
| `text/src/inverted.rs` | 33 |  |
| `text/src/inverted/index_struct.rs` | 671 | float=6, serde=10, async=15 |
| `text/src/inverted/types.rs` | 55 | serde=1 |
| `text/src/lib.rs` | 91 | serde=2 |
| `text/src/memory_index.rs` | 508 | float=8, panic=7 |
| `text/src/morphology.rs` | 34 |  |
| `text/src/morphology/compound_splitter.rs` | 316 |  |
| `text/src/morphology/metrics.rs` | 33 | float=3 |
| `text/src/morphology/passthrough.rs` | 34 |  |
| `text/src/morphology/stopwords.rs` | 64 |  |
| `text/src/morphology/tests.rs` | 765 | float=2, unsafe=6, zeit_zufall=1, panic=2 |
| `text/src/morphology/trie.rs` | 65 |  |
| `text/src/posting_list.rs` | 585 | serde=6, panic=15 |
| `text/src/storage.rs` | 65 |  |
| `text/src/stream.rs` | 157 | float=2, async=3 |
| `text/src/tokenizer.rs` | 1025 | serde=8, zeit_zufall=1, panic=12, hr6=2 |
| `text/src/traits.rs` | 33 |  |
| `text/src/types.rs` | 81 | float=1, serde=1 |
| `text/src/wand.rs` | 818 | float=12, async=3 |
| `text/tests/bm25_candidate_stream.rs` | 388 | async=15, hub=4 |
| `text/tests/bm25_edge_cases.rs` | 171 | float=3 |
| `text/tests/bm25_reference_values.rs` | 253 | float=33 |
| `text/tests/domain_compound_integration.rs` | 143 |  |
| `text/tests/domain_vocab_quality.rs` | 458 | zeit_zufall=1 |
| `text/tests/golden/bm25_ranking.json` | 18 |  |
| `text/tests/golden/clock_reference.json` | 1 |  |
| `text/tests/golden/doc_id.bin` | 1 |  |
| `text/tests/golden/doc_id.json` | 1 |  |
| `text/tests/golden/doc_key.bin` | 1 |  |
| `text/tests/golden/posting_list_v2.bin` | 0 |  |
| `text/tests/golden/tokenizer_default.json` | 10 |  |
| `text/tests/golden/tokenizer_german.json` | 11 |  |
| `text/tests/golden_oracle_tests.rs` | 293 | float=2, serde=4, panic=38, std_fs=1 |
| `text/tests/pipeline_analyzer_tests.rs` | 160 | panic=4 |
| `text/tests/pipeline_identity_regression.rs` | 313 | async=6, panic=1, hub=10 |
| `text/tests/posting_roundtrip_tests.rs` | 146 | panic=6, hub=1 |
| `text/tests/proptest_bm25_invariants.rs` | 369 | async=1, panic=9, hub=10 |
| `text/tests/snapshot_oracle_proptest.rs` | 107 | panic=4, alt_spec=1 |
| `text/tests/wand_differential.rs` | 488 | float=7, async=2, panic=3, hub=9 |
| `text/tests/wand_tie_break_threshold.rs` | 272 | async=4, hub=4 |
