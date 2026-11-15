# Extraktion fuer `acta-graph`

**Aufgabe:** Graphindex (CSR) fuer die Abrufpipeline: Entitaets-/Relationstraversierung, Personalized PageRank, Pfad-RAG, Entitaetsextraktion, Ergebnis-Purger. **Nicht in SPEC 3.3: ADR-Vorschlag noetig** (siehe `docs/ADR-VORSCHLAG-graph.md`)  
**Level:** L3 (Vorschlag) · **Spezifikation:** Kap. 5.5, 9.4 (Erweiterung)  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-gate, acta-vault, acta-seg  
**Umfang des Archivs:** 89 Dateien, 19801 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `graph` (csr, ppr, path_rag, provider, provenance, purger, entity_extraction, storage, types) | CSR-Snapshot lesefrei (3.6), PPR-Strategien, Pfad-RAG als vierte/fuenfte Kandidatenliste in der RRF-Fusion (9.4); Ableitung D mit `KOrd`-Lineage (5.5); Loeschung nur ueber `ErasureLayer` (Segmentersatz) |
| `graph/src/compat` | Bruecken-Modul (HR-6): nur Referenz fuer Trait-Implementierungen |

Fixtures: nur textuelle Eingaben; Binaer-Goldens (`*.bin`) und Referenzcode der Altquelle sind nicht uebernommen und werden nach dem Formatentwurf neu erzeugt.

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 302 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 45 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 423 | tokio/async (G-NOTOKIO: Kern synchron) |
| unsafe | 6 | unsafe (G-UNSAFE: nur acta-sys) |
| panic | 801 | unwrap/expect/panic (G-LINT) |
| hr6 | 2 | include!/#[deprecated] (HR-6) |
| hub | 74 | Hub-Import ex types/ports (HR-2/HR-3) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `ahash`, `arc-swap`, `bincode`, `blake3`, `bytes`, `parking_lot`, `scc`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-graph/`) | LOC | Befunde |
|---|---:|---|
| `graph/src/arc_slice.rs` | 179 |  |
| `graph/src/compat/algo_error_bridge.rs` | 20 | hub=1 |
| `graph/src/compat/collection_mutation.rs` | 80 | hub=3 |
| `graph/src/compat/error.rs` | 55 | hub=1 |
| `graph/src/compat/graph_index/inherent_shims.rs` | 27 | float=1 |
| `graph/src/compat/graph_index/mod.rs` | 310 | float=8, async=12, hub=2 |
| `graph/src/compat/graph_index/mutations.rs` | 333 | async=5, hub=2 |
| `graph/src/compat/graph_index/traversal.rs` | 302 | float=17, async=2, hub=2 |
| `graph/src/compat/graph_index_ext.rs` | 11 |  |
| `graph/src/compat/llm_bridge.rs` | 2 |  |
| `graph/src/compat/provider.rs` | 310 | float=24, hub=1 |
| `graph/src/compat/purger.rs` | 139 | float=1, async=4, panic=6, hr6=2, hub=4 |
| `graph/src/compat/replay_hook.rs` | 2 |  |
| `graph/src/compat/storage_bridge.rs` | 90 | async=14, hub=1 |
| `graph/src/compat.rs` | 23 |  |
| `graph/src/csr/graph_index.rs` | 17 |  |
| `graph/src/csr/graph_persist.rs` | 283 | float=4, serde=8, async=4, hub=2 |
| `graph/src/csr/graph_read/communities.rs` | 88 | async=1 |
| `graph/src/csr/graph_read/mod.rs` | 183 | async=4 |
| `graph/src/csr/graph_read/pagerank.rs` | 147 | float=10, async=4 |
| `graph/src/csr/graph_read/path_rag.rs` | 83 | float=3, async=1 |
| `graph/src/csr/graph_write/edge_mutation.rs` | 227 | float=5, async=4 |
| `graph/src/csr/graph_write/entity.rs` | 44 |  |
| `graph/src/csr/graph_write/hyperedge.rs` | 401 | float=1, async=2 |
| `graph/src/csr/graph_write/mod.rs` | 208 | hub=1 |
| `graph/src/csr/graph_write/tombstone.rs` | 134 | async=1, hub=1 |
| `graph/src/csr/inner/compaction.rs` | 259 | float=2 |
| `graph/src/csr/inner/estimate.rs` | 269 | float=2 |
| `graph/src/csr/inner/mod.rs` | 185 | float=2 |
| `graph/src/csr/inner/reinforcement.rs` | 169 |  |
| `graph/src/csr/path_graph.rs` | 212 | float=8 |
| `graph/src/csr/tests/basic_tests.rs` | 793 | float=9, async=42, panic=87, hub=2 |
| `graph/src/csr/tests/bitemporal_tests.rs` | 581 | float=1, async=27, panic=20, hub=2 |
| `graph/src/csr/tests/graph_index_tests.rs` | 697 | float=2, serde=4, async=31, panic=45, hub=2 |
| `graph/src/csr/tests/mutation_sequence_tests.rs` | 122 | async=2, panic=12, hub=2 |
| `graph/src/csr/tests/persistence_tests.rs` | 832 | async=37, panic=108, hub=2 |
| `graph/src/csr/tests.rs` | 22 |  |
| `graph/src/csr/types.rs` | 136 | float=8, serde=6 |
| `graph/src/csr/visibility.rs` | 100 |  |
| `graph/src/csr.rs` | 36 |  |
| `graph/src/entity_extraction.rs` | 323 | float=6 |
| `graph/src/error.rs` | 118 | float=1 |
| `graph/src/graph_view.rs` | 34 | float=2, async=2 |
| `graph/src/lib.rs` | 159 | unsafe=1 |
| `graph/src/path_rag/k_path.rs` | 336 | float=2 |
| `graph/src/path_rag/mod.rs` | 484 | float=43 |
| `graph/src/path_rag/snapshot.rs` | 227 | float=2 |
| `graph/src/path_rag/tests.rs` | 587 | float=10, panic=8 |
| `graph/src/ppr/cost.rs` | 273 | float=14, panic=1 |
| `graph/src/ppr/dense.rs` | 411 | float=9 |
| `graph/src/ppr/forward_push.rs` | 278 | float=23 |
| `graph/src/ppr/mod.rs` | 385 | float=10 |
| `graph/src/ppr/shadow_hook.rs` | 122 | float=9, panic=1 |
| `graph/src/ppr/snapshot.rs` | 147 | float=1 |
| `graph/src/ppr/tests/algo_tests.rs` | 1205 | float=12, async=40, panic=122, hub=2 |
| `graph/src/ppr/tests/context_tests.rs` | 858 | float=5, async=25, panic=78, hub=5 |
| `graph/src/ppr/tests/golden_tests.rs` | 364 | float=7, async=4, panic=25, hub=1 |
| `graph/src/ppr/tests.rs` | 20 |  |
| `graph/src/ppr/types.rs` | 89 | float=3 |
| `graph/src/ppr_stream.rs` | 172 | float=4, async=5, hub=1 |
| `graph/src/provenance.rs` | 198 | serde=1, panic=2 |
| `graph/src/provider.rs` | 15 |  |
| `graph/src/purger.rs` | 15 |  |
| `graph/src/storage.rs` | 129 | async=8, panic=19 |
| `graph/src/test_doubles.rs` | 215 | async=14, hub=2 |
| `graph/src/types.rs` | 395 | float=10, serde=16 |
| `graph/tests/algorithm_test_doubles.rs` | 67 | float=5, panic=1, hub=1 |
| `graph/tests/budgeted_k_path_hub_entity.rs` | 176 | float=4, panic=5, hub=1 |
| `graph/tests/data/entity_gold.json` | 318 |  |
| `graph/tests/entity_extraction_edge_cases.rs` | 151 | panic=2, hub=1 |
| `graph/tests/entity_extraction_tests.rs` | 97 | panic=1, hub=1 |
| `graph/tests/error_mapping.rs` | 76 | hub=1 |
| `graph/tests/fixtures/karate_club.json` | 83 |  |
| `graph/tests/graph_collection_mutation_trait_test.rs` | 81 | panic=3, hub=2 |
| `graph/tests/graph_persist_robustness.rs` | 265 | serde=10, async=23, panic=8, hub=2 |
| `graph/tests/integration_graph.rs` | 138 | async=6, panic=20, hub=2 |
| `graph/tests/path_rag_at_snapshot.rs` | 357 | async=21, panic=55, hub=2 |
| `graph/tests/persistence_test.rs` | 149 | async=6, panic=21, hub=2 |
| `graph/tests/ppr_alloc_test.rs` | 135 | async=2, unsafe=5, panic=2, hub=1 |
| `graph/tests/ppr_candidate_stream.rs` | 365 | float=1, async=23, panic=30, hub=2 |
| `graph/tests/ppr_cost_model_selection.rs` | 62 | hub=1 |
| `graph/tests/ppr_reference_validation.rs` | 302 | float=9, async=4, panic=12, hub=2 |
| `graph/tests/ppr_snapshot_isolation.rs` | 400 | async=12, panic=44, hub=2 |
| `graph/tests/ppr_strategy_equivalence.rs` | 114 | float=1, async=2, panic=13, hub=2 |
| `graph/tests/proptest_csr_invariants.rs` | 382 | async=10, panic=26, hub=2 |
| `graph/tests/provider_tests.rs` | 142 | float=1, panic=6, hub=1 |
| `graph/tests/reader_writer_concurrency.rs` | 81 | async=9, panic=5, hub=2 |
| `graph/tests/source_doc_ids_populated.rs` | 170 | async=6, panic=13, hub=2 |
| `graph/tests/storage_port_conformance.rs` | 20 | async=4 |
