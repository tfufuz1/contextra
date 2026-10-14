# AGENTS.md — contextra-crypto
> Ring 0 · Produkt: EP1/EP2 · Target Spec: SOLL-Zustand v1.0 (§3.1)

## 1. Zweck & Rolle
Kryptografische Funktionen. Künftiger Split (AP M5.5) in contextra-crypto-core (Ring 0), contextra-crypto-kv (Ring 0), contextra-erasure-evidence (Ring 3), contextra-proof-verify (Ring 0). Enforct #![forbid(unsafe_code)].
Spezifikation und Invarianten richten sich nach `docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md`.

## 2. Invarianten & Regeln (REG-01 bis REG-17)
- **REG-01 (Zero-Panic):** Kein unwrap, expect, panic! in Produktionscode.
- **REG-02 (Ring-DAG):** Strikte Einhaltung der Ring-Ordnung. Keine Aufrufe höherer Ringe.
- **REG-03 (Determinismus):** Zeit-, Zufall- und ID-Quellen ausschließlich über contextra-ports.
- **REG-06 (Wirksamkeit):** Jedes pub-Symbol benötigt Produktionsaufrufer und End-to-End Test.

## 3. Verifikation
```bash
cargo xtask crate-context contextra-crypto
cargo test -p contextra-crypto
cargo xtask doctrine-scan --crate contextra-crypto
```
