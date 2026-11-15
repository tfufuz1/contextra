//! Prüft den Export von außen: Suite und Makro müssen über die Crate-Wurzel erreichbar sein.

use acta_testkit::conformance::run_crypto_rng_suite;
use acta_testkit::{assert_conformance, FailingRng, SeededRng};

#[test]
fn seeded_rng_passes_exported_suite_via_macro() {
    assert_conformance!(run_crypto_rng_suite(&SeededRng::new(42)));
}

#[test]
#[should_panic(expected = "try_fill_ok")]
fn failing_rng_fails_exported_suite_via_macro() {
    assert_conformance!(run_crypto_rng_suite(&FailingRng));
}
