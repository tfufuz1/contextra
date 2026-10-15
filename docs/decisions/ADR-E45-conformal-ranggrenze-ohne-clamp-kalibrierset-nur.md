# ADR-E45: Conformal: Ranggrenze ohne Clamp. Kalibrierset nur aus relev

* **Status:** Akzeptiert (aus Spezifikation §13 E45)
* **Datum:** 2026-10-15

## Entscheidung
Conformal: Ranggrenze ohne Clamp. Kalibrierset nur aus relevanten Dokumenten.

## Begründung
Clamp auf `n` verletzt die Abdeckung bei kleinem `n`. Gemischte Scores ergeben keine einheitliche Nonkonformitätsfunktion.

## Verworfen
`clamp(rank,1,n)` und gemischte Scores (ADVANCED §3.2)
