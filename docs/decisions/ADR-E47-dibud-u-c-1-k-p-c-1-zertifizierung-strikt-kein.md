# ADR-E47: DiBud: `U_c = 1/(k+p_c+1)`, Zertifizierung strikt `>`, kein

* **Status:** Akzeptiert (aus Spezifikation §13 E47)
* **Datum:** 2026-10-15

## Entscheidung
DiBud: `U_c = 1/(k+p_c+1)`, Zertifizierung strikt `>`, kein Tie-Break auf Schranken.

## Begründung
Nur so ist die Schranke eine obere Schranke. Ein Tie-Break auf Schranken ist ohne echte Scores unsound.

## Verworfen
`U_c = 1/(c+n_c)` und `≥`-Variante (ARCHITECTURE_PLAN_v1 §19.3)
