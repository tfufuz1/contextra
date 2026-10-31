//! Comprehensive decay calculation and importance score test suite for `contextra-types`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_types::importance::*;
use contextra_types::TxId;

#[test]
/// Prüft das Klemmen extremer und ungültiger Werte (NaN, Inf, Grenzwerte) bei `ImportanceScore`.
fn test_importance_score_clamping_boundary_cases() {
    let s_min = ImportanceScore::new(-1.0);
    assert_eq!(s_min.value(), 0.0);

    let s_max = ImportanceScore::new(2.5);
    assert_eq!(s_max.value(), 1.0);

    let s_nan = ImportanceScore::new(f32::NAN);
    assert_eq!(s_nan.value(), 0.0);

    let s_inf = ImportanceScore::new(f32::INFINITY);
    assert_eq!(s_inf.value(), 1.0);

    let s_neg_inf = ImportanceScore::new(f32::NEG_INFINITY);
    assert_eq!(s_neg_inf.value(), 0.0);

    let s_valid = ImportanceScore::new(0.65);
    assert_eq!(s_valid.value(), 0.65);
}

#[test]
/// Prüft, dass die Verfallsvariante `None` unabhängig von der Transaktionszeit stets den Faktor 1.0 liefert.
fn test_decay_function_none_variant() {
    let decay = DecayFunction::None;
    let t_created = TxId::new(100);
    let t_now = TxId::new(1_000_000);

    let factor = decay.decay_factor(t_created, t_now);
    assert_eq!(factor, 1.0);

    let factor_wall = decay.decay_factor_wallclock(1_000, 2_000_000, t_created, t_now);
    assert_eq!(factor_wall, 1.0);
}

#[test]
/// Prüft den exponentiellen Verfall anhand von Transaktionsschritten.
fn test_decay_function_exponential_variant() {
    let half_life_tx = 1000u64;
    let decay = DecayFunction::Exponential { half_life_tx };

    let t0 = TxId::new(100);
    let t_at_half_life = TxId::new(100 + half_life_tx);
    let t_at_two_half_lives = TxId::new(100 + 2 * half_life_tx);

    let f0 = decay.decay_factor(t0, t0);
    assert_eq!(f0, 1.0);

    let f_half = decay.decay_factor(t0, t_at_half_life);
    assert!(
        (f_half - 0.5).abs() < 1e-4,
        "At half life, factor must be ~0.5"
    );

    let f_two = decay.decay_factor(t0, t_at_two_half_lives);
    assert!(
        (f_two - 0.25).abs() < 1e-4,
        "At two half lives, factor must be ~0.25"
    );

    // Out-of-order transaction safety (now < created)
    let f_past = decay.decay_factor(t_at_half_life, t0);
    assert_eq!(f_past, 1.0);
}

#[test]
/// Prüft den exponentiellen Verfall anhand von Wanduhrzeit-Sekunden.
fn test_decay_function_wallclock_exponential() {
    let half_life_secs = 86_400u64;

    let decay_wall = DecayFunction::WallClockExponential { half_life_secs };
    let t0 = TxId::new(10);
    let t1 = TxId::new(20);

    let f_same = decay_wall.decay_factor_wallclock(1000, 1000, t0, t0);
    assert_eq!(f_same, 1.0);

    let f_half = decay_wall.decay_factor_wallclock(1000, 1000 + half_life_secs, t0, t1);
    assert!((f_half - 0.5).abs() < 1e-4);

    let f_past = decay_wall.decay_factor_wallclock(1000 + half_life_secs, 1000, t1, t0);
    assert_eq!(f_past, 1.0);
}

#[test]
/// Prüft die Stufenfunktion `StepFloor` vor und nach Überschreiten der Zugriffsschwelle.
fn test_decay_function_step_floor_variant() {
    let decay = DecayFunction::StepFloor {
        access_count_floor: 10,
    };

    let t0 = TxId::new(100);
    let t_before = TxId::new(105);
    let t_after = TxId::new(115);

    let factor_before = decay.decay_factor(t0, t_before);
    assert_eq!(factor_before, 1.0);

    let factor_after = decay.decay_factor(t0, t_after);
    assert_eq!(factor_after, 0.5);
}

#[test]
/// Prüft die Gesamtwirkung des Verfalls auf den effektiven Wichtigkeits-Score eines Dokuments.
fn test_memory_importance_effective_score_decay() {
    let base = ImportanceScore::new(1.0);
    let decay = DecayFunction::Exponential { half_life_tx: 2000 };
    let created = TxId::new(1000);

    let memory = MemoryImportance::new(base, decay, created);

    let score_at_created = memory.effective_score(created);
    assert_eq!(score_at_created, 1.0);

    let score_at_half = memory.effective_score(TxId::new(3000));
    assert!((score_at_half - 0.5).abs() < 1e-4);
}
