# ADR-T04: INV-SSI-ACTIVE-1 — Strikte Durchsetzung der Serializable Snapshot Isolation (SSI)

* **Status:** Entwurf
* **Datum:** 2026-10-14
* **Target Path:** AGENTS.md, docs/decisions/ADR-T04-INV-SSI-ACTIVE-1-ssi-durchsetzung.md

## Kontext
Mehrschritt-Transaktionen im MVCC-Subsystem benötigen ein aktives Read-Set-Tracking und Write-Skew-Erkennung, um Datenkonsistenz unter hoher Concurrency sicherzustellen.

## Entscheidung (Vorschlag)
Formulierung von `INV-SSI-ACTIVE-1`: SSI-Prüfungen müssen im Transaktions-Commit-Pfad aktiv geschaltet sein und dürfen nicht durch vereinfachte Read-Committed-Optionen abgeschwächt werden.

## Status & Durchsetzung
Dieser ADR befindet sich im Status **Entwurf** und stellt einen Invarianten-Vorschlag dar. Er ist noch nicht als verbindliches Qualitätsgate in CI geschaltet.
