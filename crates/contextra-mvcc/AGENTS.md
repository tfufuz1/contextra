# AGENTS.md — `contextra-mvcc`

> **Ring 0** · Klasse `[K]` · Reifegrad `stable` · Unsafe: nein · Rolle: Snapshots, Sequenzlog, Serializable Snapshot Isolation (SSI), Transaktionspuffer.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-mvcc --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)`, `warn(missing_docs)` |

## 2. Abhängigkeiten & Schulden

- Ist = Soll, autark. Dev-Kante auf `testkit` (zulässig als Werkzeug).
- Externe Abhängigkeit `parking_lot`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `seq_log.rs` | `SequenceLog`, `SeqLogEntry`, `SeqLogChange`: append-only Sichtbarkeit |
| `snapshot.rs` | `SnapshotRegistry`, `SnapshotGuard`: Pinning, Read-Isolation |
| `ssi.rs` | `ReadSet`, `SsiValidator`, `SequenceLogSsiValidator`, `PruningBlockerInfo` |
| `tx_buffer.rs` | `TxBuffer<T>` (sharded Staging, Timeout/Orphan-Reaper), `IndexOp<T>` |

## 4. Verträge & Invarianten

1. **SSI aktiv (INV-SSI-ACTIVE-1):** Read-Sets inkl. Prefix-/Range-Phantomschutz werden bei jedem Commit validiert.
2. **Index-Snapshot (INV-INDEX-SNAPSHOT-1):** Eintrag sichtbar nur für `born_seq ≤ S < died_seq`.
3. **Sperrenordnung (INV-LOCK-ORDER-1):** dokumentierte Reihenfolge, nie über `.await` (hier ohnehin synchron).
4. `TxId` kommt aus der Collection-Sequenz, nie aus der Uhr (P28). Zeit nur über `Clock` (`SequenceLogSsiValidator::new_with_clock_and_bounds`).
5. Begrenzte Strukturen: `max_tracked_keys`, Shard-Anzahl, Tx-Timeout sind Parameter, keine globalen Konstanten.
6. Die Snapshot-Lese-Sperre für gelöschte Betroffene setzt `contextra-erasure` (ReadGate); `mvcc` kennt sie nicht.

## 5. Ports

Konsumiert `Clock`. Implementiert keine Ports (reine Bausteine).

## 6. Nebenläufigkeit & Performance

- Sharded `TxBuffer` statt globalem Lock; Snapshot-Registry mit kurzem, einzelnem Mutex.
- Pflicht: `loom`-Modelle für Registry/Validator; Property-Tests gegen ein Referenzmodell (`testkit::reference_model`).
- Komplexität des Validators dokumentieren (Read-Set-Größe × aktive Schreiber) und per `benches/` belegen.

## 7. Anti-Patterns

- Systemzeit für Sequenzen.
- Unbegrenzte Read-Sets.
- Lock halten und anschließend Fremdcode (Callback) aufrufen.

## 8. Tests & Messung

```bash
cargo test -p contextra-mvcc --locked
cargo clippy -p contextra-mvcc --all-targets -- -D warnings
RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --locked
cargo bench -p contextra-mvcc
```

- `tests/` + `benches/` vorhanden. Ziel Mutation-Score ≥ 85 % (`cargo mutants -p contextra-mvcc`).

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 4.x | Isolations-Test: Crate allein in Fremdprojekt bauen (Beispiel unter `examples/`, WP 6.1) |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-mvcc:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-mvcc:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-mvcc:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| – | – | – |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-mvcc:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-mvcc` durch (`find crates/contextra-mvcc -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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
