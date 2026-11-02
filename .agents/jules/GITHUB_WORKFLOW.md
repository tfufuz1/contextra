# GitHub als Arbeitskontext

Nicht normativ. Maßgeblich: [`CONTRIBUTING.md`](../../docs/CONTRIBUTING.md) und [`SPEC.md`](../../docs/SPEC.md) Abschnitt 15.4.

## Grundsatz

- **Issues** sind Aufgaben (Task-Karten).
- **Milestones** entsprechen den Arbeitspaketen aus SPEC 15.4 bzw. Gates aus 15.3.
- **Pull Requests** sind Reviews und Dokumentation der Änderung.
- **CI-Läufe** sind Signale. Ihr Status ist maßgeblich, nicht die Beschreibung im Issue.
- Statische Dateien im Repository enthalten nur unveränderliche Regeln, niemals Projektstatus.

## Labels

Der Bestand an Labels im Repository ist in dieser Revision nicht geprüft. Das folgende Schema beschreibt die Absicht:

| Label | Bedeutung | Verhalten |
|---|---|---|
| `jules` | Aufgabe für Jules | aufnehmen |
| `priority-p0` | Blocker | zuerst beheben |
| `bug` | Defekt | reproduzieren, korrigieren, testen |
| `enhancement` | Verbesserung | Vetos prüfen (`docs/VETOES.md`), dann umsetzen |
| `task` | Arbeitsauftrag | nach Beschreibung abarbeiten |
| `claimed` | von einem Agenten reserviert | nicht doppelt bearbeiten |
| `claim:<crate>` | crate-spezifische Reservierung | nur dieses Crate bearbeiten |
| `question` | Klärung nötig | kommentieren, nicht raten |
| `wontfix`, `invalid` | nicht bearbeiten | überspringen |

## Aufbau einer Aufgabe

```
Titel: <verb>(<scope>): <was>

## Ziel
<was danach anders sein soll>

## Akzeptanzkriterien
- [ ] <prüfbares Kriterium, mit SPEC-ID, z. B. T-…>

## Scope
crates/<crate>/

## Vetos
keine oder: VETO-… geprüft, nicht betroffen

## Außerhalb des Umfangs
<was nicht angefasst wird>
```

## Verknüpfung

Der Pull Request nennt das Arbeitspaket (`WP-x.y`), die betroffenen SPEC-IDs und das Issue (`Closes #N`). Die Automatisierung von Issues regelt [`ISSUE_AUTOMATION_POLICY.md`](ISSUE_AUTOMATION_POLICY.md).
