# AGENTS.md — contextra-cognition

## Scope & Purpose
`contextra-cognition` is a Ring 3 crate in Contextra Cognitive OS. It provides memory consolidation (Near-Duplicate-Detection, Sliding-Window-Clustering), context window compaction, generative synthesis pass execution, and background maintenance scheduling.

## Strict Invariants
- `#![forbid(unsafe_code)]` enforced at crate root (`src/lib.rs`).
- Depends on `contextra-engine` for collection, storage, and transaction abstractions.
- Zero-Panic-Doctrine: return `ContextraError` via `?`.
