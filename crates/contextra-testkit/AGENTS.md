# AGENTS.md — `contextra-testkit`

> **Werkzeug** · Klasse Werkzeug · Reifegrad `stable` · Unsafe: nein · Rolle: Test-Doubles und Konformitätssuiten: `FaultVfs`, `ManualClock`, In-Memory-Stores, Referenzmodell.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-testkit --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `testkit` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Nur als `dev-dependency`**; nie in Release-/Default-Features, nie Abhängigkeit eines Kerns im Normalbau.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `fault_vfs.rs`, `manual_clock.rs` | Fehlerinjektion, deterministische Uhr |
| `in_memory_{store,vector,text,graph}.rs` | Referenz-Backends |
| `reference_model.rs` | deterministisches Referenzmodell für MVCC/KV |
| `conformance/` (`clock`, `rng`, `storage`, `vector`, `text`, `graph`, `metrics`, `vfs`, `failure`) | Konformitätssuiten für Fremdadapter |
| `examples/standalone_testkit.rs` | Autarkes Baukasten-Minimalbeispiel (M10) |
| `mocks.rs` | Mocks (nie Produktionsstandard, INV-EMBED-1) |

## 4. Verträge & Invarianten

1. Doubles verhalten sich **wie die Spezifikation**, nicht wie die Implementierung (Orakel unabhängig vom Erzeuger, INV-ORACLE-1).
2. Deterministisch: keine Systemzeit, kein `thread_rng`.
3. Konformitätsfunktionen sind für Fremdprojekte nutzbar (Baukasten-Beweis).

## 5. Ports

Implementiert Test-Varianten von `Vfs`, `Clock`, `StorageEngine` u. a.

## 6. Nebenläufigkeit & Performance

- Keine Hintergrund-Threads ohne Join.

## 7. Anti-Patterns

- Mock als Produktionsimplementierung.
- Systemuhr in Tests.

## 8. Tests & Messung

```bash
cargo test -p contextra-testkit --locked
cargo clippy -p contextra-testkit --all-targets -- -D warnings
```

- Eigene Tests für die Doubles; Mutation-Score anstreben.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 2.5 | Konformitätssuiten für alle Ports |



---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-testkit:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-testkit:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-testkit:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `CONF-CLOCK-SUITE` | 1 | `ANCHOR[contextra-testkit:CONF-CLOCK-SUITE-002]` |
| `CONF-FAILURE-STRUCT` | 1 | `ANCHOR[contextra-testkit:CONF-FAILURE-STRUCT-002]` |
| `CONF-FAILURE-NEW` | 1 | `ANCHOR[contextra-testkit:CONF-FAILURE-NEW-002]` |
| `CONF-FAILURE-DISPLAY` | 1 | `ANCHOR[contextra-testkit:CONF-FAILURE-DISPLAY-002]` |
| `CONF-RESULT-TYPE` | 1 | `ANCHOR[contextra-testkit:CONF-RESULT-TYPE-002]` |
| `CONF-GRAPH-SUITE` | 1 | `ANCHOR[contextra-testkit:CONF-GRAPH-SUITE-002]` |
| `CONF-MACRO-ASSERT` | 1 | `ANCHOR[contextra-testkit:CONF-MACRO-ASSERT-002]` |
| `CONF-METRICS-SUITE` | 1 | `ANCHOR[contextra-testkit:CONF-METRICS-SUITE-002]` |
| `CONF-MOD-CLOCK` | 1 | `ANCHOR[contextra-testkit:CONF-MOD-CLOCK-002]` |
| `CONF-MOD-FAILURE` | 1 | `ANCHOR[contextra-testkit:CONF-MOD-FAILURE-002]` |
| `CONF-MOD-GRAPH` | 1 | `ANCHOR[contextra-testkit:CONF-MOD-GRAPH-002]` |
| `CONF-MOD-MACROS` | 1 | `ANCHOR[contextra-testkit:CONF-MOD-MACROS-002]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-testkit:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-testkit` durch (`find crates/contextra-testkit -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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
