# Contextra — Jules Agent Context

**Stand:** 14.10.2026 · **Target Specification:** [`docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md`](../docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md)

---

## 1. Systemübersicht & Ring-Modell (§4.1)

Contextra ist eine air-gap-fähige, kryptografisch beweisbare Memory-Engine für KI-Agenten in Pure Rust.
Die Schichtenarchitektur ist streng in Ring 0 bis Ring 4 plus Tooling unterteilt:

```
Ring 0:  types < ports < wire < sys < durable-fs < simd < crypto-core < mvcc < text < rank
              < vector < adapt < graph < crypto-kv < proof-verify < audit-export < avv-generator
Ring 1:  store < checkpoint < kvcache
Ring 2:  sandbox, infer-candle, infer-ollama, infer-onnx
Ring 3:  privacy < engine < erasure-evidence < db < cognition < router < agent
Ring 4:  license < contextra < mcp, py
```

---

## 2. Agenten-Regeln (REG-01 bis REG-17)

1. **Zero-Panic (REG-01):** Kein `unwrap`, `expect`, `panic!` in Produktionscode von Ring 0/1.
2. **Ring-DAG (REG-02):** Strikte Richtung von höheren auf niedrigere Ringe.
3. **Determinismus (REG-03 / P28):** `Clock`, `Rng`, `IdGenerator` über `contextra-ports`.
4. **Drei Unsafe-Inseln (REG-04):** `sys`, `simd`, `wire` (generiert mit Header). Sonst `#![forbid(unsafe_code)]`.
5. **Wirksamkeit (REG-06):** Jedes `pub`-Symbol braucht Produktionsaufrufer und Test über die Fassade.
6. **Hülle EP1 (REG-07 / §4.4):** Target Hülle ≤ 18 Crates (Ziel 17 Crates) im Default-Build.

---

## 3. Harness-Workflow (`cargo xtask`)

Primary tool for all operations:
- `cargo xtask jules start --card <card>`
- `cargo xtask repo-doctor`
- `cargo xtask crate-context <crate>`
- `cargo xtask doctrine-scan --crate <crate>`
- `cargo xtask verify`
- `cargo xtask jules submit`
