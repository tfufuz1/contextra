# AGENTS.md — `contextra-router`

> **Ring 3** · Klasse Rand `[R]` (SPEC-Katalog: Archiv, Root-§10 K5) · Reifegrad `stable` · Unsafe: nein · Rolle: Adaptives Query-/Modell-Routing (Bandits, Conformal Calibration, Dispatch).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-router --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `adapt`, `ports`, `types`, `wire` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `wire`, `adapt` |
| Features | `bandit-routing`, `flow-corrected-thompson`, `egress-sherman-morrison`, `cloud-egress-guard` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- Normalkanten = Soll. `capabilities.toml` nennt zusätzlich `privacy` (nicht in Cargo). Dev-Kanten `db`, `graph`, `text`, `vector` (M6).
- Egress soll über den Port `EgressPolicy` laufen (WP 2.4), nicht über eine Bindung an `privacy`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `router/` (`dispatch_core`, `lifecycle`, `outcomes`) | `RouterEngine` |
| `profile.rs`, `arm_registry.rs`, `routing_strategy.rs` | Profile, Arme, Strategien |
| `dispatch.rs`, `fc_ts_dispatch.rs`, `transport.rs`, `outcome.rs` | Dispatch, Transport, Ergebnisse |

## 4. Verträge & Invarianten

1. Ein Modell wird nie gewählt, dessen Kontextgrenze das Fenster überschreitet; sonst vorher kompaktieren.
2. Kalibrierung wird nach jeder Interaktion aktualisiert (Conformal).
3. Zufall nur über `Rng`, Zeit über `Clock`; Entscheidungen sind reproduzierbar (`id_gen_determinism_test`).
4. Cloud-Egress ist standardmäßig aus und budgetiert (INV-EGRESS-BUDGET-1).

## 5. Ports

Konsumiert `Clock`, `Rng`, `IdGen`, `EgressPolicy` (Soll).

## 6. Nebenläufigkeit & Performance

- Profile/Statistiken hinter kurzem `parking_lot::RwLock`; Updates synchron und kurz.

## 7. Anti-Patterns

- SLM ohne Profil aufrufen.
- Dispatch ohne Egress-Prüfung.

## 8. Tests & Messung

```bash
cargo test -p contextra-router --locked
cargo clippy -p contextra-router --all-targets -- -D warnings
```

- 7 Testdateien; Regret-Tests für Bandits.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 2.4 | `EgressPolicy` einführen und nutzen |
| M6 | Dev-Kanten entfernen |
