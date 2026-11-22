# AGENTS.md — contextra-cognition

## Scope & Purpose
`contextra-cognition` is a Ring 3 crate in Contextra Cognitive OS. It provides memory consolidation (Near-Duplicate-Detection, Sliding-Window-Clustering), context window compaction, generative synthesis pass execution, and background maintenance scheduling.

## Strict Invariants
- `#![forbid(unsafe_code)]` enforced at crate root (`src/lib.rs`).
- Depends on `contextra-engine` for collection, storage, and transaction abstractions.
- Zero-Panic-Doctrine: return `ContextraError` via `?`.

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: Konsument; `contextra-store` unter `[dependencies]`, aber nur in Tests (15 Zeilen, 0 in Produktivcode).
- Vertrag IF-8: Umstellung nach `contextra-engine`; Abhängigkeit vorher auf `[dev-dependencies]` (ab G-5).
- Verboten: neue Produktivnutzung von `contextra-store`.
- Arbeitspakete: WP-5.5.
