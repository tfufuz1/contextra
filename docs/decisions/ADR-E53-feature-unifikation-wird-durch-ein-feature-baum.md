# ADR-E53: Feature-Unifikation wird durch ein Feature-Baum-Gate abgesic

* **Status:** Akzeptiert (aus Spezifikation §13 E53)
* **Datum:** 2026-10-15

## Entscheidung
Feature-Unifikation wird durch ein Feature-Baum-Gate abgesichert (G-22). Konstruktor-Tests von `PhysicalScope` liegen in `contextra-store`.

## Begründung
Cargo vereinigt Features; eine Deklaration allein erzwingt die Grenze nicht. Die Selbst-Dev-Dependency in `contextra-ports` aktiviert das Feature für jeden Testbuild.

## Verworfen
Selbst-Dev-Dependency; reine Doku-Regel
