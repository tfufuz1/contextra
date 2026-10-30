# Routinen-Verzeichnis (`.jules/routines/`)

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates, `cargo xtask` oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

## Zweck
Dieses Verzeichnis enthält die standardisierten, maschinenlesbaren Prozessbeschreibungen für alle wiederkehrenden Routine-Aufgaben im Repository gemäß Architekturdokument §9.2 und §9.3.

## Regeln
1. **Zentrale Quelle**: Wiederkehrende Jules-UI-Tasks dürfen keine eigenen, abweichenden Prompts enthalten. Ein UI-Task verweist ausschließlich auf die jeweilige Markdown-Datei in diesem Verzeichnis.
2. **PR-Integrität**: Änderungen an Routinen-Prozessen erfolgen immer als gewöhnliche Pull Requests und niemals per direktem Push auf `main`.
3. **Suggested Tasks & UI-Einstellungen**: Suggested Tasks sind in den Repository-Einstellungen deaktiviert bzw. streng auf explizite Labels mit Task-Karten beschränkt. Dies ist eine manuelle UI-Einstellung in GitHub/Jules.
4. **Ungebaute Kommandos**: Noch nicht existierende xtask-Befehle anderer paralleler Sessions sind am Zeilenende mit `<!-- harness:planned -->` markiert.
