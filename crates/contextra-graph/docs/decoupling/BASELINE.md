# Baseline-Metriken contextra-graph

- Crate: `contextra-graph`
- LOC: ~13.410 (82 Dateien)
- Ring: 1, Klasse: A
- Golden-Korpus: 5 Dateien unter `tests/golden/` mit SHA-256 Validierung
- Fehlerabbildung: `GraphError` und `GraphMutationError` vollständig nach `ContextraError` abgebildet in `compat/error.rs`
- Port-Brücke: `src/compat/` kapselt `GraphIndex`, `GraphPurger`, `GraphProvider` hinter Feature `workspace`
