# ADR-013: Gestuftes Vektorindex-Modell — HNSW Default + DiskANN Tier (contextra-index)

*   **Datum**: 2026-08-23 (Revidiert 2026-09-16 via ADR-083 / ADR §16.2)
*   **Status**: ✅ Final (Formalisiert & Erweitert durch ADR-083)
*   **Entscheidung**: HNSW bleibt Default-Index für mutable Kollektionen. DiskANN wird als offizieller Tier für große, leselastige Kollektionen verabschiedet. Die detaillierte Formalisierung, Gate-Bedingungen (Prompt 1.6 Tombstone-Fix & Prompt 1.7 SQ8-Drift-Fix) und der 4-Stufen-Migrationspfad werden normativ in **ADR-083 (ADR §16.2)** geregelt.
*   **Alternativen**:
    - **Option A**: Volle Integration durch Refactoring der `VectorIndex`-Abstraktion und Anpassung der `contextra-db::Collection`, um dynamisch zwischen HNSW und DiskANN zu wechseln.
*   **Begründung**: `contextra-db::Collection` und `HnswIndex` sind aktuell extrem eng verzahnt (z.B. direkte Nutzung von `all_doc_ids_from_map()` in der Collection). Eine überhastete Integration würde die Architektur-Integrität und Snapshot-Isolation gefährden, da DiskANN derzeit `insert()` und `delete()` nicht vollständig (oder nur mit `Err`) implementiert. Option A hätte gravierende Umbauten am Kern-Datenfluss der Collection zur Folge gehabt. Das Verbergen von DiskANN schützt die Produktionspfade, lässt aber den Code für zukünftige Entwicklungen im Baum.
*   **Konsequenzen**:
    - `contextra-db` nutzt HNSW weiterhin hartcodiert.
    - Endnutzer sehen die DiskANN-Funktionalität nicht in der öffentlichen API.

---

---
