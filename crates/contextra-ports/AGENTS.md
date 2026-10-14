# AGENTS.md — contextra-ports
> Ring 0 · stable · Produkt: EP1 · Target Spec: SOLL-Zustand v1.0 (§3.1, §5.5)

## 1. Zweck
`contextra-ports` definiert alle rein abstrakten Port-Traits (`Clock`, `Rng`, `IdGenerator`, `StorageEngine`, `ErasureBackend`, `WriteGate`, `FreeSpaceProbe`, `RestoreGuard`, `LayerScan`, `ReadSnapshot`, `MergeOperatorFactory`, `LicenseGate`, `ErasureHooks` u. a.) zur vollkommenen Entkopplung aller Subsysteme.
Es verlangt `#![forbid(unsafe_code)]` und dient als zentraler Ring-0-Vertrag ohne konkrete Backend-Implementierungen.

## 2. Invarianten & Regeln (REG-01 bis REG-17)
- **REG-02 / Ring 0:** Keine Abhängigkeiten zu höheren Ringen (Ring 1+). Keinerlei `tokio`-Laufzeit-Abhängigkeit im Produktions-pfad.
- **REG-03 / Determinismus:** `Clock`, `Rng`, `IdGenerator` definieren die Schnittstellen für Zeit, Zufall und IDs.
- **REG-13 / Ports-Only:** Kern-Port-Traits hängen nie direkt von Erasure-Plugin- oder Evidence-Crates ab.
- **REG-14 / Fehlerfreie Evidenz:** `LayerObservation` trägt `layer_id: u16`, `clean: bool`, `state_digest: [u8; 32]` und `item_count: u64`. `None`- / Default-Zeugen sind verboten.
- **Port-Feature `backend-impl`:** Konstruktoren von `PhysicalScope` sind ausschließlich hinter dem Feature `backend-impl` freigeschaltet, welches nur von `contextra-store` aktiviert werden darf (Gate G-12).
- **Feature `test-utils`:** Befindet sich ausschließlich in `[dev-dependencies]`.

## 3. Verifikation (xtask Cheat-Sheet)

```bash
cargo xtask crate-context contextra-ports
cargo test -p contextra-ports
cargo xtask check-ring0-async-purity
cargo xtask unwired-ports --only-unwired
cargo xtask check-port-features
cargo xtask doctrine-scan --crate contextra-ports
```
