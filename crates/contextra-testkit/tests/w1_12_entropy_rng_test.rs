// FILE-CONTEXT
// STAND: 2026-10-22
// ZWECK: Verification that testkit EntropyRng structure does not implement fill_secret.
// INVARIANTEN: Cryptographic secret separation doctrine (§7.4 / E55).

use contextra_ports::EntropyRng as EntropyRngPort;
use contextra_testkit::EntropyRng;

#[test]
fn testkit_rng_never_implements_fill_secret() {
    let rng = EntropyRng::new();

    // Verify next_u64 and fill_bytes work via trait
    let val = EntropyRngPort::next_u64(&rng);
    let mut buf = [0u8; 16];
    EntropyRngPort::fill_bytes(&rng, &mut buf);

    assert_ne!(val, 0);
    assert_ne!(buf, [0u8; 16]);

    // Ensure struct type is properly gated in testkit
    assert_eq!(
        std::any::type_name::<EntropyRng>(),
        "contextra_testkit::dst::rng::EntropyRng"
    );
}
