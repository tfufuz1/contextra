# MemFuse — AI-Assistenten-Kontext (`memfuse-wire`)

## Verifizierter Codestand · Ring 0 Unsafe Island

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `memfuse-wire`.
> `memfuse-wire` ist die Unsafe-Insel für automatisch generierte FlatBuffers IPC-Bindings
> und Zero-Copy-Adapter im Ring 0 des MemFuse-Sicherheitsarchitekturmodells.

---

## Crate-Topologie

- **Layer 0 — Ring 0 Unsafe Island**:
  - `memfuse-wire`: FlatBuffers-Generat (`memfuse_generated.rs`), Zero-Copy IPC-Adapter (`adapter.rs`).
  - `#![deny(unsafe_op_in_unsafe_fn)]` wird auf Crate-Ebene erzwungen.
