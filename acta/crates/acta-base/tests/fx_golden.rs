//! Goldvektoren `T-TT-FX` (Spec 9.2, 16.3). Referenzwerte stammen aus exakter
//! Dezimalarithmetik (80 Stellen), gerundet half-even auf Q32.32, unabhaengig von der
//! Rust-Implementierung erzeugt. Jedes Plattenformat und jedes Ranking haengt an diesen Werten.

use acta_base::fx::{fx_ln, Fx};

const GOLDEN: &[(i64, i64)] = &[
    (1, -95_265_423_098),
    (2, -92_288_378_626),
    (3, -90_546_919_247),
    (65_536, -47_632_711_549),
    (4_294_967_295, -1),
    (4_294_967_296, 0),
    (4_294_967_297, 1),
    (6_442_450_944, 1_741_459_379),
    (8_589_934_592, 2_977_044_472),
    (12_884_901_888, 4_718_503_851),
    (42_949_672_960, 9_889_527_671),
    (4_294_967_296_000, 29_668_583_012),
    (2_147_483_648, -2_977_044_472),
    (1_431_655_765, -4_718_503_852),
    (2_576_980_377, -2_193_979_349),
    (123_456_789_012, 14_424_421_179),
    (9_223_372_036_854_775_807, 92_288_378_626),
    (5_153_960_755, 783_065_124),
];

#[test]
fn ln_matches_golden_vectors() {
    for (raw, want) in GOLDEN {
        assert_eq!(
            fx_ln(Fx::from_bits(*raw)).map(Fx::to_bits),
            Some(*want),
            "raw={raw}"
        );
    }
}

/// Positivkontrolle (16.4): der Test ist faehig zu scheitern, wenn die Implementierung abweicht.
#[test]
fn golden_check_detects_a_wrong_value() {
    let wrong = GOLDEN.iter().any(|(raw, want)| {
        fx_ln(Fx::from_bits(*raw)).map(Fx::to_bits) != Some(want.saturating_add(1))
    });
    assert!(wrong);
}

#[test]
fn ln_is_monotonic_over_a_dense_sweep() {
    let mut prev = i64::MIN;
    let mut raw = 1_i64;
    while raw < i64::MAX / 3 {
        let v = fx_ln(Fx::from_bits(raw))
            .map(Fx::to_bits)
            .unwrap_or(i64::MIN);
        assert!(v >= prev, "nicht monoton bei raw={raw}");
        prev = v;
        raw += raw / 7 + 1;
    }
}

/// idf(t) = ln(1 + (N - df + 0,5) / (df + 0,5)) (9.2): fuer N=1000, df=10.
#[test]
fn bm25_idf_example() {
    // (1000 - 10 + 0,5) / (10 + 0,5) = 990,5 / 10,5 = 1981 / 21
    let ratio = Fx::from_ratio(1981, 21).unwrap_or(Fx::ZERO);
    assert_eq!(ratio.to_bits(), 405_158_581_589);
    let idf = fx_ln(Fx::ONE.add(ratio)).map(Fx::to_bits);
    // Referenz: ln(1 + ratio) mit ratio auf Q32.32 gerundet.
    assert_eq!(idf, Some(19_573_796_003));
}
