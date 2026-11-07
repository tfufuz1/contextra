# AGENTS.md — `contextra-py`

> **Ring 4** · Klasse Rand `[R]` · Reifegrad `stable` · Unsafe: nein · Rolle: PyO3-Python-Bindings (`_contextra`, abi3-py38) über der Rust-API.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-py --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `db`, `types`, `wire` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `types`, `ports`, `contextra` |
| Features | `docid-128`, `test-hooks` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Dep auf `db` statt auf die Fassade** (dokumentierte Ausnahme `ANCHOR:ARCH:DAG-003`).
- Workspace-Mitglied (nicht in `default-members`); Release-Profil workspace-weit `panic = "unwind"` für FFI. Frühere Angabe „nicht im Workspace“ ist überholt.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `bindings/` (`db`, `collection`, `document`, `search_result`, `hyperedge`, `crud_macros`, `runtime_state`, …) | PyO3-Klassen |
| `kv_links.rs`, `lib.rs` | Modulregistrierung |

## 4. Verträge & Invarianten

1. **GIL freigeben** bei I/O und Rechenlast (`py.allow_threads`).
2. **Panics an der Grenze abfangen** und in `PyErr` übersetzen; keine Unwinds nach Python.
3. `ContextraError` wird über `ContextraErrorDto` auf Python-Exceptions abgebildet (`NotFound→KeyError` etc.).
4. Vektoren als `PyReadonlyArray1<f32>` (Zero-Copy).
5. Globale Zustände (Runtime) isolieren (Sub-Interpreter, PEP 684).

## 5. Ports

Keine eigenen Ports.

## 6. Nebenläufigkeit & Performance

- Eigene Tokio-Runtime, unabhängig von Python-Threads; kein Lock über GIL-Grenzen.

## 7. Anti-Patterns

- `block_on` bei gehaltenem GIL.
- `Vec<f32>` statt NumPy-Array.

## 8. Tests & Messung

```bash
cargo test -p contextra-py --locked
cargo clippy -p contextra-py --all-targets -- -D warnings
```

- 13 Testdateien; CI `contextra-py-ci.yml`, Wheels `build-wheels.yml`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.3 | auf Fassade `contextra` umstellen |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-py:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-py:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-py:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `DB` | 13 | `ANCHOR[contextra-py:DB-014]` |
| `VAL` | 8 | `ANCHOR[contextra-py:VAL-009]` |
| `COL` | 6 | `ANCHOR[contextra-py:COL-007]` |
| `LINK` | 4 | `ANCHOR[contextra-py:LINK-005]` |
| `CNV` | 3 | `ANCHOR[contextra-py:CNV-004]` |
| `STAT` | 3 | `ANCHOR[contextra-py:STAT-004]` |
| `FN` | 3 | `ANCHOR[contextra-py:FN-004]` |
| `RNT` | 3 | `ANCHOR[contextra-py:RNT-004]` |
| `KV` | 3 | `ANCHOR[contextra-py:KV-004]` |
| `HLP` | 2 | `ANCHOR[contextra-py:HLP-003]` |
| `MACRO` | 2 | `ANCHOR[contextra-py:MACRO-003]` |
| `HYP` | 2 | `ANCHOR[contextra-py:HYP-003]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-py:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-py` durch (`find crates/contextra-py -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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
