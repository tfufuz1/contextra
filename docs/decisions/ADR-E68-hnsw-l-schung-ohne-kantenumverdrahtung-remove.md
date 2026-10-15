# ADR-E68: HNSW-Löschung ohne Kantenumverdrahtung. `remove_with_graph_r

* **Status:** Akzeptiert (aus Spezifikation §13 E68)
* **Datum:** 2026-10-15

## Entscheidung
HNSW-Löschung ohne Kantenumverdrahtung. `remove_with_graph_repair` wird für Löschungen nicht verwendet. O-10 ist erledigt. Die Veto-Bedingung gilt, bis ein ADR sie ausdrücklich aufhebt.

## Begründung
VETO-F02 ist bedingt akzeptiert. Governance hat Vorrang (§0.2).

## Verworfen
Kantenreparatur bei Löschung (v1.1 bis v1.3)
