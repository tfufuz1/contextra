# acta-fs

acta-fs: Vfs-Port, DurableVfs, FaultVfs, Scrub, O_DIRECT-Ruecklesen.

> Teil des Acta-Workspace. Verbindliche Vorgaben: Wurzel-`CLAUDE.md` und `docs/SPEC.md`.

## Einordnung
- **Level:** 1 (`rings.toml`)
- **Größenbudget:** 2500 LOC Produktionscode (G-HUB); Überschreitung nur mit ADR
- **Aufgabe laut SPEC 3.3:** `Vfs`-Port, `DurableVfs` (fsync, atomares Ersetzen, Scrub, `O_DIRECT`-Rücklesen), `FaultVfs` (Fehlerinjektion)
- **Erlaubte interne Abhängigkeiten (SPEC 3.3):** base
- **Ist-Abhängigkeiten (Cargo.toml):** acta-base
- **Externe Abhängigkeiten:** thiserror
- **Dev-Abhängigkeiten (zählen nicht für G1):** proptest
- **Referenz im Quelltext:** Level L1 · Budget 2500 LOC · Spezifikation docs/SPEC.md Kap. 3.7, 5.6

## Status
- **Stand:** implementiert (Teil)
- **Produktionscode (ohne Tests/Leerzeilen/Kommentare):** 437 LOC von 2500 Budget
- **Umgesetzt nach SPEC 3.7 (neu geschrieben, nicht aus dem Archiv übernommen):**
  - `vfs`: Ports `Vfs` und `VFile` mit den Signaturen aus Kap. 3.7.
  - `error`: `FsyncFailed` (Klasse `Integrity`, Code `fs.fsync_failed`).
  - `durable`: `DurableVfs` mit atomarem Ersetzen (rename + Verzeichnis-fsync, WP-5), Scrub (WP-6), `O_DIRECT`-Rücklesen auf 4096-Byte-Blöcken mit ausgerichtetem Puffer und fsync-Entwertung (WP-7). Nur hier steht `std::fs` (G-FS).
  - `fault`: `FaultVfs<V>` mit Plan (`FaultPoint`, `FaultAction`: Fehlerbild, zerrissener Schreibvorgang, Absturz).
- **Tests:** `tests/durable_vfs.rs` (8), `tests/fault_vfs.rs` (5, u. a. Absturzmatrix über Erstellen, Schreiben, Sync, Umbenennen; Fsync-Entwertung; ENOSPC; zerrissener Schreibvorgang).
- **Mutationsprüfung:** Entfernen der Fsync-Entwertung und der `O_DIRECT`-Ausrichtung lässt die jeweiligen Tests scheitern.
- **Annahmen:** `O_DIRECT`-Wert `0o40000` gilt für x86_64 und aarch64 (Linux). Auf aarch64 ist das auf Hardware nicht geprüft [A]. Andere Architekturen liefern `Unsupported`.
- **Offen:** Scrub gegen Flash-Reste bleibt Annahme A-FLASH. Journal (`durable-fs/src/journal.rs`), Gesundheitszustand und Fault-Injektor-Matrix aus dem Archiv sind nicht übernommen; sie gehören in acta-seg bzw. werden neu gebaut.
- **Archiv-Quelle:** `archive/acta-fs/` (siehe `archive/acta-fs/MANIFEST.md`). Der Archivordner bleibt bestehen, bis das Crate vollständig überführt ist.

## Regeln für diesen Crate
- Abhängigkeiten nur nach unten: `level(A) > level(B)`; keine Querkanten (G1).
- Keine Alias-Re-Exports, kein `include!`, kein `#[deprecated]` (HR-6).
- Ein Fehlertyp je Crate, `thiserror`, `#[non_exhaustive]`, kein `Other(String)` (HR-4).
- Workspace-Lints gelten (`[lints] workspace = true`): kein `unwrap`, `expect`, `panic`, Indexierung mit `[]` auf Produktionspfaden.
- `unsafe` ist hier verboten (G-UNSAFE, nur `acta-sys` darf es).
- Keine Zeit- oder Zufallsquellen außerhalb der injizierten Ports `Clock` und `CryptoRng` (G3).
- Verbotene Formulierungen aus SPEC 1.6 (Liste A) nicht verwenden (G-WORD).
- Vor Abschluss: `cargo xtask gates`, `cargo test -p acta-fs`, `cargo clippy -p acta-fs --all-targets -- -D warnings`, `cargo fmt --all -- --check`.

## Vorgehen bei Überführung aus dem Archiv
1. Archivdateien lesen, nicht blind kopieren. Namen, Importe und Verweise auf frühere Spezifikationsstände sind auf Acta umzuschreiben (siehe `archive/INDEX.md`).
2. Jede Funktion gegen den Lebenszyklus in SPEC Kap. 19 und die Zusagen in 1.5 und 16.3 prüfen. Funktionen ohne Eintrag dort werden nicht implementiert (SPEC 0.4 Regel 3).
3. Tests mit Positivkontrolle mitführen (SPEC 16.4).
4. Nach vollständiger, grün geprüfter Überführung wird der Archiv-Ordner dieses Crates entfernt.
