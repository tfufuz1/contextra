# LOG.md — Arbeitsprotokoll für WP/P05 (contextra-rank)

## Start-Check (vor der ersten Änderung)
- [V] Branch: `wp/P05` erstellt von Basis `restructure/main` (HEAD `65e1d68a5`).
- [V] Toolchain: rustc 1.89.0, cargo 1.89.0.
- [V] Vorheriger Build-Status:
  - `cargo check -p contextra-rank`: Erfolgreich (grün).
  - `cargo test -p contextra-rank`: Alle Tests erfolgreich (grün, 80 Tests bestanden).
  - Fremd-Warnungen: `contextra-ports` (deprecated ErasureId/ScanMethod).
- [V] Besitzgrenzen: Ausschließlich `crates/contextra-rank/` und `docs/handoff/fragments/P05/`.

## Commits & Arbeitspakete

### WP-60: Fusionsrollen dokumentieren und Differential-Test implementieren
- Dateien:
  - `crates/contextra-rank/src/fusion/rrf.rs`
  - `crates/contextra-rank/README.md`
  - `crates/contextra-rank/tests/rrf_reference.rs`
  - `docs/handoff/fragments/P05/LOG.md`
  - `docs/handoff/fragments/P05/OPEN_ISSUES.md`
  - `docs/handoff/fragments/P05/PENDING_API_CHANGES.md`
- Prüfbefehl: `cargo test -p contextra-rank --test rrf_reference` → grün (4/4 tests passed)
- Ergebnis: Rollen von `weighted_reciprocal_rank_fusion` (produktiv), `reciprocal_rank_fusion` (Wrapper k=60), und `rrf_fuse` (Referenz-Orakel auf ScoredItem) präzise dokumentiert. Differential-Test `test_differential_rrf_fuse_vs_weighted_rrf` belegt identische relative Rangfolge und numerische Score-Parität.

### WP-61: Golden-Tests mit handgerechneten Werten
- Dateien:
  - `crates/contextra-rank/tests/rrf_golden_tests.rs`
  - `crates/contextra-rank/src/fusion/rrf.rs`
  - `docs/handoff/fragments/P05/LOG.md`
- Prüfbefehl: `cargo test -p contextra-rank --test rrf_golden_tests` → grün (7/7 tests passed)
- Ergebnis: Exakte Handrechnungen als Code-Kommentare für alle 7 geforderten Fälle (2 Listen, 3 Listen, Gleichstand/Tie-Break nach ID aufsteigend, ID fehlt in Liste, Gewichte 0 und 1, leere Eingabe, doppelte IDs innerhalb einer Liste). Intra-Listen-Deduplizierung auf bestem Rang in `rrf.rs` sichergestellt.

### WP-62: Modul-Reduktion, Extraktion von mRRF & Feature-Gating
- Dateien:
  - `crates/contextra-rank/src/fusion/mrrf.rs`
  - `crates/contextra-rank/src/fusion/rrf.rs`
  - `crates/contextra-rank/src/fusion/mod.rs`
  - `crates/contextra-rank/Cargo.toml`
  - `crates/contextra-rank/README.md`
  - `docs/handoff/fragments/P05/LOG.md`
  - `docs/handoff/fragments/P05/PENDING_API_CHANGES.md`
- Prüfbefehl: `cargo test -p contextra-rank` → grün (88/88 tests passed)
- Ergebnis: `mrrf` aus `rrf.rs` sauber in eigenes Modul `fusion/mrrf.rs` ausgelagert. Feature `experimental-rank` eingeführt; Standard-Features behalten Abwärtskompatibilität für Konsumenten bei. Konsumentenanalyse und Migrationsvorschlag in PENDING_API_CHANGES.md festgehalten.

### WP-63: Proptests für Permutationsinvarianz und deterministischen Tie-Break im produktiven Pfad
- Dateien:
  - `crates/contextra-rank/tests/rrf_property_tests.rs`
  - `docs/handoff/fragments/P05/LOG.md`
- Prüfbefehl: `cargo test -p contextra-rank --test rrf_property_tests` → grün (8/8 tests passed)
- Ergebnis: Proptests für `weighted_reciprocal_rank_fusion` implementiert: (1) `prop_weighted_rrf_permutation_invariance_arbitrary_docs` validiert Permutationsinvarianz bei gleichen Gewichten über alle Permutationen hinweg, (2) `prop_weighted_rrf_tie_break_ascending_id` belegt deterministischen Tie-Break (lexikografisch nach ID aufsteigend) bei identischen akkumulierten RRF-Scores.

### P1–P7: Crate-Qualität, Dokumentation und Multi-Feature-Verifikation
- Dateien:
  - `crates/contextra-rank/src/lib.rs`
  - `crates/contextra-rank/README.md`
  - `crates/contextra-rank/API.md`
  - `crates/contextra-rank/src/dibud/state.rs`
  - `crates/contextra-rank/tests/dibud_equivalence.rs`
  - `crates/contextra-rank/tests/rank_invariants.rs`
  - `crates/contextra-rank/tests/rrf_property_tests.rs`
  - `crates/contextra-rank/tests/rrf_golden_tests.rs`
  - `crates/contextra-rank/tests/rrf_reference.rs`
  - `docs/handoff/fragments/P05/COVERAGE.md`
  - `docs/handoff/fragments/P05/LOG.md`
- Prüfbefehle:
  - `cargo test -p contextra-rank --no-default-features` → grün (88 passed)
  - `cargo test -p contextra-rank --all-targets --all-features` → grün (108 passed)
  - `cargo test -p contextra-rank --doc` → grün (1 passed)
  - `cargo clippy -p contextra-rank --no-default-features --all-targets -- -D warnings` → grün
  - `cargo clippy -p contextra-rank --no-default-features --features dibud --all-targets -- -D warnings` → grün
  - `cargo doc -p contextra-rank --no-deps` → grün
- Ergebnis: Alle Qualitätskriterien P1–P7 erfüllt.
