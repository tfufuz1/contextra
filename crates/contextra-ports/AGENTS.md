# Contextra — AI-Assistenten-Kontext (`contextra-ports`)

## Verifizierter Codestand · Ring 0 (Ports)

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `contextra-ports`.
> `contextra-ports` definiert die dyn-kompatiblen Kern-Traits (StorageRead, VectorIndex, TextIndex, GraphIndex, Clock, Rng).
> Er erzwingt `#![forbid(unsafe_code)]`.

---

## Crate-Topologie

- **Ring 0 Ports**:
  - `dyn`-kompatible Interfaces zur Entkopplung aller Subsysteme.
