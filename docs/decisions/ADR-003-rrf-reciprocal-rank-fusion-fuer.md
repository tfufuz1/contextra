# ADR-003: RRF (Reciprocal Rank Fusion) für Hybridisierung

*   **Datum**: 2026-05-20
*   **Status**: ✅ Final (Bestätigt & Präzisiert)
*   **Entscheidung**: RRF bleibt der verbindliche Default für die Multi-Signal-Fusion, da es score-blind und hochgradig robust gegenüber Signalausfällen ist.
*   **Alternativen**: Score-normalisierte Fusion (CombSUM / Z-Score).
*   **Begründung**: RRF verhindert Fehler durch nicht-vergleichbare Score-Skalen und Ausfall einzelner Signale (z. B. Graph ohne Kandidaten). Score-Normalisierung wird ausschließlich als opt-in Erweiterung mit dynamischem Fallback auf RRF bereitgestellt.

---

---
