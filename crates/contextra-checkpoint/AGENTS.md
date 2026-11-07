# AGENTS.md — `contextra-checkpoint`

> **Ring 1** · Klasse `[K]` · Reifegrad `stable` · Unsafe: nein · Rolle: Benannte Checkpoints, RAII-`CheckpointGuard`, Orphan-Reaping, Manifest-Verifikation (BLAKE3).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-checkpoint --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `mvcc` |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- Normalkanten sauber (`ports`, `types`). Dev-Kanten beschränkt auf `contextra-testkit`, `criterion`, `proptest`, `serde_json`, `tempfile` (M6: `store` und `mvcc` vollständig entfernt).
- `capabilities.toml` nennt noch `contextra-core`: entfernen.
- Direkte Datei-I/O: `orphan.rs` unterstützt nun `Vfs` (`persist_vfs`, `load_vfs`).
- Zugriff auf den Store nur über `StorageEngine`-Port, nie über `contextra_store`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `guard.rs` | `CheckpointGuard<S>`: commit/rollback, Drop registriert Orphan |
| `store.rs` | `PersistentCheckpointStore`: create/restore/drop, `allocate_tx`, Snapshot-Pinning |
| `manifest.rs`, `meta.rs` | signiertes/geprüftes Manifest, Metadaten |
| `orphan.rs` | Reaper für verwaiste Checkpoints |
| `hardlink_cloner.rs` | Hardlink-Klone für Snapshots |

## 4. Verträge & Invarianten

1. **Guard wird konsumiert:** `commit()` oder `rollback()`; `Drop` kann nicht async, deshalb Orphan-Registrierung und späterer Reaper (`recover_orphaned_checkpoints`).
2. **Pinning:** gepinnte Checkpoints verhindern Version-GC im Store; beim Löschen unpinnen.
3. **Restore bringt Gelöschtes nicht zurück:** Restore läuft hinter dem `RestoreGuard` von `contextra-erasure` (INV-NO-RESURRECTION-1, INV-RESTORE-HORIZON-1). `checkpoint` selbst kennt keine Löschlogik.
4. Zeit/IDs über `Clock`/`IdGen` (Tests `checkpoint_clock_determinism`, `id_gen_determinism_test`).
5. Manifest mit Prüfsumme; manipuliertes Manifest ⇒ Ablehnung (`manifest_fault_injection.rs`).

## 5. Ports

**Implementiert:** `CheckpointCoordinator`/`Checkpoint`. **Konsumiert:** `StorageEngine`, `Clock`, `IdGen`, `Vfs` (Soll).

## 6. Nebenläufigkeit & Performance

- Offene Guards halten Schreibpuffer und blockieren das MVCC-Watermark: kurz halten, nie über lange `.await`-Ketten.
- Concurrency-Tests: `cache_concurrency_pinning.rs`, `concurrency.rs`.

## 7. Anti-Patterns

- Guard vor langem `.await` anlegen.
- `SystemTime` als `TxId`.
- `contextra_store::checkpoint` importieren (existiert intern, nicht öffentlich).

## 8. Tests & Messung

```bash
cargo test -p contextra-checkpoint --locked
cargo clippy -p contextra-checkpoint --all-targets -- -D warnings
cargo bench -p contextra-checkpoint
```

- Vorhanden: systematische Crash-Tests, Tenant-Isolation, Zeitreise-Korrektheit, Proptests für Guard.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 4.6 | Backup/Snapshot isolieren, Legacy-Löschmarken bereinigen, autarker Wiederanlauf-Test |
| M6 | Dev-Kante `store` entfernen |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-checkpoint:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-checkpoint:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-checkpoint:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `ORP` | 12 | `ANCHOR[contextra-checkpoint:ORP-013]` |
| `GRD` | 11 | `ANCHOR[contextra-checkpoint:GRD-012]` |
| `STO` | 11 | `ANCHOR[contextra-checkpoint:STO-012]` |
| `CLONE` | 5 | `ANCHOR[contextra-checkpoint:CLONE-006]` |
| `MAN` | 3 | `ANCHOR[contextra-checkpoint:MAN-004]` |
| `META` | 2 | `ANCHOR[contextra-checkpoint:META-003]` |
| `MOD` | 1 | `ANCHOR[contextra-checkpoint:MOD-002]` |
| `VAL` | 1 | `ANCHOR[contextra-checkpoint:VAL-002]` |
| `STATE` | 1 | `ANCHOR[contextra-checkpoint:STATE-002]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-checkpoint:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-checkpoint` durch (`find crates/contextra-checkpoint -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

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
