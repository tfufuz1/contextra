# AGENTS.md — contextra-wire
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.32

1. Zweck
FlatBuffers-IPC-Bindings, Zero-Copy-Adapter und JSON-RPC 2.0 Typen für Inter-Process Communication in Ring 0. Der Crate bildet die Unsafe-Insel für automatisch generierten FlatBuffers-Code (`contextra_generated.rs`), der nicht manuell verändert werden darf.

2. Modul-Karte
| Datei/Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | Crate-Einstiegspunkt und Re-Exports von IPC-, Adapter- und JSON-RPC-Typen |
| `src/adapter.rs` | `WireBuffer` — Zero-Copy Byte-Buffer-Wrapper für FlatBuffers-Nachrichten |
| `src/contextra_generated.rs` | Automatisches FlatBuffers-Generat aus `schemas/contextra.fbs` |
| `src/jsonrpc.rs` | `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError` — JSON-RPC 2.0 Nachrichtentypen |

3. Invarianten
- `INV-WIRE-NO-MANUAL-EDIT`: `contextra_generated.rs` darf NIEMALS manuell editiert werden; Schema-Änderungen erfolgen ausschließlich in `schemas/contextra.fbs` (`cargo xtask check-flatbuffers-drift`).
- `INV-WIRE-SAFETY-DOC`: Jede `unsafe`-Stelle oder `unsafe fn` MUSS einen expliziten `// SAFETY:`-Kommentar zur Begründung der Speichersicherheit enthalten.

4. Verboten / Anti-Patterns
- Manuelle Änderungen an `src/contextra_generated.rs` (Führt zu Schema-Drift in `cargo xtask check-flatbuffers-drift`).
- Direct-Pointer-Casts ohne vorherige FlatBuffers-Verifier-Prüfung (z. B. Verwendung von `root_as_*_unchecked` bei unvertrauten I/O-Puffern).

5. Nebenläufigkeit, Async- und Lock-Regeln
- Die FlatBuffers-Typen und `WireBuffer` sind rein synchrone Datenstrukturen.
- Gemäß P26 ist Async/tokio in Ring 0 unzulässig. Es existieren keine Sperren.

6. Verifikation
- `cargo test -p contextra-wire`
- `cargo xtask check-flatbuffers-drift`

7. Bekannte Lücken / SOLL
- `contextra_generated.rs` wird bei fehlendem `flatc`-Compiler im System aus dem eingecheckten Generat geladen.
