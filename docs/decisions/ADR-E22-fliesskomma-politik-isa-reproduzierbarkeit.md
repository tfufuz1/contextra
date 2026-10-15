# ADR-E22: Fließkomma-Politik: Reproduzierbarkeit je ISA, Bit-Identität

* **Status:** Akzeptiert (aus Spezifikation §13 E22)
* **Datum:** 2026-10-15

## Entscheidung
Fließkomma-Politik: Reproduzierbarkeit je ISA, Bit-Identität nur für Festkomma/Integer, kanonischer Kernel beim HNSW-Aufbau, NaN-Ablehnung am Eingang (§7.6).

## Begründung
FMA bricht ISA-Gleichheit; Kahan hilft nicht; `total_cmp` ordnet negatives NaN vor −∞.

## Verworfen
„Bit-Identität über alle ISAs", Kahan-Pflicht
