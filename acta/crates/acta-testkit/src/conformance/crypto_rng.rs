//! Konformitätsprüfsuite für den `CryptoRng`-Port (Spec 3.7, 4.6, Z10).
//!
//! Die Suite folgt dem Archiv-Vorbild `rng.rs`, prüft aber den tatsächlichen Vertrag
//! `try_fill` (fallibel, kein Ersatzwert). Getestete Verträge:
//!
//! 1. **Befüllung:** `try_fill` liefert `Ok` und füllt Puffer verschiedener Länge
//!    (auch Restlängen ohne volle 8-Byte-Gruppe). Ein Puffer aus lauter Nullen gilt
//!    nach mehreren Ziehungen als Verletzung.
//! 2. **Keine Wiederholung:** zwei aufeinanderfolgende Ziehungen liefern verschiedene Puffer.
//!
//! Die Suite panict nicht. Sie liefert `ConformanceResult`.

use super::failure::{ConformanceFailure, ConformanceResult};
use acta_base::CryptoRng;

const SUITE: &str = "CryptoRng";

/// Puffergrößen für Test 1: einzelnes Byte, Restlängen, Schlüssel- und Nonce-Größen.
const FILL_SIZES: [usize; 5] = [1, 8, 24, 33, 1024];

/// Anzahl der Ziehungen je Puffergröße. Erst wenn alle Ziehungen nur Nullen liefern,
/// gilt der Test als verletzt. Ein gesunder Zufall schlägt damit praktisch nie fehl.
const DRAWS_PER_SIZE: usize = 8;

/// Führt die Konformitätsprüfung für einen [`CryptoRng`]-Port durch.
///
/// # Fehler
/// Liefert `Err(ConformanceFailure)` mit dem verletzten Testfall. `Ok(())` nur, wenn
/// alle Verträge erfüllt sind.
pub fn run_crypto_rng_suite<R: CryptoRng>(rng: &R) -> ConformanceResult {
    check_fill_sizes(rng)?;
    check_consecutive_draws_differ(rng)
}

fn check_fill_sizes<R: CryptoRng>(rng: &R) -> ConformanceResult {
    for len in FILL_SIZES {
        let mut non_zero_seen = false;
        for _ in 0..DRAWS_PER_SIZE {
            let mut buf = vec![0_u8; len];
            rng.try_fill(&mut buf).map_err(|e| {
                ConformanceFailure::new(
                    SUITE,
                    "try_fill_ok",
                    format!("try_fill({len} bytes) failed: {e}"),
                )
            })?;
            if buf.iter().any(|b| *b != 0) {
                non_zero_seen = true;
                break;
            }
        }
        if !non_zero_seen {
            return Err(ConformanceFailure::new(
                SUITE,
                "fill_non_zero",
                format!("{DRAWS_PER_SIZE} draws of {len} bytes were all zero"),
            ));
        }
    }
    Ok(())
}

fn check_consecutive_draws_differ<R: CryptoRng>(rng: &R) -> ConformanceResult {
    let mut first = [0_u8; 32];
    let mut second = [0_u8; 32];
    rng.try_fill(&mut first).map_err(|e| {
        ConformanceFailure::new(SUITE, "try_fill_ok", format!("first draw failed: {e}"))
    })?;
    rng.try_fill(&mut second).map_err(|e| {
        ConformanceFailure::new(SUITE, "try_fill_ok", format!("second draw failed: {e}"))
    })?;
    if first == second {
        return Err(ConformanceFailure::new(
            SUITE,
            "consecutive_draws_differ",
            "two consecutive 32-byte draws are identical",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use acta_base::EntropyError;

    /// Liefert immer Nullen. Positivkontrolle für `fill_non_zero` (SPEC 16.4).
    struct StuckZeroRng;
    impl CryptoRng for StuckZeroRng {
        fn try_fill(&self, dst: &mut [u8]) -> Result<(), EntropyError> {
            dst.iter_mut().for_each(|b| *b = 0);
            Ok(())
        }
    }

    /// Liefert immer dasselbe Muster. Positivkontrolle für `consecutive_draws_differ`.
    struct RepeatingRng;
    impl CryptoRng for RepeatingRng {
        fn try_fill(&self, dst: &mut [u8]) -> Result<(), EntropyError> {
            dst.iter_mut().for_each(|b| *b = 0xA5);
            Ok(())
        }
    }

    #[test]
    fn seeded_rng_passes_the_suite() {
        assert!(run_crypto_rng_suite(&crate::SeededRng::new(7)).is_ok());
    }

    #[test]
    fn failing_rng_is_reported_as_try_fill_error() {
        let res = run_crypto_rng_suite(&crate::FailingRng);
        assert!(matches!(res, Err(ref f) if f.test_case == "try_fill_ok"));
    }

    #[test]
    fn stuck_zero_source_is_detected() {
        let res = run_crypto_rng_suite(&StuckZeroRng);
        assert!(matches!(res, Err(ref f) if f.test_case == "fill_non_zero"));
    }

    #[test]
    fn repeating_source_is_detected() {
        let res = run_crypto_rng_suite(&RepeatingRng);
        assert!(matches!(res, Err(ref f) if f.test_case == "consecutive_draws_differ"));
    }
}
