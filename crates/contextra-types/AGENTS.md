# AGENTS.md — `contextra-types`

> **Ring 0** · Klasse `[K]` · Reifegrad `stable` · Unsafe: nein · Rolle: kanonische IDs, Fehlerbaum, `RequestContext` und Basis-Datenmodelle, ohne jede Contextra-Abhängigkeit.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`. Diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, nicht kompiliert.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-types --no-default-features --locked` |
| Contextra-Abhängigkeiten (Ist = Soll) | keine |
| Externe Abhängigkeiten | `thiserror`, `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes` (dev: `proptest`) |
| Features | `docid-128` (breite `DocId`), `auto-extraction-opt-out`; `default = []` |
| Lints im Crate | `#![forbid(unsafe_code)]`, `#![deny(dead_code)]`, `#![warn(missing_docs)]` |
| Konsumenten | alle Crates (transitiv über `ports`) |

## 2. Abhängigkeiten

- **Erlaubt:** nur externe Crates. **Verboten:** jedes `contextra-*` (Ring-0-Wurzel).
- **Keine I/O, keine Async-Runtime, keine Uhr, kein Zufall.** Das Crate ist deterministisch (INV-RING0-DETERMINISM-1).

## 3. Modul-Karte (am Code verifiziert)

| Datei | Verantwortung |
|---|---|
| `error.rs`, `error_dto.rs` | `ContextraError` (≈51 Varianten, einziger Fehlerbaum, A6) und `ContextraErrorDto` für FFI/MCP |
| `ids.rs`, `types/domain/ids.rs` | `DocId` (`u64` oder per `docid-128` `u128`), `TxId` (mit `INTERNAL_BASE`), `TenantId`, `CollectionId` |
| `types/domain/{document,misc}.rs` | Dokument- und Hilfsmodelle |
| `types/{saos,filter,budget,importance}.rs` | Abfrage-, Filter-, Budget- und Gewichtstypen |
| `tenant_scope.rs`, `refs.rs`, `schema.rs`, `model_fingerprint.rs`, `retrieval_strategy.rs` | Mandantenscope, Referenzen, Schema, Embedding-Identität (INV-EMBED-2), Strategie |
| `tombstone.rs`, `types/domain/misc.rs` | `SeqBitTombstone`, `TombstoneSemanticsCheck`; die Konstante `TOMBSTONE_BIT` (`1 << 63`) steht in `types/domain/misc.rs` |
| `witness.rs` | `RequestContext`, `ScanMethod`, `SagaRecord` |
| `erasure.rs` | **Löschdomäne (516 Zeilen):** `EraseRequest`, `EraseReceipt`, `ErasureStatus`, `RetentionAttestation`, `LayerId`, `EraseTarget` |

## 4. Verträge & Invarianten

1. **Ein Fehlerbaum (A6).** Neue Varianten nur am **Ende** von `ContextraError` anfügen (Binärkompatibilität der DTOs). `From`-Impls stehen ausschließlich in `error.rs`. Keine crate-lokalen Fehler-Enums, die nicht in `ContextraError` münden.
2. **Tombstone-Disziplin.** Bit 63 einer Sequenznummer ist `TOMBSTONE_BIT`. Vor jedem `max_seq`-Vergleich maskieren: `seq & !TOMBSTONE_BIT`. Unmaskierte Vergleiche erzeugen Phantom-Sichtbarkeit.
3. **`TxId`-Bereiche.** Reguläre Sequenz vs. interner Systembereich (`INTERNAL_BASE`). Niemals aus `SystemTime` ableiten (P28).
4. **Tenant (VETO-F10).** `RequestContext` trägt den Mandanten. Typen dürfen keinen Default-Mandanten liefern.
5. **`DocId`-Breite.** `docid-128` ist ein **Workspace-weites** Feature (`vector`, `text`, `graph`, `store`, `py` spiegeln es). Ändere die Breite nie in einem Crate allein.
6. **Persistierte Typen** (`bincode`/`serde`) sind Formatvertrag (INV-FORMAT-1): Feldreihenfolge nicht ändern ohne Formatversion.
7. **DbC durch Typen.** Newtypes mit validierenden Konstruktoren (`try_new`), keine öffentlichen Felder dort, wo eine Invariante gilt.

## 5. Ports

Keine. `types` definiert Daten, keine Austauschrollen. Traits gehören nach `contextra-ports`.

## 6. Nebenläufigkeit & Performance

Keine Locks, kein `async`. Alle Typen sind `Send + Sync`, wo möglich. Große Strukturen (`Document`, Filter) sollen `Clone` günstig halten (`Arc`/`Bytes`). Neue Hot-Path-Typen: Größe mit `size_of` im Test festschreiben.

## 7. Anti-Patterns

- **Neue Typfamilie anlegen, ohne `docs/TYPE_REGISTRY.md` und `grep -rn "struct <Name>" crates/` zu prüfen.** `EraseTarget` und `LayerId` existieren bereits doppelt (`types` und `ports`, Root-§10 K3).
- **Erasure-Domänentypen weiter ausbauen.** `erasure.rs` gehört fachlich nach `contextra-erasure`. Nur reine Port-Typen dürfen im Fundament bleiben. Keine neuen Felder, keine neuen Varianten ohne ADR.
- **Variante in der Mitte einfügen** oder umbenennen (bricht DTO/Binärformat).
- **`unwrap()` in `From`-/`TryFrom`-Impls.**

## 8. Tests & Messung

```bash
cargo test -p contextra-types --locked
cargo test -p contextra-types --locked --features docid-128
cargo clippy -p contextra-types --all-targets -- -D warnings
```

- Property-Tests (`proptest`) für `DocId`-Roundtrip, DTO-Konvertierung (verlustfrei) und Tombstone-Maskierung.
- Zielwert Mutation-Score ≥ 85 % (`cargo mutants -p contextra-types`).
- Beide Feature-Varianten (`docid-128` an/aus) gehören in die CI-Matrix.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 2.2 / 3.9 | Doppelte Löschtypen (`EraseTarget`, `LayerId`) mit `ports` vereinen; Löschdomäne aus `types/erasure.rs` nach `contextra-erasure` verlagern, Port-Typen in `ports` belassen |
| 4.5 | `Language` aus `contextra-text` hierher verlagern (PLAN) |
| 5.5 | Typen, die heute über `contextra-core` re-exportiert werden, direkt hier beziehen (DEBT-03) |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-types:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-types:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-types:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `ID-DOC` | 128 | `ANCHOR[contextra-types:ID-DOC-129]` |
| `SCHEMA` | 8 | `ANCHOR[contextra-types:SCHEMA-009]` |
| `SCOPE` | 6 | `ANCHOR[contextra-types:SCOPE-007]` |
| `ERR` | 5 | `ANCHOR[contextra-types:ERR-006]` |
| `WITNESS` | 5 | `ANCHOR[contextra-types:WITNESS-006]` |
| `TOMB` | 4 | `ANCHOR[contextra-types:TOMB-005]` |
| `WIT` | 3 | `ANCHOR[contextra-types:WIT-004]` |
| `DTO` | 2 | `ANCHOR[contextra-types:DTO-003]` |
| `FINGERPRINT` | 2 | `ANCHOR[contextra-types:FINGERPRINT-003]` |
| `RETRIEVAL` | 2 | `ANCHOR[contextra-types:RETRIEVAL-003]` |
| `SCH` | 2 | `ANCHOR[contextra-types:SCH-003]` |
| `TS` | 2 | `ANCHOR[contextra-types:TS-003]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-types:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-types` durch (`find crates/contextra-types -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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