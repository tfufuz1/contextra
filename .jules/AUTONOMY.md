# Autonomiestufen, Freigaberegeln und Review-Schleife

Stand: 2026-10-19

## 1. Übersicht & Zielsetzung

Dieses Dokument definiert die verbindlichen Autonomiestufen für Agenten-gestützte Entwicklungs-Sessions (Jules) im Repository `contextra`. Die Differenzierung basiert auf der Risiko-Klassifizierung aus `.jules/tasks/_TEMPLATE.toml` (`risk: sec | crash | simd | utf8 | fusion | concurrency | calib | none`), den Prompter-Tiers aus `.jules/prompter-tiers.toml` sowie der Berührung geschützter Pfade (`governance/protected-paths.toml`).

---

## 2. Autonomie-Matrix (Tier & Risiko → Freigabe)

| Tier | Risikoklasse (`risk`) | Geschützte Pfade berührt? | Plan-Freigabe | Auto-Merge | Review-Pflicht | Bedingungen / Voraussetzungen |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Tier 1** | `none` | **Nein** | Automatisch | **Ja** | Keine (automatischer Merge nach grünem Gate) | PR-Gate-Check (`gate`) ist vollständig grün. Diff liegt strikt innerhalb des Task-Budgets (`budget.files`, `budget.lines`). |
| **Tier 1** | `none` | **Ja** | Manuell | **Nein** | Manuell | Freigabe des Owners (`@tfufuz1`) sowie ADR-Trailer `Protected-Change: ADR-N34`. |
| **Tier 1** | `!= none` (`sec`, `crash`, `simd`, etc.) | Egal | Manuell | **Nein** | Manuell | Vorab-Planfreigabe erforderlich. Manuelles Review & Merge durch Maintainer. |
| **Tier 2** | `none` | **Nein** | Automatisch | **Nein** | Manuell | Plan wird automatisch freigegeben. Merge erfordert manuelles Review/Approval durch Maintainer. |
| **Tier 2** | `!= none` oder Geschützt | Egal | Manuell | **Nein** | Manuell | Vorab-Planfreigabe erforderlich. Manuelles Review & Merge durch Maintainer. |
| **Tier 3** | Alle (`none` oder `!= none`) | Egal | Manuell | **Nein** | Manuell | Höchste Aufmerksamkeitsstufe. Sowohl Plan als auch PR erfordern explizite menschliche Prüfung. |

---

## 3. PR-Überarbeitungs-Schleife (Iteratives Feedback)

Erfordert ein bestehender Pull Request Korrekturen (z. B. durch fehlgeschlagene CI-Gates, automatische Verdicts oder Review-Kommentare des Maintainers), arbeitet der Agent nach folgenden Regeln:

1. **Sticky Verdict-Kommentar & Review-Analyse:**
   - Der Agent liest den **sticky Verdict-Kommentar** am PR sowie eventuelle Inline-Review-Kommentare.
   - Es wird kein neuer PR eröffnet; alle Änderungen erfolgen direkt auf dem bestehenden Feature-Branch.

2. **Iterative Anpassung:**
   - Der Agent behebt die identifizierten Mängel und pusht die Korrekturen.
   - Der Re-Test wird automatisch oder durch einen erneuten Trigger angestoßen.

3. **Maximale Iterationsanzahl & Escalation:**
   - Die Überarbeitungsschleife ist auf **maximal 3 Iterationen** pro PR beschränkt.
   - Bricht das Gate auch nach der 3. Iteration fehl oder kann der Agent die Anmerkungen nicht auflösen, stoppt die automatische Bearbeitung.
   - Der Agent setzt das GitHub-Label **`jules:needs-human`** am PR und informiert den Maintainer (`@tfufuz1`).
