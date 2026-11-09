# contextra-wire

FlatBuffers Wire-Protokolle, Serialisierungs-Formate und gehärtetes JSON-RPC 2.0 für Contextra.

## Zweck
`contextra-wire` stellt transportneutrale Zero-Copy FlatBuffers Wire-Protokolle, sichere Puffer-Verifikation (`verification`), allokationsarme Adapter (`WireBuffer`) und einen gehärteten JSON-RPC-2.0-Parser ohne Panics bereit.

## Reifegrad
- **Status:** Stabil / Produktionsreif (P1–P6 erfüllt)
- **Unsafe-Disziplin:** Handgeschriebener Code erzwingt `#![deny(unsafe_code)]` (0 handgeschriebene Unsafe-Blöcke). Generierter Code (`contextra_generated.rs`) kapselt 45 `unsafe`-Stellen, jeweils lückenlos mit `// SAFETY:` dokumentiert.
- **Robustheit & Fuzzing:** Fuzz-Targets für FlatBuffers-IPC, Verifikation und JSON-RPC vorhanden.

## Interne Abhängigkeiten
`contextra-wire` ist ein autarker Baustein und hat **0 interne Workspace-Abhängigkeiten**. Externe Crates: `flatbuffers`, `bytes`, `serde`, `serde_json`, `thiserror`.

## Features
- `default = ["std"]`
- `std`: Standardbibliotheksunterstützung.
- `serde`: Serde-Serialisierung.
- `tracing`: Tracing-Integration.
- `inspect`: Erweiterte Diagnosemöglichkeiten.

## Minimalbeispiel
```rust
use bytes::Bytes;
use contextra_wire::WireBuffer;

let buf = WireBuffer::new(Bytes::from_static(b"raw-unverified-data"));
assert_eq!(buf.len(), 19);
```

## Entnahme-Rezept
1. Kopiere `crates/contextra-wire` in dein Projekt.
2. Binde es in deiner `Cargo.toml` ein:
```toml
[dependencies]
contextra-wire = { path = "../contextra-wire" }
```

## Bekannte Grenzen
- Größere Nachrichten übersteigen bei FlatBuffers und JSON-RPC konfigurierte Schutzgrenzen (`FlatBufferConfig`, `JsonRpcConfig`).
