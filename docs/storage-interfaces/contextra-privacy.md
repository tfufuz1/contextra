# `contextra-privacy` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Politik und Subject Directory.

## Schnittstelle und Semantik

Implementiert IdentityDirectoryOwner als MixedPerSubjectKey mit EntryDEK, Reverse Lookup, Gate, Rewrite und Attest; ScopeResolver prüft Tenant-Naht.

## Abnahme

Personenlöschung ohne Directory-Attest niemals Complete.

