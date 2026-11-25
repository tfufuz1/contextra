# ADR-007: Produktstrategie — Lokale Agent-Memory-Library (Pure Rust) [AKTUALISIERT durch Spezifikation v9, 2026-09-27]

*   **Datum**: 2026-07-19 (Aktualisiert: 2026-09-27)
*   **Status**: ✅ Final
*   **Entscheidung**: Contextra wird als **air-gap-fähige, kryptografisch beweisbare Memory-Engine für KI-Agenten** positioniert — ein `cargo add contextra`, kein Server. Primärer Vertriebskanal: Pure Rust Bibliothek (`cargo add contextra`) und B2B2G-Systemhäuser. Python-Bindings (`contextra-py`) sind rein opt-in und aus default-members ausgelagert.
*   **Alternativen**:
    - (A) Air-Gapped / Sovereign Edge-DB — strategisch wertvoll, aber Enterprise-Vertrieb als Solo-Entwickler aktuell nicht realisierbar.
    - (B) DACH Enterprise-Search (Morphologie-Fokus) — das Morphologie-Merkmal ist zu schmal für ein eigenständiges Produkt, aber wertvoll als Differenzierungsfeature innerhalb von C.
*   **Begründung**: Option C erfordert den geringsten Pivot (80% des Codes existiert bereits), liefert in 4–8 Wochen überprüfbares Feedback (Benchmarks, PyPI-Downloads statt 12+ Monate Enterprise-Verkaufszyklen), und schließt Richtung A nicht aus — im Gegenteil: Zero-C-Deps und ACID-Garantien sind der Vorbereitungsschritt für Sovereign Edge. Die Sovereign-Core-Eigenschaften bleiben vollständig erhalten.
*   **Konsequenzen**:
    - `contextra-graph` und `contextra-py` werden in den aktiven Workspace reaktiviert (höchste Priorität).
    - `contextra-cluster`, `contextra-sandbox`, `contextra-saos-agent` wurden physisch aus dem Repo entfernt (ausgelagert).
    - README und alle Governance-Dokumente werden auf "eingebettete Agent-Memory-Library" ausgerichtet.

---

---
