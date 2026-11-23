# `contextra-agent` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

Workflow-, Audit- und Task-Zustand.

## Schnittstelle und Semantik

Sitzung ist Prefix oder TTL-Branch der echten Löschdomäne; Step/Audit/Resultat sind idempotente durable Changes.

## Abnahme

Crash/Retry, Branch-TTL und Löschpfad getestet.

