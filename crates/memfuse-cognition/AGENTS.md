# AGENTS.md — memfuse-cognition

## Scope & Purpose
`memfuse-cognition` is a Ring 3 crate in MemFuse Cognitive OS. It provides memory consolidation (Near-Duplicate-Detection, Sliding-Window-Clustering), context window compaction, generative synthesis pass execution, and background maintenance scheduling.

## Strict Invariants
- `#![forbid(unsafe_code)]` enforced at crate root (`src/lib.rs`).
- Depends on `memfuse-engine` for collection, storage, and transaction abstractions.
- Zero-Panic-Doctrine: return `MemFuseError` via `?`.
