# VETO Review Deadline CI Gate (`docs/ci/VETO_DEADLINE_GATE.md`)

## Zweck des Gates
Das **VETO Review Deadline Gate** (`cargo xtask check-veto-deadlines` bzw. `check-vetoes`) überwacht das Feature-Veto-Register (`VETOES.md`) im Repository-Root auf bedingt akzeptierte Feature-Entscheidungen (`status: conditionally_accepted`). Es verhindert, dass befristete VETO-Bedingungen und deren Review-Fristen stillschweigend verstreichen.

## Geprüfte Felder in `VETOES.md`
Das Gate parst Blöcke mit `## VETO-<ID>`-Überschriften und folgenden Zeilenfeldern:
- `feature_id`: ID des betroffenen Features (z. B. `F-02`, `OP-03`).
- `status`: Derzeitiger Status. Das Gate prüft ausschließlich Einträge, deren Status mit `conditionally_accepted` beginnt.
- `conditional_review_due`: Ablauffrist der bedingten Akzeptanz im ISO-Format `YYYY-MM-DD` (z. B. `2026-10-07`).
- `adr_ref`: Referenz auf das zugrundeliegende ADR unter `docs/decisions/` (z. B. `docs/decisions/ADR-077-produktvision-pypi-library-fokus-und.md`).
- `affected_paths`: Optionales Array von Pfaden/Crates, die diesem Feature zugeordnet sind.

## Verhalten & Scoped-Blocking-Logik
Beim Ausführen vergleicht das Gate das aktuelle Datum mit `conditional_review_due`:
1. **Frist überschritten (Resttage < 0):**
   - Falls ein PR geänderte Dateien enthält, die den `affected_paths` oder `keywords` des abgelaufenen VETOs entsprechen, bricht das Gate mit einem harten CI-Fehler (`Exit != 0`) ab.
   - Falls der PR das betroffene Feature **nicht** berührt, erzeugt das Gate eine Warnung und fordert zur Erstellung eines Review-Tickets auf (kein harter Abbruch).
2. **Warnfenster (0 ≤ Resttage ≤ 14):**
   Kein Fehler, aber das Gate gibt die betroffenen Einträge als Warnung aus, um rechtzeitig auf die bevorstehende Review-Frist aufmerksam zu machen.
3. **Zukunft (Resttage > 14):**
   Keine Ausgabe oder Warnung.

## Verlängerung von Fristen
Eine Verlängerung von Fristen akzeptiert das Gate nur, wenn das referenzierte ADR bereits auf dem Basis-Branch existiert. Eine Datumsänderung im selben PR ohne bereits auf dem Basis-Branch existierendes ADR wird abgelehnt.

## Aktueller Kontext
Aktuell stehen folgende bedingt akzeptierten VETOs zur Review an:
- `VETO-F02` (feature_id: `F-02`, adr_ref: `docs/decisions/ADR-077-produktvision-pypi-library-fokus-und.md`) — Frist: **2026-10-07**
- `VETO-OP03` (feature_id: `OP-03`, adr_ref: `docs/decisions/ADR-077-produktvision-pypi-library-fokus-und.md`) — Frist: **2026-10-07**
