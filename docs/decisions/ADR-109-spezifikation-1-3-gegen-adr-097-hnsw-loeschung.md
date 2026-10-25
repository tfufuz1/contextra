# ADR-109: Spezifikation 1.3 gegen ADR-097 (HNSW-Löschung)

* **Status:** Proposed
* **Datum:** 2026-10-25
* **Autoren:** Principal Architecture Governance Engineer (Contextra)
* **Betroffene Komponenten:** `crates/contextra-vector`, `docs/CONTEXTRA_SPEZIFIKATION.md`, `VETOES.md`, `docs/decisions/ADR-097-*.md`

---

## 1. Kontext & Konfliktbeschreibung

Zwischen den normativen Dokumenten und Entscheidungsakten der Contextra-Architektur besteht ein direkter, unvereinbarer Konflikt bezüglich der Handhabung gelöschter HNSW-Vektorknoten:

1. **Spezifikation 1.3 (Dokument A, SSOT):**
   - In **§4.1 (Vetos)**, **§9.3 (Löschung)** sowie **REQ-IDX-003** ist das Veto **VETO-F02** verankert:
     > *"HNSW-Löschung MUSS Tombstone-Pruning sein; kein partielles Rewiring des Graphen (VETO-F02)."*
   - Dieses Veto ist als absolut unverletzliche Systeminvariante eingestuft, um unbeabsichtigte Recall-Kollapse, komplexe RwLock-Contention und nicht-deterministisches Graph-Verhalten im produktiven Kern zu verhindern.

2. **ADR-097 (Accepted am 2026-09-27) & VETOES.md:**
   - In ADR-097 wurde beschlossen, dass `remove_with_graph_repair` (Nachbarschaftsreparatur/Rewiring) das reine Tombstone-Pruning ersetzt.
   - In `VETOES.md` wurde VETO-F02 unter dem Status `conditionally_accepted` mit einer Überprüfungsfrist bis zum 2026-10-07 eingetragen.

---

## 2. Entscheidung

Gegenstand der Entscheidung im Rahmen von Auftrag MP-00:

1. **Rangfolge & Vorrang der Spezifikation 1.3:**
   Gemäß den Governance-Regeln (Dokument C §1.4 / W-04, Spezifikation §28.5) gilt die Gesamtspezifikation (Fassung 1.3) als oberste Quelle der Wahrheit (SSOT).
2. **ADR-097 Status-Klassifikation:**
   ADR-097 steht im Widerspruch zu VETO-F02 der Spezifikation 1.3. Nach der Freigabe dieses ADRs durch den Auftraggeber wird ADR-097 formal den Status **Superseded** erhalten (durchgeführt durch MP-16 nach Freigabe).
3. **Beibehaltung von VETO-F02 (Tombstone-Pruning):**
   Die HNSW-Löschung erfolgt im Kern strikt nach VETO-F02 ausschließlich via Tombstone-Pruning. Es findet kein partielles Rewiring statt.
4. **Keine Änderung von ADR-097 im aktuellen Schritt:**
   Die Datei `docs/decisions/ADR-097-hnsw-nachbarschaftsreparatur-bei-loeschung-ersetzt-reines-tombstone-pruning.md` selbst wird in MP-00 unberührt gelassen. Die Statusänderung erfolgt im Zuge von MP-16 nach der expliziten Freigabe durch den Auftraggeber.

---

## 3. Governance-Regel für Veto-Änderungen

Ein Invarianten-Veto (`VETO-*`) ist ein zentraler Pfeiler der Produktarchitektur. Eine Abweichung oder Aufhebung eines Vetos erfordert zwingend:
1. Ein eigenständiges, formal ausgearbeitetes ADR.
2. Eine explizite, kryptografisch oder dokumentarisch signierte Freigabe durch den Auftraggeber / Lead Architect.

Ohne diese zweistufige Freigabe bleibt jedes Veto in vollem Umfang in Kraft.

---

## 4. Konsequenzen

- Der Vektorindex (`crates/contextra-vector`) verbleibt im deterministischen Tombstone-Pruning-Modus.
- Sämtliche CI-Gates (`veto-guard`, `check_vetoes`) erzwingen VETO-F02 auf Basis von Spezifikation 1.3.
- Nach der Freigabe von ADR-109 durch den Auftraggeber markiert MP-16 ADR-097 als Superseded.
