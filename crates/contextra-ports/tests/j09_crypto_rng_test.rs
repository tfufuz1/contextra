//! J09 Security tests: SecretBytes and CryptoRng trait bounds in contextra-ports.

use contextra_ports::{CryptoRng, EntropyRng, SecretBytes};
use std::sync::Arc;

struct MockCryptoRng;
impl CryptoRng for MockCryptoRng {
    fn fill_secret(&self, dest: &mut [u8]) -> Result<(), contextra_ports::RngError> {
        dest.fill(0x77);
        Ok(())
    }
}
impl EntropyRng for MockCryptoRng {
    fn fill_bytes(&self, dest: &mut [u8]) {
        dest.fill(0x77);
    }
    fn next_u64(&self) -> u64 {
        0x1122_3344_5566_7788
    }
}

fn require_crypto_rng<R: CryptoRng + EntropyRng>(rng: &R, buf: &mut [u8]) {
    rng.fill_bytes(buf);
}

#[test]
fn test_secret_bytes_lifecycle_and_redaction() {
    let mut secret = SecretBytes::new([0xDE; 32]);
    assert_eq!(secret.expose_secret(), &[0xDE; 32]);
    assert_eq!(secret.as_slice(), &[0xDE; 32]);

    // Debug and Display MUST redact secret content
    let dbg = format!("{:?}", secret);
    let disp = format!("{}", secret);
    assert!(dbg.contains("[REDACTED]"), "Debug output leaked secret");
    assert!(!dbg.contains("de"), "Debug output contained secret byte representation");
    assert_eq!(disp, "[REDACTED]");

    // Zeroize explicitly
    secret.zeroize();
    assert_eq!(secret.expose_secret(), &[0; 32]);
}

#[test]
fn test_crypto_rng_trait_accepts_valid_csprng() {
    let rng = MockCryptoRng;
    let mut buf = [0u8; 16];
    require_crypto_rng(&rng, &mut buf);
    assert_eq!(buf, [0x77; 16]);

    let arc_rng: Arc<dyn CryptoRng> = Arc::new(MockCryptoRng);
    let _ = arc_rng;
}
