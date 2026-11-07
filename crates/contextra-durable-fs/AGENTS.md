# AGENTS.md — `contextra-durable-fs`

> **Ring 0** · Klasse `[K]` · Reifegrad `beta` (in `capabilities.toml` eingetragen) · Unsafe: nein · Rolle: Dateisystem-Härtung: Durability-Flush, atomares Rename, Health-State-Machine, Fehlerinjektion, RealFs VFS-Port-Implementierung.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-durable-fs --no-default-features --locked` |
| Contextra-Abhängigkeiten (Cargo) | `ports`, `sys`, `types` |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]`, `deny(clippy::unwrap_used/expect_used/panic)`, `warn(missing_docs)` |

## 2. Abhängigkeiten & Schulden

- **In `capabilities.toml` eingetragen:** Ring 0, `may_depend_on = [ports, sys, types]`.
- **Einheitlicher `Vfs`-Trait:** `vfs.rs` implementiert `contextra_ports::Vfs` (`RealFs`) und `contextra_ports::VfsFile` (`RealFsFile`).
- `store`/`vector` Umstellung folgt in Paket B14 / B12.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `vfs.rs` | `RealFs`, `RealFsFile`, implementiert `contextra_ports::Vfs` |
| `lib.rs` | `FlushKind`, `FsClass`, `flush_file`, `flush_dir`, `rename_durable`, `DurableHandle<F: FaultInjector>`, `PoisonedHandle`, `DirLock` |
| `health.rs` | `HealthState`, `HealthEvent`, `AtomicHealthState` (Healthy→Degraded→ReadOnly→Poisoned) |
| `fault_injector.rs` | `FaultInjector`, `NoFault` |
| `journal.rs` | `PendingDeleteJournal`: aufgeschobene Datei-Löschungen (mechanisch) |
| `scrub.rs` | `scrub_if_safe` (als deprecated markiert, Migration zu E2) |

## 4. Verträge & Invarianten

1. **Fehler propagieren (INV-FSYNC-1):** nach `fsync`-Fehler ist der Handle `Poisoned`; weder Ack noch Retry auf demselben Handle.
2. **Gesundheitsübergänge** laufen ausschließlich über `HealthState::transition`; `allows_writes()` ist die einzige Schreibfreigabe.
3. `rename_durable` = tmp-Datei schreiben, `fsync`, `rename`, Parent-Verzeichnis `fsync`.
4. `scrub`/`journal` sind **mechanisch**: keine Rechts- oder Nachweislogik (M7). Nachweise erzeugt `contextra-erasure`.
5. `FaultInjector` ist generisch (Monomorphisierung): `NoFault` kostet im Produktionspfad nichts.

## 5. Ports

`RealFs` implementiert `contextra_ports::Vfs`.

## 6. Nebenläufigkeit & Performance

- `AtomicHealthState` ist lock-frei (Atomics). Kein Shared Mutable State außer diesem Zustand.
- Blockierend: in `async`-Kontext nur über `spawn_blocking`.

## 7. Anti-Patterns

- Zweiten `Vfs`-Trait weiter ausbauen.
- `let _ = flush_dir(..)`.
- Plattformabhängiges Verhalten ohne `FsClass`-Prüfung annehmen (z. B. Scrub auf CoW-/Flash-Medien als Beweis werten).

## 8. Tests & Messung

```bash
cargo test -p contextra-durable-fs --locked
cargo clippy -p contextra-durable-fs --all-targets -- -D warnings
```

- Vorhanden: 9 Testdateien in `tests/`, inklusive `vfs_conformance.rs` mit `contextra_ports::contract_tests::verify_vfs_contract`.
- Chaos: `FaultInjector` für Torn Writes, `ENOSPC`, `fsync`-Fehler.

## 9. Folgearbeiten

| WP | Inhalt |
|---|---|
| B14 | `contextra-store` per Konstruktor auf `Vfs` (`RealFs`) umstellen |
| B12 | `contextra-vector` auf `Vfs` umstellen |
| E2 | `scrub.rs` nach `contextra-erasure-store` verlagern |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-durable-fs:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-durable-fs:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-durable-fs:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `FAULT-TRAIT` | 1 | `ANCHOR[contextra-durable-fs:FAULT-TRAIT-002]` |
| `NO-FAULT` | 1 | `ANCHOR[contextra-durable-fs:NO-FAULT-002]` |
| `HEALTH-STATE` | 1 | `ANCHOR[contextra-durable-fs:HEALTH-STATE-002]` |
| `HEALTH-EVENT` | 1 | `ANCHOR[contextra-durable-fs:HEALTH-EVENT-002]` |
| `ATOMIC-HEALTH-STATE` | 1 | `ANCHOR[contextra-durable-fs:ATOMIC-HEALTH-STATE-002]` |
| `JOURNAL-STRUCT` | 1 | `ANCHOR[contextra-durable-fs:JOURNAL-STRUCT-002]` |
| `FLUSH-KIND` | 1 | `ANCHOR[contextra-durable-fs:FLUSH-KIND-002]` |
| `FS-CLASS` | 1 | `ANCHOR[contextra-durable-fs:FS-CLASS-002]` |
| `POISONED-HANDLE` | 1 | `ANCHOR[contextra-durable-fs:POISONED-HANDLE-002]` |
| `DURABLE-HANDLE` | 1 | `ANCHOR[contextra-durable-fs:DURABLE-HANDLE-002]` |
| `LOCK-MODE` | 1 | `ANCHOR[contextra-durable-fs:LOCK-MODE-002]` |
| `DIR-LOCK` | 1 | `ANCHOR[contextra-durable-fs:DIR-LOCK-002]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-durable-fs:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-durable-fs` durch (`find crates/contextra-durable-fs -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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
