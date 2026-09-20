# MemFuse — AI-Assistenten-Kontext (`memfuse-testkit`)

## Verifizierter Codestand · Tooling

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `memfuse-testkit`.
> `memfuse-testkit` stellt Determinismus- und Testinfrastruktur bereit (ManualClock, FaultVfs, InMemoryStore).

---

## Crate-Topologie

- **Tooling**:
  - `ManualClock`: Injizierbare deterministische Uhr (P28).
  - `FaultVfs`: Fehlersimulation und Crash-Injektion für Storage-Tests.
