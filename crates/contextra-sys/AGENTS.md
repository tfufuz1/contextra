# AGENTS.md — `contextra-sys`

> **Ring 0 · Unsafe-Insel** · Klasse `[K]` · Reifegrad `stable` · Rolle: autarke Betriebssystem-Primitive (Durability-Flush, `mmap`, Memory-Lock, FD-Limits, Datei-ACL) hinter einer **sicheren** API.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`. Stand 2026-11-07, statisch gelesen, nicht kompiliert.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-sys --locked` (keine Contextra-Abhängigkeit, keine Features) |
| Contextra-Abhängigkeiten (Ist = Soll) | keine |
| Lints | `#![allow(unsafe_code)]` (Insel) mit `#![deny(unsafe_op_in_unsafe_fn)]`, `#![warn(missing_docs)]` |
| Unsafe-Umfang | 21 `unsafe`-Blöcke (`unsafe {`), 0 `unsafe fn`, 0 `unsafe impl`, ausnahmslos mit `// SAFETY:` |
| Konsumenten (Ist, laut Cargo) | `store`, `vector`, `durable-fs`, `bench`, `db` (Kommentar), `engine` (tote Kante, 0 Nutzungen im Code) |

## 2. Abhängigkeiten

Keine. Das Crate steht am Boden des DAG und darf nie ein `contextra-*`-Crate importieren. (SPEC §4.4 erlaubt `types` nur falls nötig; heute nicht gebraucht.)

## 3. Modul-Karte (am Code verifiziert)

| Datei | Öffentliche API | Verantwortung |
|---|---|---|
| `durability.rs` | `durable_flush_file`, `durable_flush_dir`, `write_all_durable` | plattformgerechte Durability-Primitive (`fdatasync`, `F_FULLFSYNC`, Verzeichnis-Sync) |
| `mmap.rs` | `mmap_readonly` | schreibgeschützte Abbildung |
| `mlock.rs` | `mem_lock`, `mem_lock_with_policy`, `mem_unlock`, `MemLockPolicy` | Speicher sperren (Swap-Schutz) |
| `vault.rs` | `LockedRegions` | verwaltete gesperrte Regionen für Schlüsselmaterial |
| `posix.rs` | `ensure_fd_limit`, `set_fd_limit`, `reopen_and_dup2` | FD-Limits, Umleitung |
| `acl_win32.rs` | `set_restrictive_file_acl`, `verify_file_acl_owner_only` | Windows-ACL (nur Best Effort, INV-PLATFORM-1) |

## 4. Verträge & Invarianten

1. **Sichere API nach außen (M9).** Jede `pub fn` ist sicher aufrufbar. Alle Vorbedingungen (Längen, Ausrichtung, Dateityp) werden **vor** dem `unsafe`-Block geprüft und als `Err` gemeldet.
2. **Jeder `unsafe`-Block trägt `// SAFETY:`** mit den eingehaltenen Invarianten (Pointer-Gültigkeit, Alignment, Lebensdauer, Aliasing). `clippy::undocumented_unsafe_blocks = deny` gilt workspace-weit. Auch Plattformcode (Win32) ist eingeschlossen.
3. **`mmap` nur für vollständig prüfsummenverifizierte, unveränderliche, volumenlokale Dateien** (INV-IO-1). Andernfalls `pread` über `Vfs`. Zero-Copy-`mmap` auf unverschlüsselten Volumes ist nur zulässig, wenn `doctor` ein verschlüsseltes Volume nachweist (INV-ENCRYPT-REST-2).
4. **Durability-Fehler propagieren (INV-FSYNC-1).** `durable_flush_*` gibt jeden Fehler zurück. Der Aufrufer vergiftet den Handle und wiederholt **nie** auf demselben Handle.
5. **Plattformäquivalente deklarieren.** Funktionen ohne Äquivalent (`mlock` unter Windows) liefern einen benannten Fehler oder eine `MemLockPolicy`-Entscheidung, nie ein stilles `Ok`.
6. **Kein SIGBUS durch `mmap`.** Truncate-Races sind zu bedenken: Datei nach Verifikation nicht mehr verkürzbar halten (Lock) oder `pread` verwenden.
7. **Keine Heap-Allokation mit Geheimnissen ohne Zeroization** in `LockedRegions` (INV-HEAP-ZERO-1).

