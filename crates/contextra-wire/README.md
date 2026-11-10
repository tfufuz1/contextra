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

## Build-Skript & FlatBuffers-Regeneration (Entscheidung D-10)
- In `build.rs` wird geprüft, ob die Schemadatei `../../schemas/contextra.fbs` existiert.
- Da `cargo` Build-Skripte im Crate-Verzeichnis (`crates/contextra-wire`) ausführt, zeigt dieser Pfad auf `<Repo-Root>/schemas/contextra.fbs`. Dieser Pfad existiert im Repository nicht (das Schema liegt stattdessen unter `crates/contextra-wire/schemas/contextra.fbs`).
- Befund per `ls`:
  `ls -d crates/contextra-wire/../../schemas/contextra.fbs` liefert `No such file or directory`.
- Befund per `cargo build -v`:
  Das Build-Skript läuft durch, ohne Regeneration auszuführen, ohne `flatc` aufzurufen und ohne Fehler abzuliefern.
- **Entscheidung (D-10):** `build.rs` bleibt unverändert. Die automatische Regeneration läuft nie, wodurch der fest eingecheckte Code in `src/contextra_generated.rs` deterministisch und unverändert benutzt wird. Dies verhindert unabsichtliche Verhaltensänderungen. Die Unversehrtheit des Schemas wird stattdessen über den SHA-256 Prüfsummentest (`G-F2`, `cargo test -p contextra-wire --test schema_checksum`) sichergestellt.

## Bekannte Grenzen
- Größere Nachrichten übersteigen bei FlatBuffers und JSON-RPC konfigurierte Schutzgrenzen (`FlatBufferConfig`, `JsonRpcConfig`).
