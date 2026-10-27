# Offene Fragen & Kontext-Befunde (xtask Inventar)

## 1. Wer liest `.agents/governance/*`?

### System-weiter Scan
Befehl:
```bash
rg "\.agents/governance" xtask/ .github/ .agents/ justfile .githooks/
```

### Befund
Ein direkter Scan über den gesamten Workspace zeigt **0 Treffer** für den Pfad `.agents/governance`.
Die Verweisstruktur in `justfile` bezieht sich stattdessen auf `governance/gates.toml` und `governance/*`.
Der Ordner `.agents/governance/*` existiert derzeit nicht im Dateisystem.

---

## 2. Wo liegt der ADR-Index?

### System-weiter Scan
Befehl:
```bash
rg "ADR-Index|docs/decisions" docs/ README.md
```

### Befund
- Laut `docs/SOURCE_OF_TRUTH.md` ist der normative Pfad für den ADR-Index als **`docs/decisions/`** definiert.
- Das Verzeichnis `docs/decisions/` existiert in diesem Stand des Repositories noch nicht als physisches Verzeichnis.
- In `docs/PLAN.md` §0.3 (WP 0.3) ist das Anlegen von `docs/decisions/` und das Füllen des ADR-Index für D-01 bis D-14 als ausstehendes Arbeitspaket dokumentiert.
