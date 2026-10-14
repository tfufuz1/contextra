# AGENTS.md — contextra-checkpoint
> Ring 1 · Produkt: EP1 · Target Spec: SOLL-Zustand v1.0 (§3.1)

## 1. Zweck & Rolle
Persistent Checkpoint Engine, Blake3 Checksum-Verifikation, Anti-Resurrection Restore-Filter.
Spezifikation und Invarianten richten sich nach `docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md`.

## 2. Invarianten & Regeln (REG-01 bis REG-17)
- **REG-01 (Zero-Panic):** Kein unwrap, expect, panic! in Produktionscode.
- **REG-02 (Ring-DAG):** Strikte Einhaltung der Ring-Ordnung. Keine Aufrufe höherer Ringe.
- **REG-03 (Determinismus):** Zeit-, Zufall- und ID-Quellen ausschließlich über contextra-ports.
- **REG-06 (Wirksamkeit):** Jedes pub-Symbol benötigt Produktionsaufrufer und End-to-End Test.

## 3. Verifikation
```bash
cargo xtask crate-context contextra-checkpoint
cargo test -p contextra-checkpoint
cargo xtask doctrine-scan --crate contextra-checkpoint
```
