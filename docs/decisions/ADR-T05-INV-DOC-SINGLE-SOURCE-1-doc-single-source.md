# ADR-T05: INV-DOC-SINGLE-SOURCE-1 — Single Source of Truth für Dokumentationsreferenzen

* **Status:** Entwurf
* **Datum:** 2026-10-14
* **Target Path:** AGENTS.md, docs/decisions/ADR-T05-INV-DOC-SINGLE-SOURCE-1-doc-single-source.md

## Kontext
Redundante oder verstreute Spezifikationen führen zu Widersprüchen zwischen Architektur-Dokumenten und Code-Realität.

## Entscheidung (Vorschlag)
Formulierung von `INV-DOC-SINGLE-SOURCE-1`: Dokumentationen und Spezifikationen verweisen strictly auf kanonische Primärquellen (`docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`, `AGENTS.md`, `CONSTITUTION.md`). Veraltete Pfade oder duplizierte Regelsätze sind untersagt.

## Status & Durchsetzung
Dieser ADR befindet sich im Status **Entwurf** und stellt einen Invarianten-Vorschlag dar. Er ist noch nicht als verbindliches Qualitätsgate in CI geschaltet.
