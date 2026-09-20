# MemFuse — AI-Assistenten-Kontext (`memfuse-types`)

## Verifizierter Codestand · Ring 0 (Domain-Typen)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `memfuse-types`.
> `memfuse-types` definiert die fundamentalen Identifikatoren (DocId, TxId, TenantId), Filter-AST und Budgets.
> Er erzwingt `#![forbid(unsafe_code)]`.

---

## Crate-Topologie

- **Ring 0 Typen**:
  - `DocId`, `TxId`, `TenantId`, `EntityId`, `Embedding`.
  - Filter-AST und Budget-Typen.