## 5. Ports

Konsumiert und implementiert keine Ports (das Crate ist port-frei, damit es überall einsetzbar ist). Ein `Vfs`, das diese Primitive nutzt, lebt in `contextra-durable-fs`.

## 6. Nebenläufigkeit & Performance

Keine globalen Zustände, kein Runtime-Bezug. `ensure_fd_limit`/`set_fd_limit` verändern **prozessweiten** Zustand: nur aus der Composition Root aufrufen, nie aus Bibliothekscode. `durable_flush_*` blockiert: in `async`-Kontext nur über `spawn_blocking`.

## 7. Anti-Patterns

- `unsafe` ohne `// SAFETY:` oder mit nur beschreibendem Kommentar („reads pointer“).
- `let _ = durable_flush_dir(..)`: verschluckt eine Durability-Verletzung.
- `mmap` auf Dateien, die konkurrierend wachsen oder schrumpfen können.
- Plattform-`cfg`-Zweige, die das Verhalten **semantisch** ändern, ohne dass `doctor` es ausweist.
- Neue `unsafe`-Nutzung außerhalb dieses Crates, `simd` und `wire` (M9): stattdessen hier eine sichere Funktion ergänzen.

## 8. Tests & Messung

```bash
cargo test -p contextra-sys --locked
cargo +nightly miri test -p contextra-sys            # soweit Miri die Syscalls unterstützt; sonst Referenztest
cargo clippy -p contextra-sys --all-targets -- -D warnings
```

- Gegen eine sichere Referenz testen (Property-Test: `mmap_readonly` liefert dieselben Bytes wie `std::fs::read`).
- Fehlerinjektion über `contextra-durable-fs::FaultInjector` für den Flush-Pfad.
- Es gibt **kein** `tests/`-Verzeichnis im Crate. Ergänze eines für Plattform- und Referenztests.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 4.2 | `mmap`-Abstraktion mit versiegelten Speicherbereichen (`SealedMap`), robuste Signal-/Fehlerbehandlung ohne Panics |
| 5.6 | Unsafe-Audit: jeder Block mit `// SAFETY:`, Miri-Abdeckung belegen |
| D9 | Tote Kante `engine → sys` und `db → sys` entfernen (laut PLAN D9) |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-sys:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-sys:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-sys:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `ACL-FALLBACK` | 2 | `ANCHOR[contextra-sys:ACL-FALLBACK-003]` |
| `ACL-SET-UNIX` | 1 | `ANCHOR[contextra-sys:ACL-SET-UNIX-002]` |
| `ACL-VERIFY-UNIX` | 1 | `ANCHOR[contextra-sys:ACL-VERIFY-UNIX-002]` |
| `ACL-WIN32-TOKEN` | 1 | `ANCHOR[contextra-sys:ACL-WIN32-TOKEN-002]` |
| `ACL-WIN32-SID` | 1 | `ANCHOR[contextra-sys:ACL-WIN32-SID-002]` |
| `ACL-SET-WIN32` | 1 | `ANCHOR[contextra-sys:ACL-SET-WIN32-002]` |
| `ACL-VERIFY-WIN32` | 1 | `ANCHOR[contextra-sys:ACL-VERIFY-WIN32-002]` |
| `ACL-WIN32-SECDESC` | 1 | `ANCHOR[contextra-sys:ACL-WIN32-SECDESC-002]` |
| `ACL-TESTS` | 1 | `ANCHOR[contextra-sys:ACL-TESTS-002]` |
| `ACL-TEST-ROUNDTRIP` | 1 | `ANCHOR[contextra-sys:ACL-TEST-ROUNDTRIP-002]` |
| `ACL-TEST-NONEXISTENT` | 1 | `ANCHOR[contextra-sys:ACL-TEST-NONEXISTENT-002]` |
| `ACL-TEST-SET-NONEXISTENT` | 1 | `ANCHOR[contextra-sys:ACL-TEST-SET-NONEXISTENT-002]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-sys:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-sys` durch (`find crates/contextra-sys -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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
