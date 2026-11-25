# contextra-wire

`contextra-wire` ist eine Unsafe-Insel für FlatBuffers IPC-Generat und Zero-Copy-Adapter (Ring 0).

## Zweck

Stellt auto-generierte FlatBuffers IPC-Bindings und Adapter für schnellen, allokationsfreien IPC-Datenaustausch bereit.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Unsafe-Insel)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![allow(unsafe_code)]` — Isoliert in `contextra-wire` für auto-generierten IPC-Code.

## Öffentliche API-Übersicht

- **Generated IPC:** `contextra_generated::mem_fuse::ipc::*`
- **Adapter:** `adapter`

## Architektur & Verweise

Details zum IPC-Schema (§12) und CI-Drift-Gate finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §12.
