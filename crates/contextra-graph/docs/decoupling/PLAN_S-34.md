# Entkopplungsplan S-34: contextra-graph

## Zielzustand
`contextra-graph` ist ein vollständig autarker Ring-1 / Klasse-A Baustein.
- Kern-Closure (`--no-default-features`): `contextra-ids`, `contextra-env`.
- Workspace-Abhängigkeiten (`contextra-types`, `contextra-ports`) liegen strikt gekapselt in `src/compat/` hinter dem Feature `workspace`.
- Fehlerzahl ohne Default-Features: 0.
- Null-Prüfung G-D: 0 Treffer im Kern.
- Alle existierenden Tests und Golden-Fixtures bleiben unverändert grün.

## Arbeitspakete
- WP0: Baseline-Erhebung und Dokumentation (erledigt in BASELINE.md)
- WP1: Determinismus- & Golden-Fixtures (G-I)
- WP2: Fehler- und Feature-System
  - `GraphError` und `GraphMutationError`
  - `compat/error.rs` mit Bitgleichheitstest
  - Cargo.toml Manifest anpassen: `contextra-ids`, `contextra-env`, `docid-128 = ["contextra-ids/docid-128"]`
  - Experiment-Features (exp-*) einführen und Abwärtskompatibilitätsaliase halten
- WP3: Schrittweise Entkopplung des Kerns
  - `cascade.rs`
  - `ppr/mod.rs` & `ppr/`
  - `edge_reinforcement_buffer.rs`
  - `percolation.rs`
  - `apprh/`, `tl_hfd/`, `path_rag/`, `community/`
  - `csr/`: `csr/graph_index/*` nach `compat/graph_index/`, lokale schmale StorageEngine-Naht
- WP4: Typen aufnehmen (`PprConfig`, `PprAlgorithm`, `TlHfdParams`)
  - Nach `types.rs` kopieren, Herkunfts-Commit im Kopf
  - Differenztest alt gegen neu in `tests/legacy_reference/types_diff.rs`
- WP5: Ids & Environment-Integration
  - `DocId`, `TxId`, `Clock`, `Rng` auf `contextra-ids` und `contextra-env` umstellen
  - Differenztest in `tests/legacy_reference/id_diff.rs`
- WP6: Standalone, Tests, Hygiene & Abschlussbericht
  - `tests/core_smoke.rs`
  - `examples/standalone.rs`
  - Doctest in `src/lib.rs`
  - Gates G-A bis G-I verifizieren
  - REPORT.md verfassen
- WP7: Pre-Commit & Finalisierung
