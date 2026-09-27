# VETO Review Deadline CI Gate (`docs/ci/VETO_DEADLINE_GATE.md`)

## Zweck des Gates
Das **VETO Review Deadline Gate** (`cargo xtask check-veto-deadlines`) überwacht das Feature-Veto-Register (`VETOES.md`) im Repository-Root auf bedingt akzeptierte Feature-Entscheidungen (`status: conditionally_accepted`). Es verhindert, dass befristete VETO-Bedingungen und deren Review-Fristen stillschweigend verstreichen.

## Geprüfte Felder in `VETOES.md`
Das Gate parst Blöcke mit `## VETO-<ID>`-Überschriften und folgenden Zeilenfeldern:
- `feature_id`: ID des betroffenen Features (z. B. `F-02`, `OP-03`).
- `status`: Derzeitiger Status. Das Gate prüft ausschließlich Einträge, deren Status mit `conditionally_accepted` beginnt.
- `conditional_review_due`: Ablauffrist der bedingten Akzeptanz im ISO-Format `YYYY-MM-DD` (z. B. `2026-10-07`).
- `adr_ref`: Referenz auf das zugrundeliegende ADR (z. B. `DECISIONS.md#adr-077`).

## Verhalten & Klassifikation
Beim Ausführen vergleicht das Gate das aktuelle Utc-Datum mit `conditional_review_due`:
1. **Frist überschritten (Resttage < 0):**
   Das Gate bricht mit einem harter CI-Fehler (`Err(...)`) ab und listet alle überfälligen VETO-IDs auf.
2. **Warnfenster (0 ≤ Resttage ≤ 14):**
   Kein Fehler, aber das Gate gibt die betroffenen Einträge als Warnung aus, um rechtzeitig auf die bevorstehende Review-Frist aufmerksam zu machen.
3. **Zukunft (Resttage > 14):**
   Keine Ausgabe oder Warnung.

## Maßnahmen bei Ablauf oder Herannahen der Frist
Wenn eine Frist abläuft oder sich im Warnfenster befindet, müssen folgende Schritte durchgeführt werden:
1. **Frist verlängern:** Die VETO-Bedingungen formal in `DECISIONS.md` re-evaluieren und das Feld `conditional_review_due` in `VETOES.md` per neuem ADR verlängern.
2. **Status abschließen:** Falls die Bedingung nicht mehr befristet gilt, den `status` in `VETOES.md` entweder auf `permanent_rejected` oder einen endgültig akzeptierten Status ändern.

## Aktueller Kontext
Aktuell stehen folgende bedingt akzeptierten VETOs zur Review an:
- `VETO-F02` (feature_id: `F-02`, adr_ref: `DECISIONS.md#adr-077`) — Frist: **2026-10-07**
- `VETO-OP03` (feature_id: `OP-03`, adr_ref: `DECISIONS.md#adr-077`) — Frist: **2026-10-07**
