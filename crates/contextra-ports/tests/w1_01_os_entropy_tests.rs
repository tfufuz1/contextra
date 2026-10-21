use contextra_ports::{CryptoRng, EntropyRng, OsEntropyRng, RngError, SecretBytes};

#[test]
fn os_entropy_fills_and_differs_between_calls() {
    let rng = OsEntropyRng::new();

    let secret1 = SecretBytes::<32>::generate(&rng).expect("OsEntropyRng should generate secret1");
    let secret2 = SecretBytes::<32>::generate(&rng).expect("OsEntropyRng should generate secret2");

    assert_ne!(
        secret1.expose(),
        secret2.expose(),
        "OsEntropyRng generated identical secret bytes across successive calls"
    );

    let mut buf = [0u8; 16];
    rng.fill_bytes(&mut buf);
    assert_ne!(buf, [0u8; 16], "fill_bytes should fill non-zero entropy");
}

#[test]
fn os_entropy_error_propagates() {
    struct FailingEntropyRng;
    impl CryptoRng for FailingEntropyRng {
        fn fill_secret(&self, _dest: &mut [u8]) -> Result<(), RngError> {
            Err(RngError::Unavailable(
                "simulated OS entropy failure".to_string(),
            ))
        }
    }
    impl EntropyRng for FailingEntropyRng {
        fn fill_bytes(&self, dest: &mut [u8]) {
            dest.fill(0);
        }
        fn next_u64(&self) -> u64 {
            0
        }
        fn fill_secret(&self, dest: &mut [u8]) -> Result<(), RngError> {
            CryptoRng::fill_secret(self, dest)
        }
    }

    let rng = FailingEntropyRng;
    let res = SecretBytes::<32>::generate(&rng);
    assert!(
        res.is_err(),
        "SecretBytes::generate must return Err when fill_secret fails"
    );
    match res {
        Err(RngError::Unavailable(msg)) => {
            assert!(msg.contains("simulated OS entropy failure"));
        }
        _ => panic!("Expected RngError::Unavailable"),
    }
}
