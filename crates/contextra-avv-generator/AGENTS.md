# AGENTS.md — contextra-avv-generator
> Ring 0 · Produkt: EP2 · Target Spec: SOLL-Zustand v1.0 (§3.1)

## 1. Zweck & Rolle
AVV (Auftragsverarbeitungsvertrag) Generator gemäß Art. 28 DSGVO.
Spezifikation und Invarianten richten sich nach `docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md`.

## 2. Invarianten & Regeln (REG-01 bis REG-17)
- **REG-01 (Zero-Panic):** Kein unwrap, expect, panic! in Produktionscode.
- **REG-02 (Ring-DAG):** Strikte Einhaltung der Ring-Ordnung. Keine Aufrufe höherer Ringe.
- **REG-03 (Determinismus):** Zeit-, Zufall- und ID-Quellen ausschließlich über contextra-ports.
- **REG-06 (Wirksamkeit):** Jedes pub-Symbol benötigt Produktionsaufrufer und End-to-End Test.

## 3. Verifikation
```bash
cargo xtask crate-context contextra-avv-generator
cargo test -p contextra-avv-generator
cargo xtask doctrine-scan --crate contextra-avv-generator
```
