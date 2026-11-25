# AGENTS.md — contextra-cognition
> Ring 3 · stable · Quelle: capabilities.toml · Spec: K.6, III.15

## 1. Zweck

Implementiert die Kognitions-Pipeline für Gedächtniskonsolidierung, Near-Duplicate-Detection, Turn-Segmentierung, Kontextkomprimierung (`ContextCompactor`) und synthetische Zusammenfassungsdurchläufe (`SynthesisPhaseResult`). Steuert den Konsolidierungs-Lebenszyklus und Wartungszeitpläne (`MaintenanceScheduler`).

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | `#![forbid(unsafe_code)]`, Re-Exports, Facade für Konsolidierungspässe |
| `src/context_compaction/` | `ContextCompactor`, `ConsolidationSession`, `CompactedContext` & Cleanup-Routinen |
| `src/memory_consolidation.rs` | `ConsolidationEngine`, `detect_near_duplicates()`, Turn-Segmentierung |
| `src/transitivity_veto.rs` | `validate_transitivity_veto()`, `filter_candidates_with_transitivity_veto()` |
| `src/consolidation_executor.rs` | Executer für Konsolidierungs-Passes & Decay-Eviction nach Summary-Commit |
| `src/consolidation_locks.rs` | `ConsolidationNodesGuard` zur Vermeidung paralleler Konsolidierung gleicher Knoten |
| `src/maintenance_scheduler.rs` | `MaintenanceScheduler` mit `with_clock()` Injection für Hintergrundaufgaben |
| `src/maintenance_config.rs` | `MaintenanceConfig` (P10) für Decay-, Percolation- & Konsolidierungs-Parameter |
| `src/aggregation_phase.rs` | `AggregationPhaseResult`, Vorbereitung strukturierter Gedächtniscluster |
| `src/synthesis_phase.rs` | `run_synthesis_pass()`, LLM-gestützte Generierung synthetischer Memories |
| `src/context.rs` | `ContextManager`, `SpatialFence` & Token-Budget-Accounting |
| `src/graph_sink.rs` | `GraphSinkAdapter` zur Rückspeicherung konsolidierter Relationen |
| `src/leanrag_input.rs` | Vorbereitung strukturierter Prompts für Konsolidierungs-LLMs |
| `src/semantic_aggregation_facade.rs` | Fassade für semantische Aggregation über Vektor-Clustering |

## 3. Invarianten

- **INV-DEDUP-TRANSITIVITY-1:** Dedup-Clustering wendet bei Kandidatenpaaren ein Transitivitäts-Veto an, um falsch-positive Verschmelzungen bei indirekter Ähnlichkeit zu verhindern.
- **P28 Determinismus:** Zeitstempel und Zeitabstände in `MaintenanceScheduler` werden ausschließlich über den `Clock`-Port (`with_clock`) injiziert.
- **INV-SUMMARY-DECAY:** Der `AdaptiveDecayController` wird in `consolidation_executor` erst nach Bestätigen der synthetischen Zusammenfassungen angewendet.

## 4. Verboten / Anti-Patterns

- **Keine direkte SystemTime in Schedulern:** Zeit-Injektion via `Arc<dyn Clock>` ist Pflicht für deterministische Tests.
- **Keine Konsolidierung ohne Node-Lock Guard:** Konsolidierung gleicher Node-IDs ohne `ConsolidationNodesGuard` führt zu Datenrennen.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

Arbeitet asynchron auf Layer 3 über `contextra-engine`. LLM-Konsolidierungsdurchläufe dauern lang und dürfen niemals unter aktiven Key-Locks oder ReadGuards gehalten werden. Knoten-Sperren für Konsolidierung werden über `ConsolidationNodesGuard` verwaltet.

## 6. Verifikation

```bash
cargo test -p contextra-cognition
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-cognition
cargo xtask determinism-check
cargo xtask check-unsafe-islands
```

## 7. Bekannte Lücken / SOLL

- `filter_candidates_with_transitivity_veto` ist im Code vollständig implementiert und in `memory_consolidation.rs` integriert.

