# AGENTS.md — xtask-heavy

> Ring tooling · stable · Quelle: capabilities.toml

## 1. Zweck

Schwere xtask-Befehle mit Domain-Build-Abhängigkeit: Benchmark-Gates und Latenz-Budget-Prüfungen.
Wird von `cargo xtask bench-gate` und `cargo xtask check-bandit-latency-budget` aufgerufen.
Darf Domain-Crates als Cargo-Dependency haben (im Gegensatz zu Tier-0/Tier-1-Harness-Modulen).

## 2. Modul-Karte

| Datei | Verantwortung |
|---|---|
| `src/main.rs` | Entrypoint und Subcommand-Dispatcher |
| `src/bench_gate.rs` | `bench-gate`: Benchmark-Ergebnisse gegen Toleranz-Schwellenwert prüfen |
| `src/check_bandit_latency_budget.rs` | `check-bandit-latency-budget`: Bandit-Routing-Latenz-Budget-Validierung |

## 3. Invarianten

- **Tier-2-Constraint:** Darf Domain-Crates importieren, aber KEINE neuen xtask-Registry-Einträge ohne Eintrag in `xtask/registry.toml`.
- **Kein Gate-Weakening:** Toleranz-Schwellenwerte (`--tolerance`) dürfen nicht ohne ADR abgesenkt werden.

## 4. Verboten / Anti-Patterns

- Keine Tier-0/1-Logik in diesen Build-abhängigen Modulen (Tier-0/1-Analyse → `xtask/src/harness/`).
- Kein direkter `std::process::exit` ohne Fehlermeldung.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Rein synchrone Ausführung. Keine Async-Runtime.

## 6. Verifikation

```bash
cargo xtask bench-gate --tolerance 0.05
cargo xtask check-bandit-latency-budget
cargo xtask check-agents-integrity
```

## 7. Bekannte Lücken / SOLL

- Benchmark-Baseline-Aktualisierung (`ratchet`) ist über `cargo xtask ratchet` im Haupt-xtask geregelt.
