# ADR-040: collection.rs Modularisierung (God Object Auflösung) <!-- doc-ref-ignore -->

*   **Datum**: 2026-08-29
*   **Status**: ✅ Final
*   **Entscheidung**: `collection.rs` wird in Submodule unter `crates/contextra-db/src/collection/` aufgeteilt. <!-- doc-ref-ignore -->
*   **Alternativen**: Belassen von `collection.rs` als monolithischer ~2.900 LOC Crate-Teil. <!-- doc-ref-ignore -->
*   **Begründung**: Beseitigt AUD-08 ("God Object") und verbessert Lesbarkeit sowie Wartbarkeit. Öffentliche API und alle Typnamen bleiben exakt unverändert. Alle Re-Exports werden über `crates/contextra-db/src/collection/mod.rs` bereitgestellt (identische öffentliche Oberfläche wie bisher).

---

---
