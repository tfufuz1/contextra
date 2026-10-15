# ADR-T02: INV-KMS-1 — Strikte KMS-Provider-Verdrahtung im Crypto-Modul

* **Status:** Entwurf
* **Datum:** 2026-10-14
* **Target Path:** AGENTS.md, docs/decisions/ADR-T02-INV-KMS-1-kms-provider-verdrahtung.md

## Kontext
Schlüsselmaterial für Verschlüsselung und Signierung von Löschnachweisen muss sicher über deklarierte KMS-Provider (`KmsProvider`) bezogen und verdrahtet werden.

## Entscheidung (Vorschlag)
Formulierung von `INV-KMS-1`: Alle kryptografischen Schlüssel- und Derivationsoperationen müssen über den explizit injizierten `KmsProvider` laufen. Ein unvollständiges oder bypassed Wiring ist unzulässig.

## Status & Durchsetzung
Dieser ADR befindet sich im Status **Entwurf** und stellt einen Invarianten-Vorschlag dar. Er ist noch nicht als verbindliches Qualitätsgate in CI geschaltet.
