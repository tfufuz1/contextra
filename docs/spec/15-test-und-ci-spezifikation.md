---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "15"
---
## 15. Test- und CI-Spezifikation

### 15.1 Unit-Tests

Jede öffentliche Funktion mit nicht-trivialer Logik erhält mindestens:
- einen Normalfall-Test,
- einen Grenzfall-Test (leere Eingabe, einzelnes Element),
- einen Fehlerfall-Test, der das korrekte `Result::Err`-Enum-Mitglied prüft (kein pauschales `is_err()`).

### 15.2 Integrationstests

| Testpfad | Zweck / AK |
|---|---|
| `crates/contextra-graph/tests/hyperedge_persistence_survives_restart.rs` | AK-1 |
| `crates/contextra-graph/tests/hyperedge_compact_race.rs` | AK-1 (nebenläufig zu `compact()`) |
| `crates/contextra-graph/tests/hyperedge_memory_budget.rs` | AK-2 |
| `crates/contextra-db/tests/signal_kind_no_new_variant.rs` | AK-4 |
| `crates/contextra-graph/tests/hyperedge_cascade_fanout.rs` | AK-6 (High-Fan-out) |
| `crates/contextra-graph/tests/community_hyperedges_included_flag.rs` | AK-7 |
| `crates/contextra-graph/benches/binary_edge_regression.rs` | AK-8 (Baseline-Vergleich) |
| `crates/contextra-graph/tests/hyperedge_payload_sharing.rs` | AK-9 |
| `crates/contextra-graph/tests/star_expansion_equals_clique.rs` | AK-10 |
| `crates/contextra-graph/tests/hyperedge_cascade_crash_recovery.rs` | AK-11 |
| `crates/contextra-vector/tests/sq8_bias_calibration.rs` | AK-12 |
| `crates/contextra-store/tests/wal_backpressure.rs` | AK-13 |
| `crates/contextra-store/tests/kv_locks_stable_shard.rs` | AK-14 |
| `crates/contextra-router/tests/ips_requires_propensity.rs` | AK-15 |

### 15.3 Loom-Tests (`#[cfg(loom)]`, `loom`-Feature)

| Testpfad | Zweck |
|---|---|
| `crates/contextra-store/tests/loom_group_commit.rs` | Group-Commit-Atomizität |
| `crates/contextra-store/tests/loom_multi_key_lock.rs` | Multi-Key-Deadlockfreiheit |
| `crates/contextra-graph/tests/loom_relate_n_ary.rs` | AK-3, Hyperkanten-Deadlockfreiheit |

Alle drei MÜSSEN als eigener CI-Job sichtbar grün laufen.

### 15.4 `.github/workflows/merge-gate.yml` — Pflicht-Jobs

```yaml
jobs:
  unit-tests:
    run: cargo test --workspace
  flatbuffers-drift-gate:
    run: cargo run -p xtask -- check-flatbuffers-drift
  hyperedge-schema-merge:
    needs: [flatbuffers-drift-gate]
    run: cargo test -p contextra-graph --features hyperedges -- hyperedge
  check-bandit-latency-budget:
    run: cargo run -p xtask --features contextra-router/egress-sherman-morrison -- check-bandit-latency-budget
  loom-tests:
    run: RUSTFLAGS="--cfg loom" cargo test --workspace --features loom -- --test-threads=1
  clippy-panic-lints:   # ersetzt `check-unwrap-baseline`: der Ratchet entfällt ersatzlos (§0.4)
    run: cargo clippy --workspace --lib --bins --locked -- -D warnings   # Lint-Konfiguration aus §0.4
```

**⚠️ Opus-Optimierung 3.1 — Feature-Kombinationen in CI (Stufe 3, gering):**
Powerset-Build der relevanten Feature-Flags ergänzen.

**⚠️ Opus-Optimierung 3.2 — Panic-Inventar (Stufe 3, gering):**
Gate nur für `src/` (ohne Tests/Benchmarks), harte sinkende Obergrenze.

---

<a id="16-abnahme"></a>
