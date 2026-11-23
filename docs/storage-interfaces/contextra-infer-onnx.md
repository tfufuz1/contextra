# `contextra-infer-onnx` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

ONNX-Modelladapter.

## Schnittstelle und Semantik

Keine Storage-API; Persistenz nur über Owner und Modellfingerprints als Provenienz.

## Abnahme

Backend-Ausfall verändert Kernkorrektheit nicht.

