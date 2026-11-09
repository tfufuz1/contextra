# Anstehende API-Änderungen (AUFTRAG P09)

Zwei-Phasen-API-Regel: Änderungen an Symbolen oder Features, die von externen Crates konsumiert werden, werden dokumentiert und vom Integrator (P40) ausgeführt.

| Symbol / Feature | Betroffene Crates | Gewünschte Änderung |
|---|---|---|
| `mint_witness_for_tests` / Feature `erasure-conformance` in `contextra-testkit` | `contextra-crypto`, `contextra-engine` | Entfernung von `mint_witness_for_tests` und des Features `erasure-conformance` aus `contextra-testkit`, sobald die Konsumenten in Phase 2 entkoppelt sind. |
