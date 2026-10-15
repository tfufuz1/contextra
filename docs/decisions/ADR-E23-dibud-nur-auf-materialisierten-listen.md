# ADR-E23: DiBud nur auf materialisierten Listen, strikt `>`; Feature `

* **Status:** Akzeptiert (aus Spezifikation §13 E23)
* **Datum:** 2026-10-15

## Entscheidung
DiBud nur auf materialisierten Listen, strikt `>`; Feature `dibud` bleibt aus, bis Gates bestanden (§9.2).

## Begründung
HNSW nicht präfix-stabil; Gleichstandslücke bei `≥`.

## Verworfen
DiBud direkt auf wachsendem `ef`
