//! Festkomma Q32.32 (Spec 3.3, 9.2). Kein `f32`/`f64` in Ranking-Pfaden; Rundung half-even.
//!
//! Offen: `fx_ln` (Zerlegung `x = m * 2^e`, `2 * atanh`-Reihe mit 12 Termen) erst nach den Goldvektoren `T-TT-FX`.

/// Q32.32-Festkommazahl.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Fx(pub i64);

impl Fx {
    /// Anzahl Nachkommabits.
    pub const FRAC: u32 = 32;
    /// Null.
    pub const ZERO: Self = Self(0);
    /// Eins.
    pub const ONE: Self = Self(1_i64 << 32);

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
    fn half_even_rounding() {
        // 0.5 * 0.5 = 0.25 exakt
        let half = Fx(1 << 31);
        assert_eq!(half.mul(half), Fx(1 << 30));
    }
}
