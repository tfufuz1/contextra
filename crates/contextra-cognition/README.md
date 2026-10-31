# contextra-cognition

Hintergrundprozesse für Gedächtniskonsolidierung (Memory Consolidation), Kontextfenster-Kompaktierung (Context Compaction), semantische Aggregation (LeanRAG Stage 3) und periodische Wartungsplanung (Maintenance Scheduling).

## Status und Einordnung

- **Ring:** 3 (Orchestrierungs- und Kognitionsschicht)
- **Reifestatus:** `experimental`
- **Scope Contextra v1:** Laut AGENTS.md §9 sind Kognition und Konsolidierung über Schlafzyklen Nicht-Ziele von v1. Die Scope-Entscheidung über den Einbezug oder die Archivierung ist in **ADR 0009** (`docs/decisions/0009-scope-contextra-v1-14-oder-35-crates.md`) als Status `Proposed` / offen dokumentiert. Das Crate dient dem Nachweis und der Dokumentation bestehender Fähigkeiten.

## Architektur und Modulübersicht

Die Quelltexte unter `src/` gliedern sich in folgende Module:

| Modul | Datei | Beschreibung |
|---|---|---|
| `aggregation_phase` | `src/aggregation_phase.rs` | Semantische Aggregationsphase (LeanRAG Stage 3) mit Typ-Denoising, Diagonal-GMM Clustering, Alpha-Knoten-Synthese und Super-Hyperkanten. |
| `consolidation_executor` | `src/consolidation_executor.rs` | Koordination von Hintergrund-Konsolidierungsworkern, Sleep-Cycles und Kaskaden-Invalidierung. |
| `consolidation_locks` | `src/consolidation_locks.rs` | RAII-Sperren zur thread-sicheren Kaskaden-Invalidierung und Verhinderung von Race-Conditions während Konsolidierungsläufen. |
| `context` | `src/context.rs` | `ContextManager` zur autonomen Kontextverwaltung, Budgetierung und räumlichen Abgrenzung (`SpatialFence`). |
| `context_compaction` | `src/context_compaction/` | Transaktions- und Dialogkompaktierung (`ContextCompactor`) mit Truncate-, Summary- und Selective-Strategien sowie Bereinigung verwaister Intents. |
| `graph_sink` | `src/graph_sink.rs` | Implementierung des `SuperEdgeSink`-Traits (`CsrGraphSuperEdgeSink`) zur Persistierung erzeugter Super-Hyperkanten in den CSR-Graphen. |
| `leanrag_input` | `src/leanrag_input.rs` | Deterministische Extraktion und Vorbereitung von Graph- und Vektordaten für LeanRAG Stage 3 (`build_leanrag_inputs`). |
| `maintenance_config` | `src/maintenance_config.rs` | Konfigurationsstrukturen für Wartungsintervalle, Verfall-Bereinigung, Perkolation und Konsolidierungsschwellen (`MaintenanceConfig`). |
| `maintenance_scheduler` | `src/maintenance_scheduler.rs` | Zentraler Planer (`MaintenanceScheduler`) für periodische Hintergrund-Wartungsarbeiten mit Clock-Port-Injektion und WAL-Intent-Protokollierung. |
| `memory_consolidation` | `src/memory_consolidation.rs` | Structural Pass der Gedächtniskonsolidierung: Erkennung von Beinahe-Duplikaten (`detect_near_duplicates`), Sliding-Window-Clusterung und Intake vorverdichteter Eingaben (`intake_pre_condensed_inputs`). |
| `semantic_aggregation_facade` | `src/semantic_aggregation_facade.rs` | Fassadenfunktion `consolidate_semantic_hyperedges` zur bequemen Ausführung der Hyperkanten-Konsolidierung. |
| `synthesis_phase` | `src/synthesis_phase.rs` | Generativer Synthesis Pass (`run_synthesis_pass`) zur LLM-gestützten Abstraktion von Gesprächssegmenten. |
| `transitivity_veto` | `src/transitivity_veto.rs` | Schutzmechanismus gegen fehlerhafte transitive Duplikat-Verschmelzungen (`validate_transitivity_veto`). |

## Invarianten

1. **Zero-Panic-Doktrin (P2):** Kein `unwrap`, `expect` oder `panic!` im Produktionscode. Alle Fehler werden über `ContextraError` typisiert zurückgegeben. Teilschritte im Wartungsplaner sind fehlertolerant isoliert.
2. **Ports für Nicht-Determinismus (Regel 3):**
   - Zeitmessung und Ticks erfolgen über den `Clock`-Port (in Tests deterministisch über `ManualClock`).
   - Pseudo-Zufall erfolgt über den `Rng`-Port (`contextra_ports::SeededRng`).
   - IDs werden deterministisch oder über generierte Hashes abgeleitet.
3. **Mandantenisolierung (VETO-F10):** Strikte Trennung nach `TenantId`. Vorverdichtete Eingaben und Konsolidierungsschritte überschreiten niemals Mandantengrenzen.
4. **P12-Kostenschutz & Budgetierung:**
   - Maximale Anzahl an LLM-Aufrufen pro Konsolidierungszyklus ist strikt gedeckelt (`max_llm_calls_per_cycle`).
   - Maximaler Speicherbedarf bei Compaction wird vorab via `check_compaction_budget` validiert.
5. **Kein Unsafe-Code:** `#![forbid(unsafe_code)]` wird auf Crate-Ebene strikt erzwungen.

## Nicht-Ziele (SPEC §2.3, AGENTS.md §9)

Folgende Fähigkeiten sind für Contextra v1 explizit als Nicht-Ziele definiert:
- Graph-Reasoning und autonome Inferenzketten im Anfragepfad.
- Online-Adaption und Modellgewichtsaktualisierungen während der Laufzeit.
- Schlafzyklus-Konsolidierung als Pflichtbestandteil des Kern-Workspaces.

## Nutzungsbeispiel

Das folgende Beispiel demonstriert die Konfiguration und Prüfung des Compaction-Speicherbudgets sowie die Erstellung einer Aggregationskonfiguration:

```rust
use contextra_cognition::aggregation_phase::{check_compaction_budget, AggregationConfig};
use contextra_types::Result;

fn main() -> Result<()> {
    // Standardkonfiguration erzeugen und validieren
    let config = AggregationConfig {
        max_compaction_peak_memory_mb: 256,
        clustering_tau_threshold: 0.35,
        gmm_deterministic_seed: 42,
        max_llm_calls_per_cycle: 5,
        ..Default::default()
    };
    config.validate()?;

    // Speicherbedarf (z. B. 128 MB) vorab prüfen
    let estimated_bytes = 128 * 1024 * 1024;
    check_compaction_budget(estimated_bytes, &config)?;

    Ok(())
}
```
