# ADR-T01: Prinzip P29 — Kein globaler veränderlicher Zustand

* **Status:** Entwurf
* **Datum:** 2026-10-14
* **Target Path:** AGENTS.md, docs/decisions/ADR-T01-P29-kein-globaler-veraenderlicher-zustand.md

## Kontext
Globaler veränderlicher Zustand (`static mut`, globale mutable Einzellokationen oder `Lazy`/`OnceLock`-Zustände mit innerer Mutabilität) führt zu verdeckten Abhängigkeiten, Nicht-Determinismus und Race Conditions bei paralleler Testausführung sowie Mehrmandantenbetrieb.

## Entscheidung (Vorschlag)
Formulierung von Prinzip P29: Globaler veränderlicher Zustand ist im gesamten Workspace verboten. Alle Zustandsträger müssen instanzgebunden verwaltet oder explizit über Ports/Traits injiziert werden.

## Status & Durchsetzung
Dieser ADR befindet sich im Status **Entwurf** und stellt einen Invarianten-Vorschlag dar. Er ist noch nicht als verbindliches Qualitätsgate in CI geschaltet.
