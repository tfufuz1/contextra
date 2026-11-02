# Contextra — Chronologischer Tag- & Review-Bericht (Changelog)

<!-- doc-stamp: 2026-11-02 | Normativ: docs/SPEC.md §3, Anhang H -->

## [4.0.0] - 2026-11-02

### Geändert
- **SPEC 4.0.0 als alleinige normative Quelle:** Alle Byte-Formate (Anhang F) und Umsetzungspläne (§15.4) wurden in `docs/SPEC.md` konsolidiert. Alle Begleitdokumente wurden angepasst oder verweisen auf SPEC 4.0.0.
- **Drei Säulen der Löschung in v1 (D-01 / ADR-002):** Aufhebung von VETO-D01 durch VETO-D01b. Löschung umfasst verbindlich (1) Gate & physischen Purge, (2) kryptografische Schlüsselvernichtung im hierarchischen Tresor (`T`, `C`, `S`, `D`-Slots gem. D-79 sowie `F`-Dateischlüssel gem. D-85), (3) unabhängige Messung (MK1–MK3) und signierter Bericht.
- **Löschcluster-Schlüsselraum (D-31, D-70):** Dokumente werden über `DocRef` adressiert; Betroffene bilden den Schlüsselraum-Cluster (`cluster_tag = subject_tag`), wodurch Purge-Aufwand strikt mit dem Ziel skaliert (P11).
- **HNSW-Löschverfahren (D-77):** Dreistufiger Ablauf mit Navigationssurrogaten vor `Gated`, Batch-Kantenreparatur im Purge-Fenster und deterministischem Rebuild.
- **WAL-Unveränderlichkeit (D-73):** WAL-Dateien werden nicht umgeschrieben, sondern nach Memtable-Flush und Segmentsiegelung bis zum Flush-Punkt ganzheitlich gelöscht.
- **Konsolidierte Messbegriffe (D-72):** Trennung von Evidenzstufen (**EV0–EV4**) und Messklassen (**MK1–MK3**).
- **Bereinigung der Crate-Architektur (§5.1):** 14 Produktions-Crates in den Ringen 0–4; Altlasten und produktfremde Crates entfernt.
