# contextra-core

`contextra-core` ist eine Strangler-Fassade für historische Core-Typen und Traits (Ring 0 / Legacy).

## Zweck

Dient während der Migration als Re-Export-Schicht für Typen und Traits, die schrittweise nach `contextra-types`, `contextra-ports` und `contextra-mvcc` ausgelagert wurden.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Legacy Facade)
- **Status:** 🟡 In Migration / Strangler
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

Re-exportiert grundlegende Submodule und Typen aus Ring-0-Crates:
- Re-exports von `contextra_types::*`
- Re-exports von `contextra_ports::*`

## Architektur & Verweise

Details zum Strangler-Muster und der Zerlegung von `contextra-core` finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §20 (Phase 1b).
