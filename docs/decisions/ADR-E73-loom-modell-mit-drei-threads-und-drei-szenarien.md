# ADR-E73: Loom-Modell mit drei Threads und drei Szenarien: Fence vor F

* **Status:** Akzeptiert (aus Spezifikation §13 E73)
* **Datum:** 2026-10-15

## Entscheidung
Loom-Modell mit drei Threads und drei Szenarien: Fence vor Fenster 1, zwischen den Fenstern, nach Fenster 2.

## Begründung
FE-12 nennt zwei Szenarien. Drei decken die Lücke zwischen Check und Publish ab.

## Verworfen
Zwei Szenarien (FE-12)
