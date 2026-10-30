# AGENTS.md — contextra-engine

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. 29 Quelldateien mit Verweisen auf entfernte Crates (SPEC §16, B2). Die Hyperedge-Funktion `relate` hängt an der entfernten Graph-Crate. Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

## Scope & Purpose
`contextra-engine` is a Ring 3 crate in Contextra Cognitive OS. It provides the core storage, vector/text/graph indexing, transaction orchestration (`DbTransaction`), and collection management (`Collection`, `Contextra`).

## Strict Invariants
- `#![forbid(unsafe_code)]` enforced at crate root (`src/lib.rs`).
- Thread-safe concurrent collection access.
- Monotonic `TxId` allocation.
- Zero-Panic-Doctrine: return `ContextraError` via `?`.
