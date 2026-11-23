# `contextra-license` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Lizenz-/Feature-Gate.

## Schnittstelle und Semantik

Prüft oberhalb des Kerns; darf Erase, Recovery, Gate, Feed oder Attest nicht beeinflussen.

## Abnahme

Lizenzfehler verletzen keine Sicherheitsinvariante.

