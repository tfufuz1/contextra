# AGENTS.md — `contextra-wire`

> **Ring 0 · Unsafe-Insel** · Klasse `[K]` · Reifegrad `stable` · Unsafe: **ja** (Insel: 45 generierte, 0 handgeschriebene) · Rolle: Zero-Copy-Wire-Format: FlatBuffers-IPC, JSON-RPC-Typen, Verifikation.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-wire --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | keine (0 Kanten, autarker Baustein) |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | keine (SPEC §4.4 erlaubt `types`, wird nicht benötigt) |
| Features | keine |
| Lints | `#![deny(unsafe_code)]` in `lib.rs` mit lokaler Ausnahme im generierten Submodul, `deny(clippy::unwrap_used/expect_used/panic)`, `deny(unsafe_op_in_unsafe_fn)` |
| Unsafe-Inventar | 45 maschinell erzeugte `unsafe`-Funktionen/Blöcke in `contextra_generated.rs`, 0 in handgeschriebenem Code |

## 2. Abhängigkeiten & Schulden

- Keine Contextra-Abhängigkeit; darf keine einführen.
- Konsumenten laut Cargo: `router`, `core` (deprecated), `mcp`, `py`. `mcp`/`py` sollen es nur über die Fassade erreichen (SPEC §4.4 Regel 4).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `generated.rs`, `contextra_generated.rs` | **Generierter** FlatBuffers-Code (`@generated`); nie von Hand ändern |
| `adapter.rs` | sichere Zero-Copy-Adapter (`WireBuffer`) um `bytes::Bytes` ohne Transmutes |
| `jsonrpc.rs` | transportneutraler, gehärteter JSON-RPC-2.0-Parser (Zero-Panic, Zero-Unsafe, DoS-Schutz) |
| `verification.rs` | Validierung eingehender Puffer vor dem Zugriff |
| `schemas/contextra.fbs` (+ `.sha256`), `build.rs`, `GENERATED.md` | Schema, Hash-Pin, Regenerierung |
| `examples/wire_standalone.rs` | Eigenständiges Einbettungsbeispiel ohne monorepo-Kopplung |
| `fuzz/`, `tests/` | Fuzz-Ziele (`fuzz_flatbuffers_ipc`, `fuzz_jsonrpc`, `fuzz_verification`), Integrationstests |

## 4. Verträge & Invarianten

1. **Generierter Code (P2, SPEC 4.6 Punkt 6):** Generator (`flatc 24.12.23`) und Schema-SHA-256 sind gepinnt. Regeneriere nur mit `flatc --rust -o src schemas/contextra.fbs` und aktualisiere `contextra.fbs.sha256`. Ausgabe muss in der CI reproduzierbar sein. Generiertes und handgeschriebenes Unsafe sind strikt getrennt ausgewiesen.
2. **Nie unverifizierte Puffer lesen.** Jeder externe Puffer läuft durch `verification` vor dem Zugriff. Untrusted Input (stdio/IPC) ist Fuzz-Ziel.
3. Nur **sichere Adapter** verlassen das Crate; generierte `unsafe`-Typen sind nicht Teil der stabilen API.
4. Schemaänderungen sind Formatänderungen (INV-FORMAT-1): nur additiv, mit Versionierung.
5. Handgeschriebener Produktionscode enthält strikt `#![deny(unsafe_code)]` und 0 `unsafe`-Blöcke.

## 5. Ports

Keine.

## 6. Nebenläufigkeit & Performance

- Zustandslos; Parsen ist Zero-Copy (`O(1)` Zusatzspeicher, `O(n)` Verifikation). Keine Allokation auf dem Lesepfad.
- Kein Runtime-Bezug (kein tokio, kein async).

## 7. Anti-Patterns

- `unwrap`/`expect` trotz `deny` (in Handcode strikt verboten).
- Generierte Dateien manuell patchen.
- Schema ändern, ohne Hash-Pin und Fuzz-Korpus zu aktualisieren.

## 8. Tests & Messung

```bash
cargo test -p contextra-wire --locked
cargo clippy -p contextra-wire --all-targets -- -D warnings
cargo run --example wire_standalone -p contextra-wire
```

- Pflicht: Fuzzing des IPC-Parsers, Roundtrip-Property-Tests.
- Gate: reproduzierbare Generierung (Hash-Vergleich in `tests/schema_drift.rs`).
- M2-Isolationsnachweis erfolgreich validiert.
