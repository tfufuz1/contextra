# Fassade: Jules (`jules`)

## Zweck
`jules` bietet eine 5-Phasen-Fassade über alle Einzelkommandos des Harnesses.

## Befehle
- `cargo xtask jules start --card <DATEI>`
- `cargo xtask jules check`
- `cargo xtask jules verify`
- `cargo xtask jules submit`
- `cargo xtask jules stop --reason <GRUND>`

## Konfiguration
Die Zuordnung Phase -> Einzelkommandos ist dynamisch in `.jules/harness-phases.toml` konfiguriert. Missing required commands lead to fail-closed Exit 2 with explicit session ownership hints.

### Optionale Step-Felder
Steps unterstützen folgende Steuerungsparameter (`#[serde(default)]`):
- `parallel_group: Option<String>`: Aufeinanderfolgende Schritte derselben Phase mit gleicher `parallel_group` laufen parallel in `std::thread::scope`. Ausgaben werden gepuffert und in TOML-Reihenfolge dargestellt. Alle Schritte einer Gruppe laufen zu Ende (kein Early-Exit).
- `when_changed: Option<Vec<String>>`: Pfad-Globs. Ein Schritt wird übersprungen (`SKIPPED(kein passender Pfad im Diff)`), falls keine geänderte Datei im Git-Diff (merge-base..Worktree inkl. untracked) ein Glob trifft. Ist die Diff-Basis nicht ermittelbar, wird der Schritt fail-safe ausgeführt.
- `serial: bool`: Erzwingt sequentielle Ausführung, auch innerhalb einer parallelen Gruppe.

### Binäraufruf & Platzhalter
- **Binäraufruf-Optimierung**: Schritte mit `cmd[0..2] == ["cargo", "xtask"]` werden direkt über das laufende `xtask`-Binary (`std::env::current_exe()`) ausgeführt. Fallback auf den bisherigen Aufruf, falls `current_exe()` fehlschlägt.
- **Platzhalter `{pkgs}`**: Expandiert in Argumenten zu `-p <crate>` für jedes betroffene Crate laut `blast-radius` (transitive Abhängigkeiten von geänderten Dateien). Falls Wurzeldateien (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `capabilities.toml`) geändert wurden, keine Crates betroffen sind oder `--full` / `JULES_FULL=1` gesetzt ist, expandiert `{pkgs}` zu `--workspace`.
- **Laufzeitmessung (`--timings`)**: Gibt Ausführungsdauern je Schritt und je Phase in der Text- und JSON-Zusammenfassung aus (`duration_ms`, `duration_str`).
