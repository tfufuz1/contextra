@AGENTS.md

# GitHub Copilot — Spezifische Regeln

Dieses File importiert alle Regeln aus `@AGENTS.md`. Hier stehen NUR Copilot-spezifische Ergänzungen.

## Completion-Richtlinien
- Keine Completions mit `.unwrap()` oder `.expect()` außerhalb von `#[cfg(test)]` vorschlagen.
- `unsafe`-Completions nur in `contextra-index/src/distance.rs`, `contextra-index/src/diskann.rs`, `contextra-index/src/persistence.rs` und stets mit `// SAFETY:`-Beweis.
- DAG-Schichtgrenzen (§2) bei Import-Vorschlägen einhalten.
