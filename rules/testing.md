# Testing Rules

This document consolidates rules for testing standards, test quality criteria, anti-test-mirroring, and chaos testing.

---

## Testing Core Principles & Anti-Test-Mirroring

> Origin: `rules/testing.md`

### Core Principle

**Every assertion must use a reference value computed independently of the code under test.**

A test that reconstructs the implementation's formula and asserts equality with the result
is a tautology — it proves the code does what the code does, not what it should do.

### Test-Mirroring Detection

```rust
// ❌ TEST-MIRRORING: assertion derived from implementation formula
let expected = (a - b).powi(2).sqrt();  // Same formula as compute()
assert!((result - expected).abs() < 1e-6);

// ✅ INDEPENDENT: reference value from external source or hand-calculated
// Euclidean distance of [1,0] and [0,1] is sqrt(2) ≈ 1.4142135
assert!((result - 1.4142135).abs() < 1e-4);
```

### Required Test Categories

For every module, tests must cover:

1. **Happy path** — normal input, expected output
2. **Empty input** — zero-length vectors, empty maps, nil transactions
3. **Boundary values** — `u64::MAX`, `f32::INFINITY`, dimension=1, dimension=0
4. **Error paths** — mismatched dimensions, corrupted data, invalid checksums
5. **Concurrency** — if the code uses locks/atomics, test with concurrent access

### Mutation Survival Check

Before marking a test suite as complete, ask:
> "If I changed `<` to `<=` or `+1` to `+0` in the implementation, would any test fail?"

If the answer is "no" for a logic branch, the test suite has a gap.

### Test Code Allowances

- `.unwrap()` and `.expect()` are **permitted** in test code (`#[cfg(test)]`)
- `panic!` in tests is acceptable for setup failures
- These allowances do NOT extend to production code called by tests

---

## Test Quality Criteria

> Origin: `rules/test_quality.md`

### Anti-Mirroring-Regel

Ein Test ist ungültig, wenn sein Erwartungswert aus derselben Formel berechnet wird wie die Implementierung.

**Falsch (Mirroring)**:
```rust
let result = rrf_score(rank: 0, k: 60);
assert_eq!(result, 1.0 / (60.0 + 0.0 + 1.0));  // Formel kopiert
```

**Richtig (unabhängiger Referenzwert)**:
```rust
// doc_b erscheint auf Rang 0 in Vektor-Set UND Rang 0 in Keyword-Set
// Erwarteter Score: 1/(60+1) + 1/(60+1) = 2/61 ≈ 0.03279
// doc_a erscheint nur auf Rang 1 in Vektor-Set: 1/(60+2) = 1/62 ≈ 0.01613
assert!(fused[0].id == "doc_b");  // höchster Score weil in beiden Sets
assert!(fused[0].score > fused[1].score);  // Reihenfolge ist unabhängig begründbar
```

### Pflicht-Grenzfälle

Jedes neue Modul braucht Tests für:
- Leere Eingabe
- Einzelnes Element
- Maximum capacity (OOM-Grenze, falls vorhanden)
- Fehlerfall (ungültige Dimension, korrupte Bytes, etc.)

### Proptest-Pflicht

Numerische Invarianten (SIMD vs. Scalar, Quantisierungsfehler) → **proptest**, nicht einzelne Handwerte.

### Mutation-Robustheit

Prüfprinzip: Würde `< statt <=` oder `+1 statt -1` diesen Test brechen?
Falls nein: Test hat kein ausreichendes Differenzierungspotenzial → erweitern.

---

## Chaos Testing Rules — WAL V3/MVCC Fault-Injection

> Origin: `rules/chaos_testing.md`

### Szenario-Übersicht

| Szenario | Beschreibung / Umsetzung in Contextra |
|---|---|
| `TaskMassacre` | Echte `JoinHandle::abort()` gegen konkurrierende Writer auf `LsmStorage` |
| `BitFlipInjection` | SSTable-Block/Bloom/Index-CRC Bit-Flips via direkte Dateimanipulation |
| `PowerCutSimulation` | Echter Prozess-Kill (SIGKILL) via Subprozess `examples/chaos_writer.rs` |
| `MemoryExhaustion` | Generiert Speicherdruck unter Nutzung des realen `ResourceTracker`/`LsmConfig`-Budgets |
| `DroppedWrite` | Simuliert I/O-Fehler auf Dateisystem-Ebene via `chmod`-Read-Only-Berechtigungen |
| `ConcurrentWriteFlood` | Konkurrierende Schreiber-Tasks fluten `LsmStorage` gleichzeitig unter knappem Budget |
| `CombinedChaosMatrix` | Kombinierte Ausführung mehrerer Fault-Szenarien in randomisierter Reihenfolge |

Explizit verworfene Szenarien:
- `IOLatency` (kein belegter Slow-Disk-Use-Case)
- `NetworkDegradation` (keine Netzwerkschicht in Contextra, siehe ADR-010)

### Kernregeln für Chaos-Tests

#### 1. Unabhängige Ground-Truth-Referenz
Jeder Chaos-Test MUSS einen von der Implementierung unabhängigen Ground-Truth-Wert verwenden (siehe Anti-Test-Mirroring), z.B. eine externe Log-Datei mit tatsächlich geschriebenen Werten VOR jedem Schreibversuch, NICHT eine aus dem WAL selbst rekonstruierte Erwartung.

#### 2. Isolierter Test-Scope
Diese Testsuite darf ausschließlich `tests/` und `examples/` in `crates/contextra-store` verändern. Jede Änderung an `src/` im Rahmen dieser Suite erfordert eine eigene, separate ADR.

### CI-Ausführung

- Einzeltests (`chaos_power_cut`, `chaos_task_massacre`, `chaos_bitflip_sstable`, `chaos_dropped_write`, `chaos_memory_pressure`) laufen als reguläre Integrationstests bei jedem `cargo test --workspace`.
- Die kombinierte Fault-Matrix (`crates/contextra-store/tests/chaos_matrix.rs`) ist `#[ignore]`-gated, wird über `just chaos-test` aufgerufen und läuft ausschließlich im nightly/schedule CI-Workflow (`.github/workflows/chaos.yml`) sowie bei manuellem Triggern via `workflow_dispatch`. Sie blockiert keine PRs.
- Bei lokaler Reproduktion eines CI-Fehlschlags wird der geloggte Seed übergeben: `CHAOS_SEED=<seed> just chaos-test`.
