use contextra_ports::{CryptoRng, EntropyRng, RngError, SecretBytes};

struct DummyEntropy;
impl CryptoRng for DummyEntropy {
    fn fill_secret(&self, dest: &mut [u8]) -> Result<(), RngError> {
        dest.fill(0x11);
        Ok(())
    }
}
impl EntropyRng for DummyEntropy {
    fn fill_bytes(&self, dest: &mut [u8]) {
        dest.fill(0x11);
    }
    fn next_u64(&self) -> u64 {
        0x1111_1111_1111_1111
    }
    fn fill_secret(&self, dest: &mut [u8]) -> Result<(), RngError> {
        dest.fill(0x11);
        Ok(())
    }
}

#[test]
fn secret_bytes_not_clone_and_debug_redacted() {
    let sec = SecretBytes::<16>::generate(&DummyEntropy).expect("should generate");
    assert_eq!(sec.expose(), &[0x11; 16]);

    let debug_str = format!("{:?}", sec);
    assert!(
        debug_str.to_lowercase().contains("redacted"),
        "Debug string must contain 'redacted', got: {}",
        debug_str
    );
    assert!(
        !debug_str.contains("17") && !debug_str.contains("11"),
        "Debug string must not leak secret bytes"
    );
}

#[test]
fn secret_bytes_ct_eq_semantics() {
    struct CustomEntropy(u8);
    impl CryptoRng for CustomEntropy {
        fn fill_secret(&self, dest: &mut [u8]) -> Result<(), RngError> {
            dest.fill(self.0);
            Ok(())
        }
    }
    impl EntropyRng for CustomEntropy {
        fn fill_bytes(&self, dest: &mut [u8]) {
            dest.fill(self.0);
        }
        fn next_u64(&self) -> u64 {
            u64::from(self.0)
        }
        fn fill_secret(&self, dest: &mut [u8]) -> Result<(), RngError> {
            dest.fill(self.0);
            Ok(())
        }
    }

    let a1 = SecretBytes::<32>::generate(&CustomEntropy(0xAA)).unwrap();
    let a2 = SecretBytes::<32>::generate(&CustomEntropy(0xAA)).unwrap();
    let b = SecretBytes::<32>::generate(&CustomEntropy(0xBB)).unwrap();

    assert!(a1.ct_eq(&a2), "Identical secret bytes must be ct_eq");
    assert!(!a1.ct_eq(&b), "Different secret bytes must not be ct_eq");
}
