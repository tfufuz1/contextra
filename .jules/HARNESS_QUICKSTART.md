# Harness Quickstart

10-Zeilen-Spickzettel für die 5 Jules-Phasen. Siehe [PREAMBLE.md](PREAMBLE.md) für den Sitzungsvertrag.

```bash
# 1. Session starten (Karte linten, Claim setzen, Kontext packen)
cargo xtask jules start --card .jules/tasks/T-2026-0001.toml

# 2. Während der Arbeit Syntax, Scope, Pfade & Duplikate prüfen
cargo xtask jules check

# 3. Akzeptanz-Tests & Architektur-Gates verifizieren
cargo xtask jules verify

# 4. Finales Submit-Gate und PR-Bericht
cargo xtask jules submit

# 5. Im Fehlerfall / Abbruch
cargo xtask jules stop --reason <grund>
```
