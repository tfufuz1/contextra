# Routine: Abhängigkeitshygiene

- **Takt:** wöchentlich
- **Ziel:** Lizenzen, Sicherheitsberichte und Matrixkonformität neuer oder aktualisierter Abhängigkeiten prüfen (SPEC 5.3, D-54, HR-1)
- **Eingaben:** `cargo deny check`, `cargo tree -e normal` (darf keine Netzwerk-Bibliothek enthalten). `cargo vet` setzt ein Verzeichnis `supply-chain/` voraus, das im Repository fehlt; ohne Einrichtung entfällt dieser Schritt.
- **Harte Grenzen:** Scope `Cargo.toml`, `Cargo.lock`, `deny.toml`. Jede Änderung von `Cargo.toml` erneuert `Cargo.lock` im selben Commit (D-93).
- **Stopp:** Unzulässige Lizenz ohne freigegebene Ausnahme. Ausnahmen in `deny.toml` brauchen ein Ablaufdatum (SPEC 17.3).
- **Ergebnis:** PR mit Korrektur oder Bericht über die Befunde.
- **Eskalation:** siehe [`../HARNESS_QUICKSTART.md`](../HARNESS_QUICKSTART.md).
