# ADR-002: HNSW für Vektor-Indexierung

*   **Datum**: 2026-05-15
*   **Status**: ✅ Final (Erweitert durch Gestufte Vektorindex-Architektur)
*   **Entscheidung**: Verwendung des Hierarchical Navigable Small World (HNSW) Graphen als Default- und Primärindex für aktive, mutable Kollektionen.
*   **Alternativen**: IVF-PQ (Quantisierung), Flat Index, DiskANN.
*   **Begründung**: HNSW bietet exzellente Suchpräzision (Recall) und sehr geringe Suchlatenz auf CPU. Für großvolumige, leselastige Kollektionen wird DiskANN als gestufter Tier nach Behebung des SQ8-Codebook-Drifts (IP-17) und nativer Delete-Semantik bereitgestellt.

---

---
