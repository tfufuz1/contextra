# AGENTS.md — `contextra-ingest`

> **Ring 4** · Klasse `[R]` (Rand/Extension) · Reifegrad `stable` · Unsafe: nein · Rolle: Dokument-Parsing und Text-Extraktion (PDF, DOCX, ODF, E-Mail, Tabellen, Markup) mit strukturbewusstem Chunking.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, verifiziert gegen Paket B24.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-ingest --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `types` |
| Features | `all-formats`, `fmt-*`, `xml`, `zip-container` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Kante `engine` vollständig entfernt (Paket B24):** Token-Schätzung (`estimate_tokens`), Window-Splitting (`chunk_text_with_overlap`) und Fallback-Chunking sind autark in `chunking/` implementiert. `contextra-ingest` hängt nur noch an `types`.
- **Kein `[K]`-Crate darf `ingest` importieren** (P30, E34).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `formats/` (`pdf`, `docx`, `pptx`, `odf`, `ole`, `email`, `sheets`, `structured`, `text`, `web`) | Extraktoren |
| `chunking/` (`prose`, `tabular`, `paged`, `email`, `structured`, `fallback`) | Autarke Chunking-Strategien |
| `safe/` (`archive`, `opc`, `xml`), `limits.rs` | Zip-Bomb-/Größenschutz, sicheres XML/OPC |
| `pipeline.rs`, `registry.rs`, `detect/`, `ir.rs`, `canonical.rs`, `fuzz_support.rs` | Pipeline, Formaterkennung, Zwischenform |

## 4. Verträge & Invarianten

1. **Alle Eingaben sind untrusted:** Größen-, Tiefen-, Entpack-Limits (`limits.rs`); kein Netz, keine externen Entitäten in XML.
2. **Jeder Extraktor ist Fuzz-Ziel**; Fehler sind typisiert (`IngestError`), nie Panic.
3. Jeder Chunk trägt Quellenangabe (Provenienz, Locators, kanonischer Hash).
4. Formatparser in Ring 0–3 sind verboten; Ingestion im Kern bleibt auf Text/Markdown beschränkt (SPEC §10.3).

## 5. Ports

Keine Ports (reine zustandslose Bibliothek / Ingestion-Engine).

## 6. Nebenläufigkeit & Performance

- Parser sind synchron und zustandslos; Parallelität auf Dokumentebene durch den Aufrufer.

## 7. Anti-Patterns

- Entpacken ohne Limit.
- `unwrap` auf Parserfehlern.
- Abhängigkeit des Kerns auf `ingest`.

## 8. Tests & Messung

```bash
cargo test -p contextra-ingest --locked
cargo clippy -p contextra-ingest -- -D warnings
```

- 26 Testdateien/-module; Fuzz je Format unter `fuzz/`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| B24 | Erledigt: Autarke Entkopplung von `engine`, Kante `engine` entfernt, Standalone-Beispiel hinzugefügt |
| 5.4 | `contextra-chunker` extrahieren, Kante `engine` entfernen |
| 1.1 | Ring angleichen |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-ingest:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-ingest:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-ingest:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `FUZZ` | 464 | `ANCHOR[contextra-ingest:FUZZ-465]` |
| `TST` | 450 | `ANCHOR[contextra-ingest:TST-451]` |
| `CHK` | 241 | `ANCHOR[contextra-ingest:CHK-242]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-ingest:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-ingest` durch (`find crates/contextra-ingest -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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
