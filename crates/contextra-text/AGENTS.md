# AGENTS.md — `contextra-text`

> **Ring 1 (`capabilities.toml`: Ring 0 ≠)** · Klasse `[K]` · Reifegrad `beta` · Unsafe: nein · Rolle: Inverted Index: BM25/BM25F, Block-Max-WAND, Tokenizer, Morphologie. Implementiert `TextIndex`.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-text --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `bm25f`, `docid-128` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- Kanten Ist = Soll; **bereits frei von Löschlogik** (0 Treffer). `inverted/purge.rs` (`TextPurger`) und `inverted/scanner.rs` (`TextScanner`) implementieren die Ports.
- Datentyp `Language` liegt in `inverted/types.rs`, Soll: `contextra-types` (WP 4.5).
- 43 `async fn` (Ratchet: nicht steigern, Ziel synchroner Kern).
- Ring in `capabilities.toml` 0, Soll 1.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `inverted/` (`index_struct`, `types`, `morph_index`, `purge`, `scanner`) | Index, Port-Implementierungen |
| `bm25.rs`, `wand.rs`, `posting_list.rs`, `stream.rs` | Scoring, Block-Max-WAND, Postings, Kandidatenstrom |
| `tokenizer.rs`, `morphology/`, `domain/legal_de.rs`, `domain/medical_de.rs` | Analyse, Komposita, Domänenvokabular |

## 4. Verträge & Invarianten

1. **Statistik je Tenant** (IDF, Dokumentlängen): keine mandantenübergreifende Posting-Liste oder Statistik (INV-TENANT-IDX-1).
2. **BM25-Referenzwerte** sind Test-Orakel (`tests/bm25_reference_values.rs`); WAND muss Differentialtest gegen Brute-Force bestehen (`wand_differential.rs`).
3. **Tokenizer ohne Panics:** Fehler statt `panic!`; ungültiges UTF-8 ist Eingabefehler.
4. Tombstone-Auflösung mit Read-Your-Writes (`tombstone_read_your_writes.rs`).
5. Gleichstände deterministisch (`wand_tie_break_threshold.rs`).

## 5. Ports

**Implementiert:** `TextIndex`, `Purger`, `LayerScanner`. **Konsumiert:** `StorageEngine` (generisch `TextPurger<S>`).

## 6. Nebenläufigkeit & Performance

- Posting-Listen unveränderlich je Segment; Updates über Delta/Tombstone; kurzes `parking_lot`-Locking.
- Komplexität: WAND sublinear in Postings; Allokationsprofil in `tests/alloc_profiler.rs`, Schreibverstärkung in `write_amplification.rs`.

## 7. Anti-Patterns

- Stoppwörter/Analyzer pro Aufruf neu bauen.
- Dokumentstatistik tenantübergreifend cachen.
- `Language`-Typ weiter in `text` erweitern.

## 8. Tests & Messung

```bash
cargo test -p contextra-text --locked
cargo clippy -p contextra-text --all-targets -- -D warnings
cargo test -p contextra-text --locked --features bm25f
(cd crates/contextra-text && cargo +nightly fuzz run fuzz_bm25_tokenize -- -max_total_time=60)
```

- Property-Tests (BM25-Invarianten, Purge), Fehlerinjektion (`purge_error_injection_tests.rs`), Fuzz-Korpus vorhanden.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 4.5 | Autarker Index, `Language` nach `types`, BM25F mit Block-Max-WAND |
| 3.4 | Index-Bereinigung als eigenständiger Baustein |
| 1.1 | Ring auf 1 |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-text:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-text:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-text:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `TEST` | 194 | `ANCHOR[contextra-text:TEST-195]` |
| `MORPH` | 28 | `ANCHOR[contextra-text:MORPH-029]` |
| `POST` | 23 | `ANCHOR[contextra-text:POST-024]` |
| `INV` | 21 | `ANCHOR[contextra-text:INV-022]` |
| `BM25` | 14 | `ANCHOR[contextra-text:BM25-015]` |
| `LIB` | 12 | `ANCHOR[contextra-text:LIB-013]` |
| `TOK` | 12 | `ANCHOR[contextra-text:TOK-013]` |
| `DOM` | 11 | `ANCHOR[contextra-text:DOM-012]` |
| `TEST-MORPH` | 10 | `ANCHOR[contextra-text:TEST-MORPH-011]` |
| `STREAM` | 10 | `ANCHOR[contextra-text:STREAM-011]` |
| `SCAN` | 8 | `ANCHOR[contextra-text:SCAN-009]` |
| `WAND` | 8 | `ANCHOR[contextra-text:WAND-009]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-text:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-text` durch (`find crates/contextra-text -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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
