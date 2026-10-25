# Governance in Contextra

Diese Governance-Dokumentation beschreibt die Richtlinien, Werkzeuge und Ausnahmeverfahren zur Sicherung der Code-Integrität und Supply-Chain-Sicherheit.

## Zweck
Die drei Governance-Gates verhindern, dass Menschen oder KI-Agenten Sicherheitsabsicherungen, Lints oder Tests schwächen, um rote Pipeline-Ergebnisse künstlich zu umgehen:
1. `protected-paths`: Schützt kritische Konfigurations-, CI- und Governance-Pfade vor unbefugter oder versehentlicher Änderung.
2. `gate-weakening`: Erkennt Versuche, CI-Pipelines, Lints, Test-Assertionen oder Baselines aufzuweichen (z. B. `continue-on-error: true`, Entfernen von `-D warnings`, Reduzieren von Asserts).
3. `ratchet`: Erzwingt Monotonie bei Qualitätskennzahlen (wie `.unwrap()`-Verwendung oder `unsafe`-Blöcken) und senkt die Baselines automatisch ab, wenn Werte verbessert werden.

## Governance-Dateien und Zuständigkeiten

### `constitution.toml`
`constitution.toml` führt die maschinenlesbare Verfassung für Contextra (Prinzipien, Vetos und Verfassungs-Doktrinen nach Spezifikation 1.3) inklusive Durchsetzungs-Gates. Darf nur mit freigegebenem ADR (`docs/decisions/`) und Genehmigung der Lead-Architekten bzw. `@contextra/build-infra` geändert werden.

### `determinism-allow.toml`
`determinism-allow.toml` führt dokumentierte Ausnahmen vom strikten Determinismus-Verbot (z. B. für kryptografische Schlüssel- und Salt-Generierung per System-RNG). Darf nur durch `@contextra/build-infra` mit expliziter ADR-Referenz angepasst werden.

### `determinism-baseline.toml`
`determinism-baseline.toml` führt Zähler und Ausnahmen der Nicht-Determinismus-Baseline für den Codebase-Ratchet. Darf bei Verbesserungen automatisch abgesenkt oder durch `@contextra/build-infra` mit gültigem ADR angepasst werden.

### `gates.toml`
`gates.toml` führt den zentralen Gate-Katalog für Qualitäts-, Sicherheits- und Governance-Gates inklusive Risikoklassen, Phasen und Owner-Zuordnung. Darf nur von `@contextra/build-infra` nach Architektur-Freigabe geändert werden.

### `protected-paths.toml`
`protected-paths.toml` führt die Pfad-Muster geschützter Dateien und Verzeichnisse, die vor unbefugter Modifikation geschützt sind. Ähnlich wie CI-Workflows darf diese Datei nur von `@contextra/build-infra` mit dem Git-Trailer `Protected-Change: ADR-NNN` geändert werden.

### `ratchet.toml`
`ratchet.toml` führt die Monotonie-Baselines für Qualitätskennzahlen wie unwrap-, expect- und unsafe-Anzahlen. Darf sich im regulären Betrieb automatisch verringern (Ratchet-Prinzip); Erhöhungen dürfen ausschließlich von `@contextra/build-infra` mit dem Trailer `Gate-Weakening: ADR-NNN` vorgenommen werden.

### `unsafe-safety-baseline.toml`
`unsafe-safety-baseline.toml` führt die Erfassungs-Baseline für fehlende Safety-Dokumentationskommentare an `unsafe`-Stellen im Codebase-weit. Anpassungen unterliegen dem Ratchet-Prinzip und sind `@contextra/build-infra` mit ADR vorbehalten.

### `verdict-required.toml`
`verdict-required.toml` führt die Konfiguration des Verdict-Harnesses zur Definition blockierender Statuscheck-Gates vor PR-Merges. Darf nur von `@contextra/build-infra` geändert werden.

## Ausnahmeverfahren per ADR
Niemals dürfen Baselines oder Sicherheitsregeln erhöht bzw. gelockert werden, nur um einen roten Build "grün" zu erzwingen.

Falls eine Änderung an geschützten Pfaden oder eine vorübergehende Regelanpassung architektonisch zwingend erforderlich ist:
1. Erstelle ein neues Architecture Decision Record (ADR) in `docs/decisions/ADR-NNN-*.md`.
2. Füge den passenden Git-Trailer zum Commit hinzu:
   - Für geschützte Pfade: `Protected-Change: ADR-NNN` (plus Label `protected-change` am PR)
   - Für Gate-Anpassungen / Ratchet-Erhöhungen: `Gate-Weakening: ADR-NNN` bzw. `--adr ADR-NNN`
3. Die Änderung wird vom Gate geprüft und erfordert eine explizite menschliche Review-Freigabe.

⚠️ **WARNUNG:** Das Erhöhen einer Ratchet-Baseline ohne gültiges ADR und menschliche Freigabe wird vom System als Sicherheitsverstoß gewertet und blockiert.
