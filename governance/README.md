# Governance in Contextra

Diese Governance-Dokumentation beschreibt die Richtlinien, Werkzeuge und Ausnahmeverfahren zur Sicherung der Code-Integrität und Supply-Chain-Sicherheit.

## Zweck
Die drei Governance-Gates verhindern, dass Menschen oder KI-Agenten Sicherheitsabsicherungen, Lints oder Tests schwächen, um rote Pipeline-Ergebnisse künstlich zu umgehen:
1. `protected-paths`: Schützt kritische Konfigurations-, CI- und Governance-Pfade vor unbefugter oder versehentlicher Änderung.
2. `gate-weakening`: Erkennt Versuche, CI-Pipelines, Lints, Test-Assertionen oder Baselines aufzuweichen (z. B. `continue-on-error: true`, Entfernen von `-D warnings`, Reduzieren von Asserts).
3. `ratchet`: Erzwingt Monotonie bei Qualitätskennzahlen (wie `.unwrap()`-Verwendung oder `unsafe`-Blöcken) und senkt die Baselines automatisch ab, wenn Werte verbessert werden.

## Ausnahmeverfahren per ADR
Niemals dürfen Baselines oder Sicherheitsregeln erhöht bzw. gelockert werden, nur um einen roten Build "grün" zu erzwingen.

Falls eine Änderung an geschützten Pfaden oder eine vorübergehende Regelanpassung architektonisch zwingend erforderlich ist:
1. Erstelle ein neues Architecture Decision Record (ADR) in `docs/decisions/ADR-NNN-*.md`.
2. Füge den passenden Git-Trailer zum Commit hinzu:
   - Für geschützte Pfade: `Protected-Change: ADR-NNN` (plus Label `protected-change` am PR)
   - Für Gate-Anpassungen / Ratchet-Erhöhungen: `Gate-Weakening: ADR-NNN` bzw. `--adr ADR-NNN`
3. Die Änderung wird vom Gate geprüft und erfordert eine explizite menschliche Review-Freigabe.

⚠️ **WARNUNG:** Das Erhöhen einer Ratchet-Baseline ohne gültiges ADR und menschliche Freigabe wird vom System als Sicherheitsverstoß gewertet und blockiert.
