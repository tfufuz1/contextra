//! Sondierungsprüfungen für Restore-Guard und Konsistenzprüfungen (SPEC §11.8, D-74).

use contextra_types::Result;

/// Restore-Guard-Sondierungsprüfer zur Konsistenzsicherung nach Crash-Wiederherstellung.
#[derive(Debug, Clone, Default)]
pub struct RestoreProbe {
    probed_count: usize,
}

impl RestoreProbe {
    pub fn new() -> Self {
        Self { probed_count: 0 }
    }

    /// Prüft die Konsistenz des wiederhergestellten Schlüsselraums gegen gespeicherte Prüfsummen.
    pub fn probe_key_consistency(
        &mut self,
        key: &[u8],
        expected_digest: &[u8; 32],
    ) -> Result<bool> {
        self.probed_count += 1;
        let actual_digest = blake3::hash(key);
        Ok(actual_digest.as_bytes() == expected_digest)
    }

    /// Anzahl durchgeführter Sondierungen.
    pub fn probed_count(&self) -> usize {
        self.probed_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_probe_digest() {
        let mut probe = RestoreProbe::new();
        let key = b"test_key";
        let digest = blake3::hash(key);
        assert!(probe.probe_key_consistency(key, digest.as_bytes()).unwrap());
        assert_eq!(probe.probed_count(), 1);
    }
}
