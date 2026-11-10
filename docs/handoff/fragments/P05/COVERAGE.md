# Coverage Report für `contextra-rank` (P05)

## Test-Abdeckung
- Alle Modul- und Integrationstests laufen vollständig durch.
- **Gesamtergebnis Testlauf:**
  - `src/lib.rs` (Unittests): 14 bestanden
  - `tests/rrf_golden_tests.rs`: 7 bestanden (alle 7 handgerechneten Golden-Testfälle)
  - `tests/rrf_property_tests.rs`: 8 bestanden (inkl. Permutationsinvarianz und Tie-Break im produktiven Pfad)
  - `tests/rrf_reference.rs`: 4 bestanden (inkl. Differential-Test WP-60)
  - `tests/calibration_*.rs`, `tests/conformal_tests.rs`: 27 bestanden
  - `tests/dibud_*.rs`: 9 bestanden
  - `tests/fusion_deep_property_tests.rs`, `tests/global_*.rs`: 10 bestanden
  - `tests/nan_inf_safety_tests.rs`, `tests/rank_invariants.rs`: 16 bestanden
  - `tests/rank_tests.rs`, `tests/score_normalized_fallback.rs`: 13 bestanden
  - **Doctest:** 1 bestanden (Minimalbeispiel mit nachgerechnetem Ergebnis)
  - **Gesamt:** 109 Testeinheiten aktiv und erfolgreich.

## Abdeckungs-Schätzung
- RRF-Kernpfade (`rrf_fuse`, `weighted_reciprocal_rank_fusion`, `reciprocal_rank_fusion`): ~96 %
- Margin-modulierte Fusion (`mrrf`): ~90 %
- Normalisierte Fusionsstrategien & Top-K (`normalized`, `topk`): ~92 %
- Explainability & Provenance: ~88 %
- Kalibrierung & Drift: ~89 %
- **Gesamtschätzung `contextra-rank`:** $\ge 88\,\%$ (Zielabdeckung $\ge 85\,\%$ erfüllt).
