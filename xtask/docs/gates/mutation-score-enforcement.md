# Mutation Score Enforcement Gate (`check-mutation-score-gate`)

Das Subkommando `cargo run -p xtask -- check-mutation-score-gate --crate <NAME>` dient als technisches Gate zur Durchsetzung von Mindest-Mutation-Scores gemäß **`CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` §21 & §22.3**.

---

## Zweck & Spezifikationsbezug

- **§21 (Tier 0 & Tier 1 Qualitätskriterien)**:
  Spezifiziert explizit einen **Mutation Score von mindestens 70.0 %** für Tier 0 (`contextra-crypto`) und Tier 1 (`contextra-store`, `contextra-checkpoint`).
- **§22.3 (Keine Absichtserklärung ohne Gate)**:
  Bestimmt, dass Qualitätskriterien technisch erzwungen werden müssen. Statt Mutation-Scores nur in der Historie (`docs/mutation_score_history.jsonl`) aufzuzeichnen, blockiert `check-mutation-score-gate` Pull Requests oder CI-Läufe, wenn der gemessene Mutation-Score unter dem definierten Schwellwert liegt.

---

## Funktionsweise & Crate-Klassifikation

Das Gate liest den **zuletzt für den jeweiligen Crate-Namen aufgezeichneten Eintrag** in `docs/mutation_score_history.jsonl` und führt folgende Prüfung durch:

$$\text{score\_pct} = \frac{\text{caught}}{\text{total\_mutants}} \times 100.0$$

### Schwellwert-Zuordnung (`get_mutation_threshold_for_crate`)

| Crate | Tier | Schwellwert | Verhalten |
| :--- | :---: | :---: | :--- |
| `contextra-crypto` | Tier 0 | **70.0 %** | Hard Gate (`Err` bei Unterschreitung) |
| `contextra-store` | Tier 1 | **70.0 %** | Hard Gate (`Err` bei Unterschreitung) |
| `contextra-checkpoint` | Tier 1 | **70.0 %** | Hard Gate (`Err` bei Unterschreitung) |
| *Alle weiteren Crates* (Tier 2/3) | Tier 2/3 | *Kein Schwellwert* (`None`) | Informational (`Ok` mit `informational: true`) |

---

## Controlled `examine_globs` Expansion

In `.cargo/mutants.toml` ist `examine_globs` kontrolliert auf Tier 0/1 Crates eingegrenzt:

```toml
examine_globs = [
    "crates/contextra-crypto/src/**/*.rs",
    "crates/contextra-store/src/**/*.rs",
    "crates/contextra-checkpoint/src/**/*.rs",
]
```

---

## Befund & Dokumentation zu `test_package` in `.cargo/mutants.toml` (Punkt 6 Analysis)

### Befund

In `.cargo/mutants.toml` existiert aktuell folgende Zeile:
```toml
test_package = [
    "contextra-crypto",
]
```

### Auswirkung & Analyse
Wenn `cargo mutants -p <crate>` für z. B. `contextra-store` aufgerufen wird, überschreibt das CLI-Argument `-p contextra-store` prinzipiell das Mutieren auf das angegebene Paket. Wenn jedoch `test_package` in `.cargo/mutants.toml` fest vorgegeben ist, beschränkt `cargo-mutants` die Testausführung zur Feststellung von gefangenen Mutanten ("caught") auf die in `test_package` aufgelisteten Testpakete.

Das bedeutet: Beim Mutieren von `contextra-store` wird gegen die Testsuite von `contextra-crypto` getestet. Da `contextra-crypto` keine Tests für `contextra-store`-Code enthält, würden Mutanten in `contextra-store` als unentdeckt ("missed") gewertet.

### Empfohlene Folgemaßnahme (In eigenständiger Task zu beheben)
`test_package` sollte aus der globalen `.cargo/mutants.toml` entfernt werden, damit `cargo-mutants` standardmäßig das Paket testet, das per `-p <crate>` mutiert wird (oder per pro-crate `.cargo/mutants.toml` in den jeweiligen Crate-Verzeichnissen konfiguriert werden). Dieser Befund ist hiermit klar benannt und dokumentiert.
