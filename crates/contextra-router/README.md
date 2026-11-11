# contextra-router

Lyapunov Drift Control und Multi-Candidate Search Routing Engine.

Normativitätsklasse: **Experimentell `[E]`** (Ebene E2, `stable`; §6.0, `capabilities.fragment.toml`).

`contextra-router` ist ein isolierter Baukasten zur intelligenten Zuweisung von Anfragen an geeignete Small Language Models (SLMs) und Dispatcher-Pfade.

## 1. Baukasten-Profil & Architektur
- **Ebene:** Ebene 2 (Integration/Applikation)
- **Status:** 🟢 Modular entkoppelt (WP-RT-1)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]` strikt erzwungen
- **Kern-Closure:** `contextra-types`, `contextra-ports`, `contextra-adapt`
- **Optionale Closure:** `contextra-wire`, `tokio`, `shlex` (nur hinter `subprocess-dispatch`)
- **Keine Aufwärtsabhängigkeiten:** Tests und Benchmarks laufen autark über injizierte Ports ohne Abhängigkeiten zu höheren Ringen oder konkreten Speicherkernen (`contextra-db`, `contextra-vector`, `contextra-graph`, `contextra-text`).

## 2. Garantien & Verträge
1. **Umwelt-Injektion (M4):**
   - Zeit über `contextra_ports::Clock`.
   - Dateizugriffe (z. B. persistierte Kalibrierungen) über die Naht `CalibrationSource` (`src/seam.rs`).
   - Pseudo-Zufall über deterministische Seeded RNGs (`SplitMix64`) bzw. `contextra_adapt::Rng`.
2. **Fail-Closed & Conformal Calibration (M5):**
   - Fail-closed bei ungültigen Profilen, nicht-finiten Relevanzwerten oder fehlender Community-Zuordnung.
   - Statistisch abgesicherte Konfidenzintervalle via Conformal Prediction.
3. **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `panic!`, `unwrap()` oder `expect()` in der Produktionslogik.

## 3. Öffentliche API
- **Router Engine:** [`RouterEngine`], [`DefaultRouterEngine`], [`RoutingDecision`], [`ConfidenceMetrics`]
- **Adoptions-Fassade:** [`RouterEngineBuilder`]
- **SLM Profiles & Dispatch:** [`SlmProfile`], [`dispatch_to_slm`]
- **Outcomes & IDs:** [`RoutingOutcome`], [`DecisionId`], [`DecisionIdGenerator`]
- **Strategien:** [`RoutingStrategy`], [`ArmRegistry`]
- **Kalibrierungs-Naht:** [`CalibrationSource`], [`NoCalibrationSource`], [`StdFsCalibrationSource`], [`VfsCalibrationSource`]

## 4. Einbettungsbeispiel
Ein eigenständiges Minimalbeispiel mit dem Fluent Builder findet sich unter `examples/builder_router.rs`:

```rust
use contextra_router::{RouterEngineBuilder, SlmProfile};
use std::sync::Arc;

let router = RouterEngineBuilder::new()
    .search_provider(search_provider)
    .community_resolver(community_resolver)
    .context_preparer(context_preparer)
    .add_profile(profile)
    .build()?;

let decision = router.route(&query_vector, "Suchanfrage").await?;
println!("Ausgewähltes SLM: {}", decision.profile.name);
```
