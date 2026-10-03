# Gate: CODEOWNERS Coverage (`codeowners-coverage`)

Das Gate `codeowners-coverage` verifiziert, dass alle in `governance/protected-paths.toml` geschützten Dateipfade und Globs durch entsprechende Einträge in `CODEOWNERS` abgedeckt sind und keine verwaisten Phantom-Pfade in `CODEOWNERS` existieren.

## Funktionsweise
1. Liest alle geschützten Pfad-Globs aus `governance/protected-paths.toml`.
2. Prüft, ob jedes geschützte Glob durch mindestens ein Pattern in `CODEOWNERS` abgedeckt wird.
3. Analysiert alle Patterns in `CODEOWNERS` und schlägt fehl, wenn ein Pattern auf keine existierende Datei oder Verzeichnis matchet (Phantom-Pfade).

## Ausführung
```bash
cargo xtask codeowners-coverage [--json]
```
