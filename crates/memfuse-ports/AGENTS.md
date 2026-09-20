# MemFuse — AI-Assistenten-Kontext (`memfuse-ports`)

## Verifizierter Codestand · Ring 0 (Ports)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `memfuse-ports`.
> `memfuse-ports` definiert die dyn-kompatiblen Kern-Traits (StorageRead, VectorIndex, TextIndex, GraphIndex, Clock, Rng).
> Er erzwingt `#![forbid(unsafe_code)]`.

---

## Crate-Topologie

- **Ring 0 Ports**:
  - `dyn`-kompatible Interfaces zur Entkopplung aller Subsysteme.
