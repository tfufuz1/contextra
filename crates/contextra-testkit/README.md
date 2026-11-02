Testhilfen: `FaultVfs`, Referenzmodell, deterministische Uhr, In-Memory-Ports.

## Status

Ring dev. Ist-Zustand: vorhanden; Graph-Reste seit `b1637a8` entfernt; `FaultVfs` ersetzt das Feature `fault-injection`. Soll-Zustand nach [`docs/SPEC.md`](../../docs/SPEC.md), Abschnitt 5.1 und 8.

## Abhängigkeiten

Erlaubt laut SPEC 5.3: nach AR-14 frei für Dev-Abhängigkeiten; Produktionscrates dürfen nicht von `testkit` abhängen.

Das Crate ist Teil der Architekturprüfung (`contextra-arch-tests`). Abhängigkeiten werden nicht von Hand gepflegt, sondern gegen die Matrix geprüft (AR-1, AR-11).
