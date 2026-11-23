# `contextra-sys` — Schnittstellenspezifikation

**Normbasis:** [SPEC-INTEGRATION](../SPEC-INTEGRATION.md), [zentraler Integrationsvertrag](../CONTEXTRA-STORAGE-INTEGRATION-SPEC.md) und [Storage-SPEC](../../crates/contextra-storage/SPEC.md). Bei Konflikt gilt die höhere Quelle.

## Rolle

OS-/Unsafe-Insel.

## Schnittstelle und Semantik

Nur optionale, nichtkanonische Beschleunigung und HostProbe; kein Kern-I/O-Bypass, mmap/O_DIRECT/io_uring im Kern.

## Abnahme

Capability-Ausfall verändert keine Sichtbarkeit oder Löschsemantik.

