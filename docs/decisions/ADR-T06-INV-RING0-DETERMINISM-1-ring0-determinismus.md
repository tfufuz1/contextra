# ADR-T06: INV-RING0-DETERMINISM-1 — Ring-0 Determinismus ohne ungekapselte I/O- oder Zeitaufrufe

* **Status:** Entwurf
* **Datum:** 2026-10-14
* **Target Path:** AGENTS.md, docs/decisions/ADR-T06-INV-RING0-DETERMINISM-1-ring0-determinismus.md

## Kontext
Ring 0 Crates (`contextra-types`, `contextra-errors`, `contextra-ports`, `contextra-durable-fs`, `contextra-sys`, `contextra-simd`) stellen die Fundamentalschicht dar. Direkte Wall-Clock-, System-RNG- oder ungekapselte Laufzeit-Aufrufe gefährden die Testbarkeit und den P28-Determinismus.

## Entscheidung (Vorschlag)
Formulierung von `INV-RING0-DETERMINISM-1`: Ring 0 Komponenten müssen rein deterministisch arbeiten; Zeit-, Zufalls- und ID-Quellen müssen strikt über injizierte Port-Traits abstrahiert werden (mit BSI TR-02102 CSPRNG-Ausnahme für kryptografische Schlüssel).

## Status & Durchsetzung
Dieser ADR befindet sich im Status **Entwurf** und stellt einen Invarianten-Vorschlag dar. Er ist noch nicht als verbindliches Qualitätsgate in CI geschaltet.
