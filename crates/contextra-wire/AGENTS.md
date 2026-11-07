# AGENTS.md — `contextra-wire`

> **Ring 0 · Unsafe-Insel** · Klasse `[K]` · Reifegrad `stable` · Unsafe: **ja** (Insel) · Rolle: Zero-Copy-Wire-Format: FlatBuffers-IPC, JSON-RPC-Typen, Verifikation.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-wire --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | keine |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | keine (SPEC §4.4 erlaubt `types`) |
| Features | keine |
| Lints | `#![deny(unsafe_code)]` mit lokalen Ausnahmen, `deny(clippy::unwrap_used/expect_used/panic)`, `deny(unsafe_op_in_unsafe_fn)` |

## 2. Abhängigkeiten & Schulden

- Keine Contextra-Abhängigkeit; darf keine einführen.
- Konsumenten laut Cargo: `router`, `core`, `mcp`, `py`. `mcp`/`py` sollen es nur über die Fassade erreichen (SPEC §4.4 Regel 4).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `generated.rs`, `contextra_generated.rs` | **Generierter** FlatBuffers-Code (`@generated`); nie von Hand ändern |
| `adapter.rs` | sichere Adapter um den generierten Code |
| `jsonrpc.rs` | JSON-RPC-2.0-Typen |
| `verification.rs` | Validierung eingehender Puffer |
| `schemas/contextra.fbs` (+ `.sha256`), `build.rs`, `GENERATED.md` | Schema, Hash-Pin, Regenerierung |
| `fuzz/`, `tests/` | Fuzz-Ziel `fuzz_flatbuffers_ipc`, Tests |

## 4. Verträge & Invarianten

1. **Generierter Code (P2):** Generator (`flatc 24.12.23`) und Schema-SHA-256 sind gepinnt. Regeneriere nur mit `flatc --rust -o src schemas/contextra.fbs` und aktualisiere `contextra.fbs.sha256`. Ausgabe muss in der CI reproduzierbar sein.
2. **Nie unverifizierte Puffer lesen.** Jeder externe Puffer läuft durch `verification` vor dem Zugriff. Untrusted Input (stdio/IPC) ist Fuzz-Ziel.
3. Nur **sichere Adapter** verlassen das Crate; generierte `unsafe`-Typen sind nicht Teil der stabilen API.
4. Schemaänderungen sind Formatänderungen (INV-FORMAT-1): nur additiv, mit Versionierung.
5. Handgeschriebene `unsafe`-Blöcke tragen `// SAFETY:`.

## 5. Ports

Keine.

## 6. Nebenläufigkeit & Performance

- Zustandslos; Parsen ist Zero-Copy (`O(1)` Zusatzspeicher, `O(n)` Verifikation). Keine Allokation auf dem Lesepfad.
- Kein Runtime-Bezug.

## 7. Anti-Patterns

- `unwrap`/`expect` trotz `deny` (Heuristik zeigt 16 Treffer; prüfen, ob generierter Code sie trägt und ob die Ausnahme dokumentiert ist).
- Generierte Dateien manuell patchen.
- Schema ändern, ohne Hash-Pin und Fuzz-Korpus zu aktualisieren.

## 8. Tests & Messung

```bash
cargo test -p contextra-wire --locked
cargo clippy -p contextra-wire --all-targets -- -D warnings
(cd crates/contextra-wire/fuzz && cargo +nightly fuzz run fuzz_flatbuffers_ipc -- -max_total_time=60)
```

- Pflicht: Fuzzing des IPC-Parsers, Roundtrip-Property-Tests, Miri für handgeschriebenes `unsafe`.
- Gate: reproduzierbare Generierung (Hash-Vergleich).

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.6 | Unsafe-/unwrap-Audit; Abgrenzung generiert vs. handgeschrieben getrennt ausweisen |
| 5.3 | Konsumenten (`mcp`, `py`) nur über Fassade |
