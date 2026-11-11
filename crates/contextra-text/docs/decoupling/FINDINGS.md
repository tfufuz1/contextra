# Findings: AUFTRAG P5 - Decoupling contextra-text

- Flaky test `inverted::tests::search_tests::test_high_frequency_term_resident_index_performance`:
  timing threshold sensitive in parallel test runs, passes reliably in isolation.
