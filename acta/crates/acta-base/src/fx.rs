//! Festkomma Q32.32 (Spec 3.3, 9.2). Kein `f32`/`f64` in Ranking-Pfaden; Rundung half-even.
//!
//! `fx_ln`: Zerlegung `x = m * 2^e` mit `m` in `[1, 2)`, `ln x = e * LN2 + 2 * atanh((m - 1) / (m + 1))`
//! als Ganzzahlreihe mit 12 Termen (Rechnung in Q62, Rundung half-even nach Q32.32).
//! Goldvektoren `T-TT-FX`: `tests/fx_golden.rs`.

/// Q32.32-Festkommazahl.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Fx(pub i64);

// Die Methoden sind bewusst keine `std::ops`-Implementierungen: Addition und Subtraktion
// saettigen, `div` liefert `None` bei Division durch null. Eine Operator-Implementierung
// wuerde diese Semantik verdecken.
#[allow(
    clippy::should_implement_trait,
    reason = "saturierende Festkomma-Semantik, bewusst kein std::ops"
)]
impl Fx {
    /// Anzahl Nachkommabits.
    pub const FRAC: u32 = 32;
    /// Null.
    pub const ZERO: Self = Self(0);
    /// Eins.
    pub const ONE: Self = Self(1_i64 << 32);
    /// `ln 2` in Q32.32 (gerundet).
    pub const LN2: Self = Self(2_977_044_472);

    /// Aus den rohen Q32.32-Bits.
    #[must_use]
    pub const fn from_bits(bits: i64) -> Self {
        Self(bits)
    }

    /// Rohe Q32.32-Bits.
    #[must_use]
    pub const fn to_bits(self) -> i64 {
        self.0
    }

    /// Ganzzahlanteil, abgerundet (Richtung minus unendlich).
    #[must_use]
    pub const fn floor_int(self) -> i64 {
        self.0 >> Self::FRAC
    }

    /// Bruch `num / den`, Rundung half-even; `None` bei `den == 0`.
    #[must_use]
    pub fn from_ratio(num: i64, den: i64) -> Option<Self> {
        Self::from_int_wide(num).div(Self::from_int_wide(den))
    }

    // Ganze Zahl bis i32 exakt; darueber saturierend (Hilfe fuer `from_ratio`).
    fn from_int_wide(v: i64) -> Self {
        Self(sat(i128::from(v) << Self::FRAC))
    }

    /// Betrag (saturierend).
    #[must_use]
    pub fn abs(self) -> Self {
        Self(self.0.saturating_abs())
    }

    /// Negation (saturierend).
    #[must_use]
    pub fn neg(self) -> Self {
        Self(self.0.saturating_neg())
    }

    /// Aus ganzer Zahl.
    #[must_use]
    pub fn from_int(v: i32) -> Self {
        Self(i64::from(v) << Self::FRAC)
    }

    /// Saturierende Addition.
    #[must_use]
    pub fn add(self, o: Self) -> Self {
        Self(self.0.saturating_add(o.0))
    }

    /// Saturierende Subtraktion.
    #[must_use]
    pub fn sub(self, o: Self) -> Self {
        Self(self.0.saturating_sub(o.0))
    }

    /// Multiplikation, Rundung half-even.
    #[must_use]
    pub fn mul(self, o: Self) -> Self {
        let p = i128::from(self.0) * i128::from(o.0);
        Self(sat(round_shift_half_even(p, Self::FRAC)))
    }

    /// Division, Rundung half-even; `None` bei Division durch null.
    #[must_use]
    pub fn div(self, o: Self) -> Option<Self> {
        if o.0 == 0 {
            return None;
        }
        let num = i128::from(self.0) << Self::FRAC;
        let den = i128::from(o.0);
        let q = num / den;
        let r = num % den;
        let twice = r.abs() * 2;
        let q = if twice > den.abs() || (twice == den.abs() && q & 1 == 1) {
            q + if (num < 0) == (den < 0) { 1 } else { -1 }
        } else {
            q
        };
        Some(Self(sat(q)))
    }
}

