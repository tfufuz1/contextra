use contextra_ports::{EntropyRng, RngError, SystemRng};

#[test]
fn system_rng_fill_secret_is_err() {
    let rng = SystemRng::new();
    let mut secret_buf = [0u8; 32];
    let res = rng.fill_secret(&mut secret_buf);
    assert!(
        matches!(res, Err(RngError::SecretUnsupported)),
        "SystemRng must return Err(RngError::SecretUnsupported) when fill_secret is called"
    );
}
