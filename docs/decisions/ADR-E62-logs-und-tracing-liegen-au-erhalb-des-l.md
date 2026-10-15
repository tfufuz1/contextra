# ADR-E62: Logs und Tracing liegen außerhalb des Löschumfangs; Doc-IDs

* **Status:** Akzeptiert (aus Spezifikation §13 E62)
* **Datum:** 2026-10-15

## Entscheidung
Logs und Tracing liegen außerhalb des Löschumfangs; Doc-IDs nur als keyed hash (§5.2, G-29).

## Begründung
Logs enthalten Doc-IDs im Klartext. Eine Löschung, die Logs einschließt, ist mit vertretbarem Aufwand nicht zu leisten.

## Verworfen
Logs im Löschumfang
