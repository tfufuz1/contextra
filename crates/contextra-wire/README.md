# contextra-wire

FlatBuffers Wire-Protokolle, Serialisierungs-Formate und gehärtetes JSON-RPC 2.0 für Contextra.

Normativitätsklasse: **Kern `[K]`** (Ring 0 Unsafe-Insel 3 von 3, `stable`; SPEC §4.1, §4.4, §4.6, §6.1, `capabilities.toml`).

## Rolle in einem Satz
`contextra-wire` stellt transportneutrale, performante Zero-Copy FlatBuffers Wire-Protokolle, sichere Puffer-Verifikation (`verification`), allokationsarme Adapter (`WireBuffer`) und einen gehärteten JSON-RPC-2.0-Parser ohne Panics bereit.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Unsafe-Insel 3 von 3, FlatBuffers FFI)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Abhängigkeiten (Ist):** Keine internen Workspace-Abhängigkeiten (0 Kanten). Externe Crates: `bytes`, `flatbuffers`, `serde`, `serde_json`.
- **Erlaubte Abhängigkeiten (`may_depend_on` Obergrenze):** `contextra-types` (SPEC §4.4, `capabilities.toml`), wird aktuell nicht benötigt; die Crate ist völlig autark.
- **Features:** Keine (Standard-Build ist minimal und vollständig).

## Baukasten-Garantien & Invarianten
1. **Zero-Panic-Garantie (P7, M4):**
   Kein `unwrap()`, kein `expect()`, kein `panic!`, kein `assert!` und keine ungeprüften Indexierungen in handgeschriebenem Produktionscode. Fehler werden als typisierte `Result`-Werte (`JsonRpcError`, `FlatBufferError`) fail-closed zurückgegeben.
2. **Unsafe-Disziplin & Trennung (M9, SPEC §4.6 Punkt 6):**
   - **Handgeschriebener Code:** Strikte Absicherung via `#![deny(unsafe_code)]` und `#![deny(unsafe_op_in_unsafe_fn)]` in `src/lib.rs`. Genau **0** handgeschriebene `unsafe`-Blöcke.
   - **Generierter Code (`generated`, `contextra_generated`):** Genau **45** maschinell durch `flatc` erzeugte `unsafe`-Funktionen/Blöcke für Low-Level-Tabellenzugriffe. Diese sind strikt im Modul `generated` gekapselt und verlassen die Crate nur über sichere Verifikations- und Zugriffs-APIs.
3. **Puffer-Verifikation vor Zugriff (M4, INV-FAILCLOSED-1):**
   Jeder externe Puffer muss vor Zugriff durch `verification` geprüft werden. Ungültige Offsets, korrupte Tabellen, Schachtelungsbomben und übergroße Payloads werden deterministisch zurückgewiesen.
4. **DoS-Schutz & Härtung im JSON-RPC 2.0:**
   - Einstellbare Obergrenzen (`max_message_size`, `max_depth`, `max_batch_size`).
   - Lineare Duplikat-Schlüsselerkennung (`reject_duplicate_keys`) ohne Heap-Allokationsstau.
5. **Kein DSGVO-/Löschbezug (M7):**
   Die Crate operiert ausschließlich auf Datenebene als Übertragungs- und Serialisierungsformat und besitzt keinerlei Rechts-, Mandanten- oder Löschsemantik.

## Einbettungsbeispiel
Ein eigenständiges Lauffähiges Beispiel befindet sich unter `examples/wire_standalone.rs` und kann direkt ausgeführt werden:

```bash
cargo run --example wire_standalone -p contextra-wire
```

Minimales Codebeispiel:
```rust
use bytes::Bytes;
use contextra_wire::{parse_jsonrpc_payload, JsonRpcPayload, JsonRpcResponse, WireBuffer};

// 1. Zero-Copy-Puffer
let buf = WireBuffer::new(Bytes::from_static(b"data"));
assert_eq!(buf.as_slice(), b"data");

// 2. Gehärtetes JSON-RPC 2.0
let rpc_bytes = br#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
if let Ok(JsonRpcPayload::Single(req)) = parse_jsonrpc_payload(rpc_bytes) {
    let resp = JsonRpcResponse::ok(req.id, serde_json::json!({"status": "pong"}));
    println!("{}", serde_json::to_string(&resp).unwrap());
}
```

## Reproduzierbarkeit der Schema-Generierung
- **Compiler:** `flatc 24.12.23`
- **Schema:** `schemas/contextra.fbs`
- **SHA-256 Pin:** `schemas/contextra.fbs.sha256`
- **Drift-Tests:** `tests/schema_drift.rs` validiert die Übereinstimmung des Hash-Pins und der Codegenerierung.
