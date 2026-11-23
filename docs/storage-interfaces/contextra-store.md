# `contextra-store` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Eingefrorener LSM-Altbestand.

## Schnittstelle und Semantik

Keine neue Funktion oder neue Verbraucher; nur lesbarer Gegenstand eines einmaligen Forward-Exports. Er wird weder Dual-Write-Ziel noch dauerhafte Fassade.

## Abnahme

Tag `legacy-store-final`, verifizierter Offline-Import, danach Archivierung.

