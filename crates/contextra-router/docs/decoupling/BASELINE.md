# Baseline contextra-router (WP-RT-1.0)

Erhebungsstand: 11.11.2026 auf HEAD 9a287ace4 / origin/main

## 1. Ausgangsbefunde
- Defekt R1: Lyapunov-Importe waren auf main bereits vorhanden, ungenutzter Import `contextra_adapt::Rng` in `src/router/mod.rs` wurde bereinigt.
- Defekt R2: Features `bandit-routing` und `exp-routing-strategy` scheitern ohne Default-Features mit E0412 (`BanditProfileState` fehlt in `contextra_adapt`, da adapt-Feature `bandit-routing` nicht weitergeleitet wird).
- Cargo check / clippy:
  - `cargo check -p contextra-router --no-default-features`: GRÜN (0 Fehler, 0 Warnungen)
  - `cargo check -p contextra-router --all-targets --all-features`: GRÜN (0 Fehler, 0 Warnungen)
  - `cargo clippy -p contextra-router --all-targets --all-features --no-deps -- -D warnings`: GRÜN (ohne Befund)
  - `cargo clippy -p contextra-router --no-default-features --no-deps -- -D warnings`: GRÜN (ohne Befund)

## 2. Testläufe und Testzahlen
- `cargo test -p contextra-router`:
  - `contextra_router` (lib): 82 bestanden; 0 fehlgeschlagen; 0 ignoriert; 0 gemessen
  - `arm_registry_five_arms`: 3 bestanden
  - `bandit_latency`: 0 bestanden (0 Tests definiert ohne Feature)
  - `fc_ts_dispatch_test`: 0 bestanden (0 Tests definiert ohne Feature)
  - `golden_corpus_test`: 3 bestanden; 1 ignoriert (`test_generate_golden_corpus`)
  - `ips_requires_propensity`: 2 bestanden
  - `regret_monotonicity`: 0 bestanden
  - `router_audit_and_isolation_tests`: 19 bestanden
  - `router_bandit_dispatch`: 0 bestanden
  - Gesamtanzahl ausgeführter Tests: 82 + 3 + 3 + 2 + 19 = 109 Tests bestanden; 1 ignoriert. Dauer: ca. 0.15s.

- `cargo test -p contextra-router --all-features`:
  - `contextra_router` (lib): 84 bestanden (inkl. `bandit_regret_tests`)
  - `arm_registry_five_arms`: 3 bestanden
  - `bandit_latency`: 3 bestanden
  - `fc_ts_dispatch_test`: 3 bestanden
  - `golden_corpus_test`: 3 bestanden; 1 ignoriert
  - `ips_requires_propensity`: 2 bestanden
  - `regret_monotonicity`: 4 bestanden
  - `router_audit_and_isolation_tests`: 19 bestanden
  - `router_bandit_dispatch`: 5 bestanden
  - Gesamtanzahl: 84 + 3 + 3 + 3 + 3 + 2 + 4 + 19 + 5 = 126 Tests bestanden; 1 ignoriert.

## 3. Statische Analyse
- `tools/determinism_guard.py --crate contextra-router`:
  - Nicht-deterministische Stellen in Prod: 0
- `tools/zero_panic_analyzer.py --crate contextra-router`:
  - Prod-Verstöße bereinigt: 0 (Tool-Rohwert 14, ausschließlich in `#[cfg(test)]`-Modulen)
- Golden-Hashes:
  - `sha256sum -c MANIFEST.sha256`: alle 4 Golden-Dateien OK (arm_registry.json, routing_decisions.json, profiles_and_outcomes.json, dispatch_messages.json).

## 4. Feature-Matrix (Ist-Zustand vor WP-RT-1.1)
- `subprocess-dispatch`: PASS
- `bandit-routing`: FAIL (E0412 `BanditProfileState`)
- `cloud-egress-guard`: PASS
- `egress-sherman-morrison`: PASS
- `flow-corrected-thompson`: PASS
- `exp-fc-ts`: PASS
- `exp-routing-strategy`: FAIL (impliziert `bandit-routing`)
