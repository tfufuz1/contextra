# Extraktion fuer `_reserve/graph-ext`

**Aufgabe:** Erweiterungen des Graphindex: Hyperkanten, Community (Leiden), Kaskaden, Diffusion (APPRH, TL-HFD), Perkolation, Session-DAG, Kantenverstaerkung  
**Level:** Reserve · **Spezifikation:** nicht in SPEC; Aufnahme nur per ADR  
**Interne Abhaengigkeiten (Soll):** acta-graph  
**Umfang des Archivs:** 52 Dateien, 13188 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `graph` (apprh, tl_hfd, hyperedge*, community*, cascade, percolation, edge_reinforcement*, session_dag, consistency_enforcement, text_gen) | je Baustein einzeln nach Messbericht und ADR in `acta-graph` heben; `session_dag` und `edge_reinforcement` sind Lern-/Agentenlogik (Nicht-Ziel 1.4, Nutzungsdaten duerfen keine P-Daten ableiten) |



## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 191 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 52 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 148 | tokio/async (G-NOTOKIO: Kern synchron) |
| panic | 354 | unwrap/expect/panic (G-LINT) |
| hub | 42 | Hub-Import ex types/ports (HR-2/HR-3) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `ahash`, `arc-swap`, `bincode`, `blake3`, `bytes`, `parking_lot`, `scc`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/_reserve/graph-ext/`) | LOC | Befunde |
|---|---:|---|
| `src/apprh/diffusion.rs` | 187 | float=6 |
| `src/apprh/error.rs` | 31 |  |
| `src/apprh/gate_monitor.rs` | 281 | float=5, async=1 |
| `src/apprh/mod.rs` | 161 | float=4, async=2, panic=5 |
| `src/apprh/params.rs` | 127 | float=1 |
| `src/apprh/selector.rs` | 226 | float=1 |
| `src/apprh/shadow.rs` | 152 | float=13 |
| `src/cascade.rs` | 1100 | serde=6, async=30, panic=61, hub=1 |
| `src/community/tests.rs` | 839 | serde=4, async=24, panic=79, hub=1 |
| `src/community.rs` | 642 | float=10, serde=3, async=1 |
| `src/consistency_enforcement.rs` | 494 | serde=3, panic=7 |
| `src/edge_reinforcement.rs` | 181 | float=16, serde=1 |
| `src/edge_reinforcement_buffer.rs` | 223 | float=2, async=2, panic=4, hub=1 |
| `src/hyperedge.rs` | 807 | float=8, serde=20, panic=7 |
| `src/hyperedge_suggest.rs` | 197 | float=1, async=1, hub=1 |
| `src/percolation.rs` | 343 | float=12, serde=2, async=8, panic=9, hub=1 |
| `src/session_dag.rs` | 1016 | serde=12, async=14, panic=33, hub=1 |
| `src/text_gen.rs` | 8 |  |
| `src/tl_hfd/diffusion.rs` | 256 | float=11 |
| `src/tl_hfd/error.rs` | 27 |  |
| `src/tl_hfd/lovasz.rs` | 210 | float=6 |
| `src/tl_hfd/mod.rs` | 109 | float=2, async=2, panic=3 |
| `src/tl_hfd/params.rs` | 150 | float=3 |
| `src/tl_hfd/shadow.rs` | 170 | float=16 |
| `tests/apprh_determinism_permutation.rs` | 154 | float=7, panic=3, hub=1 |
| `tests/apprh_gate_monitor.rs` | 342 | float=7, panic=6, hub=1 |
| `tests/apprh_selector_shadow_to_production.rs` | 278 | float=5, hub=1 |
| `tests/apprh_tests.rs` | 257 | float=6, hub=1 |
| `tests/community_hyperedges_included_flag.rs` | 149 | async=2, panic=10, hub=2 |
| `tests/dangling_nodes_audit_test.rs` | 289 | float=7, async=8, panic=8, hub=2 |
| `tests/diffusion_shadow_gate_thresholds.rs` | 101 | panic=1, hub=1 |
| `tests/hyperedge_cascade_crash_recovery.rs` | 182 | async=2, panic=18, hub=2 |
| `tests/hyperedge_cascade_fanout.rs` | 89 | async=2, panic=4, hub=1 |
| `tests/hyperedge_cascade_queue_persistence.rs` | 83 | async=2, panic=7, hub=2 |
| `tests/hyperedge_compact_race.rs` | 121 | panic=4, hub=1 |
| `tests/hyperedge_memory_budget.rs` | 114 | panic=2, hub=1 |
| `tests/hyperedge_payload_sharing.rs` | 43 | hub=1 |
| `tests/hyperedge_persistence_survives_restart.rs` | 121 | async=2, panic=8, hub=2 |
| `tests/hyperedge_suggest.rs` | 197 | async=5, panic=10, hub=2 |
| `tests/hyperedge_suggest_llm_test.rs` | 138 | float=1, async=7, panic=5, hub=2 |
| `tests/hyperedge_suggest_tests.rs` | 385 | float=4, async=5, panic=8, hub=2 |
| `tests/ip08_resource_tracker_test.rs` | 57 | async=2, panic=6, hub=1 |
| `tests/leanrag_cascade_through_superedge.rs` | 345 | async=10, panic=15, hub=1 |
| `tests/leanrag_graph_sink_batch_commit.rs` | 155 | hub=1 |
| `tests/leiden_reference_validation.rs` | 183 | float=4, serde=1, async=6, panic=9, hub=2 |
| `tests/proptest_tl_hfd_diffusion.rs` | 324 | float=12, panic=4, hub=1 |
| `tests/proptest_tl_hfd_lovasz.rs` | 292 | float=9, hub=1 |
| `tests/star_expansion_equals_clique.rs` | 56 | float=4, hub=1 |
| `tests/stress_dag.rs` | 120 | async=6, panic=4 |
| `tests/supersession_characterization_tests.rs` | 145 | async=4, panic=14, hub=2 |
| `tests/tl_hfd_default_flip_gate.rs` | 192 | float=3 |
| `tests/tlhfd_shadow_mode_parity.rs` | 339 | float=5, hub=1 |
