//! Größenklassen-Padding (Spec 4.5): Eigenschaften über viele Längen.

use acta_crypto::pad::{pad, target_len, unpad, LEN_PREFIX};
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_and_canonical_length(payload in proptest::collection::vec(any::<u8>(), 0..6000)) {
        let framed = pad(&payload).map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(unpad(&framed).map_err(|e| TestCaseError::fail(e.to_string()))?, payload.as_slice());
        let n = LEN_PREFIX + payload.len();
        prop_assert_eq!(framed.len(), target_len(n).map_err(|e| TestCaseError::fail(e.to_string()))?);
    }

    #[test]
    fn overhead_is_at_most_one_eighth_above_4096(n in 4097_usize..(1 << 22)) {
        let t = target_len(n).map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert!(t >= n);
        // Höchstens 12,5 % Mehrbedarf (Spec 4.5): 8 * (t - n) <= n.
        prop_assert!(8 * (t - n) <= n, "n={n} t={t}");
    }

    #[test]
    fn classes_are_multiples_of_their_step(n in 4_usize..(1 << 22)) {
        let t = target_len(n).map_err(|e| TestCaseError::fail(e.to_string()))?;
        let step = if n <= 4096 { 256 } else {
            1_usize << ((usize::BITS - 1 - n.leading_zeros()) - 3)
        };
        prop_assert_eq!(t % step, 0);
    }
}
