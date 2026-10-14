# ADR-T03: INV-DELETION-WIRE-1 — Durchgängige Verdrahtung der Löschpfade über alle Speicher-Layer

* **Status:** Entwurf
* **Datum:** 2026-10-14
* **Target Path:** AGENTS.md, docs/decisions/ADR-T03-INV-DELETION-WIRE-1-loeschpfad-verdrahtung.md

## Kontext
Beim Erzeugen eines Löschnachweises (Deletion Proof v4) müssen alle relevanten Subsystem-Layer (LSM-Memtable, SSTables, WAL, Checkpoint, HNSW, Text, Graph) verifiziert und bereinigt werden.

## Entscheidung (Vorschlag)
Formulierung von `INV-DELETION-WIRE-1`: Der Löschaufruf muss alle Subsystem-Layer sequentiell und abweichungsfrei durchlaufen. Fehlende Zeugen oder nicht bereinigte Layer sperren die Belegausstellung fail-closed.

## Status & Durchsetzung
Dieser ADR befindet sich im Status **Entwurf** und stellt einen Invarianten-Vorschlag dar. Er ist noch nicht als verbindliches Qualitätsgate in CI geschaltet.
