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


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-wire:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-wire:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-wire:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `TST` | 64 | `ANCHOR[contextra-wire:TST-065]` |
| `RPC` | 18 | `ANCHOR[contextra-wire:RPC-019]` |
| `VER` | 12 | `ANCHOR[contextra-wire:VER-013]` |
| `ADP` | 5 | `ANCHOR[contextra-wire:ADP-006]` |
| `BLD` | 3 | `ANCHOR[contextra-wire:BLD-004]` |
| `FUZ` | 3 | `ANCHOR[contextra-wire:FUZ-004]` |
| `LIB` | 3 | `ANCHOR[contextra-wire:LIB-004]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-wire:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-wire` durch (`find crates/contextra-wire -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

- **Modul** (`//!`): Zweck, Ring-Zugehörigkeit, Nutzer, wichtigste Typen, Invarianten, Verweis auf Nachbarmodule.
- **Öffentliche Items:** Zweck in einem Satz, dann bei Bedarf `# Errors` (welcher Fehler wann), `# Panics` nur wenn der Code abbrechen kann (sollte nach Zero-Panic-Regel nicht vorkommen: dann `FIXME`), `# Safety` bei `unsafe fn`, Vorbedingungen, Nachbedingungen, Nebenwirkungen (I/O, Zeit, Zufall, IDs). Zeit, Zufall und IDs laufen nur über `Clock`/`Rng`/`IdGen`; steht es im Code, steht es im Kommentar.
- **Nicht-triviale private Funktionen:** Warum sie existieren und was sie garantieren.
- **Nebenläufigkeit:** Sperren, atomare Operationen, Reihenfolge, Warum kein Deadlock.
- **Persistenz und Fehler:** Wo Daten WAL oder Dateisystem-Port passieren; `fsync`-Fehler werden propagiert.
- **Tests:** Ein Satz je Testfunktion oder -block: welche Invariante oder welches Verhalten bewiesen wird.
- **Architektur:** Welche Ports die Crate nutzt/anbietet, welche Abhängigkeitsrichtung gilt (DAG, `may_depend_on`), was bewusst *nicht* hier liegt.

## Rustdoc-Fallen (jeder Punkt bricht sonst `cargo clippy -D warnings` oder `cargo doc`)

- Kein `///` vor Nicht-Items (`unused_doc_comments`); keine Leerzeile zwischen `///` und Item (`empty_line_after_doc_comments`).
- Fortsetzungszeilen in Listen und Zitaten korrekt einrücken (`doc_lazy_continuation`). **Vier Leerzeichen Einrückung in Doc-Text erzeugen einen Doctest**; Code immer in Fences mit Sprache. Beispiele, die nicht kompilieren oder laufen sollen: `ignore` oder `text`, begründet.
- Intra-Doc-Links `[`Typ`]` nur auf existierende, erreichbare Items; spitze Klammern im Text (`Vec<u8>`) in Backticks setzen, sonst „unclosed HTML tag“.
- `// SAFETY:` ohne Leerzeile und ohne dazwischenliegende Zeilen direkt über dem unsafe-Ausdruck.
- Doc-Beispiele müssen gegen die tatsächliche Signatur kompilieren; im Zweifel `no_run`/`ignore` mit Grund.