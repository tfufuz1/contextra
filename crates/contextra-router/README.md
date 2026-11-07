# contextra-router

Lyapunov Drift Control und Multi-Candidate Search Routing Engine.

Normativitätsklasse: **Experimentell `[E]`** (Ring 3, `stable`; §6.0, `capabilities.toml`).

`contextra-router` ist ein isolierter Baukasten zur intelligenten Zuweisung von Anfragen an geeignete Small Language Models (SLMs) und Dispatcher-Pfade.

## 1. Baukasten-Profil & Architektur
- **Ring:** Ring 3 (Applikations- und Koordinationsschicht / Peripherie)
- **Status:** 🟢 Modular entkoppelt (Baukasten-Architektur M1–M11)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]` strikt erzwungen
- **Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-wire`, `contextra-adapt`
- **Keine Aufwärtsabhängigkeiten:** Tests und Benchmarks laufen autark über injizierte Ports ohne Abhängigkeiten zu höheren Ringen oder konkreten Speicherkernen (`contextra-db`, `contextra-vector`, `contextra-graph`, `contextra-text`).

## 2. Garantien & Verträge
1. **Umwelt-Injektion (M4):**
   - Zeit über `contextra_ports::Clock`.
   - Dateizugriffe (z. B. persistierte Kalibrierungen) über `contextra_ports::Vfs`.
   - Pseudo-Zufall über deterministische Seeded RNGs (`SplitMix64`).
2. **Fail-Closed & Conformal Calibration (M5):**
   - Fail-closed bei ungültigen Profilen, nicht-finiten Relevanzwerten oder fehlender Community-Zuordnung.
   - Statistisch abgesicherte Konfidenzintervalle via Conformal Prediction.
3. **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `panic!`, `unwrap()` oder `expect()` in der Produktionslogik.

## 3. Öffentliche API
- **Router Engine:** [`RouterEngine`], [`DefaultRouterEngine`], [`RoutingDecision`], [`ConfidenceMetrics`]
- **SLM Profiles & Dispatch:** [`SlmProfile`], [`dispatch_to_slm`]
- **Outcomes & IDs:** [`RoutingOutcome`], [`DecisionId`], [`DecisionIdGenerator`]
- **Strategien:** [`RoutingStrategy`], [`ArmRegistry`]

## 4. Einbettungsbeispiel
Ein eigenständiges Minimalbeispiel findet sich unter `examples/simple_router.rs`:

```rust
use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_router::{RouterEngine, SlmProfile};
use contextra_types::{ContextChunk, ContextWindow, EntityId, Result, TokenBudget};
use std::sync::Arc;

// Initialisierung der RouterEngine mit injizierten Ports
let router = RouterEngine::new(
    search_provider,
    community_resolver,
    context_preparer,
    vec![profile],
    None,
);

let decision = router.route(&query_vector, "Suchanfrage").await?;
println!("Ausgewähltes SLM: {}", decision.profile.name);
```
