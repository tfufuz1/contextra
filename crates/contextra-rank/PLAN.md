# contextra-rank — Implementierungsplan zum Soll-Zustand

**Normative Basis:** INV-5 und `docs/crates/contextra-rank.md`. **Ausgangslage:** Kernalgorithmen existieren ohne erkannte Produktions-Panics, jedoch mit E2-`types`/`ports`-Abhängigkeiten, aktivierten Default-Experimenten und weit über den Sollumfang hinausgehenden Kalibrierungs-/Driftmodulen.

1. Begrenze den produktiven Kern auf `RankItem`, Score-Normalisierung, RRF und definierte Fusion-Pipelines; entferne/archiviere experimentelle DIBUD/Resonance/Drift-/Calibration-APIs.
2. Mache `DocId` aus `contextra-ids` und `Clock` aus `contextra-env` kanonisch; entferne E2-Kompatibilitäten und setze `default = []`.
3. Implementiere RRF exakt mit festem k, Eingabevalidierung, deduplizierter Dokumentaggregation und totaler Ordnung `score desc, DocId asc`.
4. Halte die API rein und deterministisch; keine I/O, keine Zufallsabhängigkeit und keine Such-/Gate-Entscheidung im Ranker.
5. Ergänze Property-Tests für Permutationsinvarianz, Tie-Breaks, leere/ungültige Ränge und feste Golden-Fusionen.

**Abnahme:** RRF-Formeltests, Determinismus über wiederholte/permutierte Eingaben, Feature-/Dependency-Gate und Public-API-Diff.
