# `contextra-py` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Python-Fassade.

## Schnittstelle und Semantik

Nutzt AsyncStorage/Produkt-API, nie synchronen Kern; exponiert View/Receipt/Ticket statt LSM-Stats.

## Abnahme

GIL-/Blocking-, typed Error- und Cursor-Tests.

