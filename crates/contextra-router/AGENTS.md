# AGENTS.md — `contextra-router`

> **Ring 3** · Klasse Rand `[R]` (SPEC-Katalog: Archiv, Root-§10 K5) · Reifegrad `stable` · Unsafe: nein · Rolle: Adaptives Query-/Modell-Routing (Bandits, Conformal Calibration, Dispatch).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, verifiziert und entkoppelt; Zahlen sind am Code validiert.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-router --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `adapt`, `ports`, `types`, `wire` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `wire`, `adapt` |
| Features | `bandit-routing`, `flow-corrected-thompson`, `egress-sherman-morrison`, `cloud-egress-guard` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- **Baukasten-Vertrag (M1, M6):** Dev-Kanten auf `db`, `graph`, `text`, `vector` wurden vollständig entfernt. Alle Tests und Benchmarks laufen autark über injizierte Mock-Provider.
- Normalkanten = Soll (`ports`, `types`, `wire`, `adapt`).
- Egress-Entscheidungen laufen über den Port `EgressPolicy`, ohne Bindung an `privacy`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `router/` (`dispatch_core`, `lifecycle`, `outcomes`) | `RouterEngine`, Lebenszyklus, Conformal Calibration, Drift-Watcher |
| `profile.rs`, `arm_registry.rs`, `routing_strategy.rs` | Modellprofile, Arm-Mapping, Kaskaden- & Bandit-Strategien |
| `dispatch.rs`, `fc_ts_dispatch.rs`, `transport.rs`, `outcome.rs` | JSON-RPC Dispatch, Flow-Corrected Thompson Sampling, Transport, Outcomes |

## 4. Verträge & Invarianten

1. Ein Modell wird nie gewählt, dessen Kontextgrenze das Fenster überschreitet; sonst vorher kompaktieren.
2. Kalibrierung wird nach jeder Interaktion aktualisiert (Conformal Prediction).
3. Zufall nur über `Rng`, Zeit über `Clock`, Persistenz über `Vfs`; Entscheidungen sind deterministisch reproduzierbar (`id_gen_determinism_test`).
4. Cloud-Egress ist standardmäßig aus und budgetiert (INV-EGRESS-BUDGET-1).

## 5. Ports

- **Konsumiert:** `Clock`, `Rng`, `Vfs`, `HybridSearchProvider`, `CommunityResolver`, `ContextPreparer`, `EgressPolicy` (Soll).
- **Implementiert:** `LocalDriftStatusProvider`.

## 6. Nebenläufigkeit & Performance

- Profile und Statistiken hinter kurzem `parking_lot::RwLock` bzw. atomarem `ArcSwap`; Hot-Reloads sind unterbrechungsfrei (Lock-Free Read).

## 7. Anti-Patterns

- SLM ohne Profil aufrufen.
- Dispatch ohne Egress-Prüfung.
- Uninjizierte `std::fs`-Aufrufe in Kernlogik.

## 8. Tests & Messung

```bash
cargo test -p contextra-router --locked
cargo clippy -p contextra-router --all-targets -- -D warnings
cargo run --example simple_router -p contextra-router
```

## 9. Offene Arbeit

| WP | Inhalt | Status |
|---|---|---|
| M6 | Dev-Kanten (db, graph, text, vector) entfernen | Erledigt |
| M4 | Vfs-Injektion für persistierte Kalibrierung | Erledigt |
| M10 | Eigenständiges Beispiel unter `examples/simple_router.rs` | Erledigt |
| 2.4 | Port `EgressPolicy` in `ports` konsolidieren | Folgepaket B2 |
