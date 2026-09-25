# Capability Marker Generator & Drift Gate

Dieses Dokument beschreibt den automatisierten Mechanismus zur Erzeugung von Capability- und Reifegrad-Markern (`docs/generated/capability-markers.md`) aus der zentralen `capabilities.toml` (ADR-N08 / Spec P12).

## Hintergrund & Zweck

Laut `CONTEXTRA_SPEC_UPDATED.md` (Abschnitt A.2, Prinzip P12/§20) dienen `capabilities.toml` und die daraus abgeleiteten Dokumente als Single Source of Truth für den Reifegrad und die Eigenschaften aller Workspace-Crates.

Um Dokumentations-Drift und manuelle Fehler bei Reifegrad-Symbolen (🟢, 🟡, 🔴) zu vermeiden, generiert `cargo xtask generate-markers` die Markdown-Tabelle maschinell aus den in `capabilities.toml` hinterlegten `maturity`- und `capabilities`-Feldern.

## Zuordnungsregeln für Reifegrad-Marker

Die Reifegrad-Symbole werden anhand des Feldes `maturity` in `capabilities.toml` ermittelt:

| `maturity`-Wert | Symbol & Marker | Bedeutung |
| :--- | :--- | :--- |
| `"stable"` | 🟢 stable | Vollständig implementiert und stabil |
| `"experimental"` | 🟡 experimental | Teilweise implementiert / Feature-gated |
| `"deprecated"` | 🔴 deprecated | Veraltet / Veraltet-Markiert |
| *Sonstiges / Fehlt* | 🔴 unklassifiziert | Fehlt oder noch unklassifiziert |

## Subkommandos in `xtask`

1. **Generierung**:
   ```bash
   cargo xtask generate-markers [--output <pfad>]
   ```
   Liest `capabilities.toml` und schreibt die erzeugte Markdown-Tabelle standardmäßig nach `docs/generated/capability-markers.md`.

2. **Drift-Prüfung**:
   ```bash
   cargo xtask check-marker-drift [--output <pfad>]
   ```
   Vergleicht den aktuellen Inhalt von `capabilities.toml` mit der generierten Datei `docs/generated/capability-markers.md`. Schlägt fehl, falls Abweichungen festgestellt werden (z. B. bei manuellen Edits an der Markdown-Datei oder unbeabsichtigten Änderungen in `capabilities.toml`).

## CI-Integration

Das Subkommando `check-marker-drift` ist als CI-Gate in `.github/workflows/merge-gate.yml` unter dem Job `marker-drift-gate` eingebunden und blockiert Pull Requests bei Schema- oder Marker-Drift.
