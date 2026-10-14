# AGENTS.md — contextra-cognition
> Ring 3 · Produkt: EP5 · Target Spec: SOLL-Zustand v1.0 (§3.1)

## 1. Zweck & Rolle
Hyperedge Spectral Deduplication (Brand Rang-1 SVD), Background Sleep Passes.
Spezifikation und Invarianten richten sich nach `docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md`.

## 2. Invarianten & Regeln (REG-01 bis REG-17)
- **REG-01 (Zero-Panic):** Kein unwrap, expect, panic! in Produktionscode.
- **REG-02 (Ring-DAG):** Strikte Einhaltung der Ring-Ordnung. Keine Aufrufe höherer Ringe.
- **REG-03 (Determinismus):** Zeit-, Zufall- und ID-Quellen ausschließlich über contextra-ports.
- **REG-06 (Wirksamkeit):** Jedes pub-Symbol benötigt Produktionsaufrufer und End-to-End Test.

## 3. Verifikation
```bash
cargo xtask crate-context contextra-cognition
cargo test -p contextra-cognition
cargo xtask doctrine-scan --crate contextra-cognition
```