fn sat(v: i128) -> i64 {
    i64::try_from(v).unwrap_or(if v < 0 { i64::MIN } else { i64::MAX })
}

fn round_shift_half_even(v: i128, shift: u32) -> i128 {
    let q = v >> shift;
    let rem = v - (q << shift);
    let half = 1_i128 << (shift - 1);
    if rem > half || (rem == half && q & 1 == 1) {
        q + 1
    } else {
        q
    }
}

// ln 2 in Q62, gerundet (3196577161300663915 / 2^62).
const LN2_Q62: i128 = 3_196_577_161_300_663_915;

/// Natuerlicher Logarithmus in Q32.32; `None` fuer `x <= 0`. Rein ganzzahlig, deterministisch.
#[must_use]
pub fn fx_ln(x: Fx) -> Option<Fx> {
    let raw = u64::try_from(x.0).ok().filter(|r| *r > 0)?;
    // raw < 2^63, daher 1 <= leading_zeros und p <= 62.
    let p = 63 - raw.leading_zeros();
    let e = i128::from(p) - i128::from(Fx::FRAC);
    // m in [1, 2) als Q62.
    let m = i128::from(raw) << (62 - p);
    let one = 1_i128 << 62;
    // z = (m - 1) / (m + 1) in Q62, z <= 1/3.
    let z = ((m - one) << 62) / (m + one);
    let z2 = (z * z) >> 62;
    let mut term = z;
    let mut sum = 0_i128;
    for k in 0_i128..12 {
        sum += term / (2 * k + 1);
        term = (term * z2) >> 62;
    }
    let total = e * LN2_Q62 + 2 * sum;
    Some(Fx(sat(round_shift_half_even(total, 30))))
}

/// Ganzzahlige Quadratwurzel (abgerundet).
#[must_use]
pub fn isqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isqrt_exact() {
        for n in [0_u64, 1, 2, 3, 4, 15, 16, 17, 1 << 40, u64::MAX] {
            let r = isqrt(n);
            assert!(u128::from(r) * u128::from(r) <= u128::from(n));
            assert!(u128::from(r + 1) * u128::from(r + 1) > u128::from(n));
        }
    }

    #[test]
    fn mul_div_roundtrip_simple() {
        let two = Fx::from_int(2);
        let three = Fx::from_int(3);
        assert_eq!(two.mul(three), Fx::from_int(6));
        assert_eq!(Fx::from_int(6).div(three), Some(two));
        assert_eq!(Fx::ONE.div(Fx::ZERO), None);
    }

    #[test]
    fn ln_basic_points() {
        assert_eq!(fx_ln(Fx::ONE), Some(Fx::ZERO));
        assert_eq!(fx_ln(Fx::from_int(2)), Some(Fx::LN2));
        assert_eq!(fx_ln(Fx::ZERO), None);
        assert_eq!(fx_ln(Fx(-1)), None);
        // ln(1/2) = -ln 2
        assert_eq!(fx_ln(Fx(1 << 31)), Some(Fx::LN2.neg()));
    }

    #[test]
    fn ratio_and_helpers() {
        assert_eq!(Fx::from_ratio(1, 2), Some(Fx(1 << 31)));
        assert_eq!(Fx::from_ratio(1, 0), None);
        assert_eq!(Fx::from_bits(5).to_bits(), 5);
        assert_eq!(Fx::from_int(-3).floor_int(), -3);
        assert_eq!(Fx(-1).floor_int(), -1);
        assert_eq!(Fx::from_int(-3).abs(), Fx::from_int(3));
        assert_eq!(Fx(i64::MIN).abs(), Fx(i64::MAX));
    }

    #[test]
    fn half_even_rounding() {
        // 0.5 * 0.5 = 0.25 exakt
        let half = Fx(1 << 31);
        assert_eq!(half.mul(half), Fx(1 << 30));
    }
}
