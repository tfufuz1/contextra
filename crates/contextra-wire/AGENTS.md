# Contextra — AI-Assistenten-Kontext (`contextra-wire`)

## 1. Zweck
FlatBuffers-IPC-Bindings, Zero-Copy-Adapter und JSON-RPC 2.0 Typen für Inter-Process Communication in Ring 0. Der Crate bildet die Unsafe-Insel für automatisch generierten FlatBuffers-Code (`contextra_generated.rs`), der nicht manuell verändert werden darf.

> **Für AI-Assistenten:** Diese Datei beschreibt den Crate `contextra-wire`.
> `contextra-wire` ist die Unsafe-Insel für automatisch generierte FlatBuffers IPC-Bindings
> und Zero-Copy-Adapter im Ring 0 des Contextra-Sicherheitsarchitekturmodells.

---

## 2. Modul-Karte
| Datei/Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | Crate-Einstiegspunkt und Re-Exports von IPC-, Adapter- und JSON-RPC-Typen |
| `src/adapter.rs` | `WireBuffer` — Zero-Copy Byte-Buffer-Wrapper für FlatBuffers-Nachrichten |
| `src/contextra_generated.rs` | Automatisch generierter Rust-Code für `schemas/contextra.fbs` (`flatc`) |
| `src/jsonrpc.rs` | `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError` — JSON-RPC 2.0 Nachrichtentypen |

---

## 3. Invarianten
- `INV-WIRE-NO-MANUAL-EDIT`: `contextra_generated.rs` darf NIEMALS manuell editiert werden; Schema-Änderungen erfolgen ausschließlich in `schemas/contextra.fbs` (`cargo xtask check-flatbuffers-drift`).
- `INV-WIRE-SAFETY-DOC`: Jede `unsafe`-Stelle oder `unsafe fn` MUSS einen expliziten `// SAFETY:`-Kommentar zur Begründung der Speichersicherheit enthalten.

## Crate-Topologie

- **Layer 0 — Ring 0 Unsafe Island**:
  - `contextra-wire`: FlatBuffers-Generat (`contextra_generated.rs`), Zero-Copy IPC-Adapter (`adapter.rs`).
  - `#![deny(unsafe_op_in_unsafe_fn)]` wird auf Crate-Ebene erzwungen.

## 4. Verboten / Anti-Patterns
- Manuelle Änderungen an `src/contextra_generated.rs` (Führt zu Schema-Drift in `cargo xtask check-flatbuffers-drift`).
- Direct-Pointer-Casts ohne vorherige FlatBuffers-Verifier-Prüfung (z. B. Verwendung von `root_as_*_unchecked` bei unvertrauten I/O-Puffern).

## 5. Nebenläufigkeit, Async- und Lock-Regeln
- Die FlatBuffers-Typen und `WireBuffer` sind rein synchrone Datenstrukturen.
- Gemäß P26 ist Async/tokio in Ring 0 unzulässig. Es existieren keine Sperren.

## 6. Verifikation
- `cargo test -p contextra-wire --locked`
