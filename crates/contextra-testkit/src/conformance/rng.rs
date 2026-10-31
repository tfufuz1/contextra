//! # Konformitätsprüfsuite für den `Rng`-Port (`conformance/rng`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen für Pseudozufallszahlengeneratoren
//! (PRNG) des [`contextra_ports::Rng`]-Ports.
//!
//! ## Getestete Verträge
//! 1. **Nicht-Konstanz:** [`contextra_ports::Rng::next_u64`] darf bei aufeinanderfolgenden
//!    Ziehungen keine identischen Werte liefern.
//! 2. **Byte-Befüllung:** [`contextra_ports::Rng::fill_bytes`] muss den Zielpuffer füllen
//!    (darf nicht unberührt oder rein nullwertig bleiben).
//! 3. **Intervall-Einhaltung:** [`contextra_ports::Rng::next_unit_f64`] muss strikt
//!    im halboffenen Intervall `[0.0, 1.0)` liegen.

use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::Rng;

// ANCHOR[contextra-testkit:CONF-RNG-SUITE-001]
/// Führt die vollständige Konformitätsprüfung für einen [`Rng`]-Port durch.
///
/// **Ablauf:**
/// 1. Zieht 3x `next_u64()` und verifiziert Varianz.
/// 2. Füllt einen 16-Byte-Puffer und prüft auf Nicht-Null-Inhalt.
/// 3. Zieht 100x `next_unit_f64()` und prüft die Grenzen `0.0 <= val < 1.0`.
///
/// **Rückgabe:** `Ok(())` bei Erfolg, andernfalls `Err(ConformanceFailure)`.
pub fn run_rng_suite<R: Rng>(rng: &R) -> ConformanceResult {
    let suite = "Rng";

    // Test 1: Non-constant u64 sequence
    let u1 = rng.next_u64();
    let u2 = rng.next_u64();
    let u3 = rng.next_u64();
    if u1 == u2 && u2 == u3 {
        return Err(ConformanceFailure::new(
            suite,
            "next_u64_non_constant",
            format!("3 consecutive draws returned identical u64: {u1}"),
        ));
    }

    // Test 2: fill_bytes
    let mut buf = [0u8; 16];
    rng.fill_bytes(&mut buf);
    if buf == [0u8; 16] {
        return Err(ConformanceFailure::new(
            suite,
            "fill_bytes_non_zero",
            "fill_bytes produced 16 all-zero bytes",
        ));
    }

    // Test 3: next_unit_f64 in [0.0, 1.0)
    for _ in 0..100 {
        let val = rng.next_unit_f64();
        if !(0.0..1.0).contains(&val) {
            return Err(ConformanceFailure::new(
                suite,
                "next_unit_f64_bounds",
                format!("next_unit_f64 returned {val}, outside [0.0, 1.0)"),
            ));
        }
    }

    Ok(())
}
