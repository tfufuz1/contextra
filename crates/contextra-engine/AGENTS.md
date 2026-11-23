# AGENTS.md — contextra-engine

## Scope & Purpose
`contextra-engine` is a Ring 3 crate in Contextra Cognitive OS. It provides the core storage, vector/text/graph indexing, transaction orchestration (`DbTransaction`), and collection management (`Collection`, `Contextra`).

## Strict Invariants
- `#![forbid(unsafe_code)]` enforced at crate root (`src/lib.rs`).
- Thread-safe concurrent collection access.
- Monotonic `TxId` allocation.
- Zero-Panic-Doctrine: return `ContextraError` via `?`.

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: größter Konsument von `contextra-store` (145 Zeilen mit `contextra_store`, davon ca. 26 in Produktivcode; Stand 23.11.2026).
- Genutzt: `LsmStorage`, `LsmConfig`, `tenant_codec::TenantScopedStorage`, `scanner::{lsm_purgers, lsm_scanners}` (Löschpfad `contextra_impl/erasure.rs`), `PressureLevel`, `SystemPressure`, `lsm::DurabilityMode` (Re-Export in `lib.rs:18`), `MergeOperator` (`merge_operator_adapter.rs`), `snapshot_registry.min_active_seqno()`.
- Vertrag IF-8: Umstellung über Adapter hinter einem Cargo-Feature (ab G-5); Mandant → `ClusterId`/`tenant_tag` im Adapter; `MergeOperator` entfällt (D-30) — produktiv ist nur `WasmMergeOperator` plus ein Test verdrahtet, Ersatz über `Guard::Version`; Löschpfad auf `Eraser`/`EraseLevel`.
- Verboten: vor G-5 Änderungen am Speicherzugriff; Zugriff auf Interna des Kerns.
- Arbeitspakete: WP-5.5 (erster Konsument, bestimmt den Adapterschnitt).
