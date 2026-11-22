//! # Real Entropy
//!
//! Produktive Implementierung des `Entropy`-Ports über `/dev/urandom` auf Unix.
//! Fail-closed auf nicht-unterstützten Plattformen (O-13).

use contextra_storage_ports::Entropy;
use std::io;

/// Standard-Entropiequelle über `/dev/urandom`.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealEntropy;

impl Entropy for RealEntropy {
    #[cfg(unix)]
    fn fill(&self, dst: &mut [u8]) -> io::Result<()> {
        crate::env::real_fs::RealFs::fill_entropy(dst)
    }

    #[cfg(not(unix))]
    fn fill(&self, _dst: &mut [u8]) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Entropy nur auf Unix-Plattformen über /dev/urandom unterstützt (O-13)",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_fill() {
        let entropy = RealEntropy;
        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];
        entropy.fill(&mut buf1).expect("entropy fill succeeds");
        entropy.fill(&mut buf2).expect("entropy fill succeeds");
        assert_ne!(buf1, buf2);
        assert_ne!(buf1, [0u8; 32]);
    }
}
