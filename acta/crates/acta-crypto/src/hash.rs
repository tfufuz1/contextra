//! SHA-256 und die Hash-Kette für Ledger (Spec 6.2, Typ `HashChain`, mehrere Instanzen).
//!
//! Ein Eintrag bindet den Vorgänger: `entry = SHA-256(LEDGER ‖ chain_id ‖ prev ‖ canonical)`.
//! Der Vorgänger des ersten Eintrags ist der Nullwert [`GENESIS`]. Die kanonische
//! Kodierung des Eintrags (Länge, Art, Nutzlast) liegt beim Aufrufer.

use crate::domain;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Vorgänger-Wert des ersten Eintrags einer Kette.
pub const GENESIS: [u8; 32] = [0_u8; 32];

/// SHA-256 über die konkatenierten `parts`.
#[must_use]
pub fn sha256(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for part in parts {
        h.update(part);
    }
    h.finalize().into()
}

/// Berechnet den Eintragshash einer Kette.
#[must_use]
pub fn link(chain_id: &[u8; 32], prev: &[u8; 32], canonical: &[u8]) -> [u8; 32] {
    sha256(&[domain::LEDGER, chain_id, prev, canonical])
}

/// Eine Hash-Kette mit Kopfwert. Mehrere Instanzen (K-, E- und A-Ledger) sind unabhängig.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashChain {
    chain_id: [u8; 32],
    head: [u8; 32],
}

impl HashChain {
    /// Neue, leere Kette mit dem Genesis-Kopf.
    #[must_use]
    pub fn new(chain_id: [u8; 32]) -> Self {
        Self {
            chain_id,
            head: GENESIS,
        }
    }

    /// Kette mit bekanntem Kopf, etwa nach dem Laden. Der Kopf wird nicht geprüft.
    #[must_use]
    pub fn resume(chain_id: [u8; 32], head: [u8; 32]) -> Self {
        Self { chain_id, head }
    }

    /// Aktueller Kopfwert.
    #[must_use]
    pub fn head(&self) -> [u8; 32] {
        self.head
    }

    /// Hängt einen Eintrag an und liefert dessen Hash.
    pub fn append(&mut self, canonical: &[u8]) -> [u8; 32] {
        let next = link(&self.chain_id, &self.head, canonical);
        self.head = next;
        next
    }

    /// Prüft, ob `claimed` der korrekte Hash des Eintrags `canonical` nach dem Vorgänger `prev` ist.
    /// Der Vergleich läuft in konstanter Zeit.
    #[must_use]
    pub fn check(chain_id: &[u8; 32], prev: &[u8; 32], canonical: &[u8], claimed: &[u8]) -> bool {
        let expected = link(chain_id, prev, canonical);
        expected.as_slice().ct_eq(claimed).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_abc_matches_nist_vector() {
        let hex: String = sha256(&[b"abc"])
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn chain_links_and_detects_tampering() {
        let id = [7_u8; 32];
        let mut chain = HashChain::new(id);
        let e1 = chain.append(b"entry-1");
        let e2 = chain.append(b"entry-2");
        assert_eq!(chain.head(), e2);
        assert!(HashChain::check(&id, &GENESIS, b"entry-1", &e1));
        assert!(HashChain::check(&id, &e1, b"entry-2", &e2));
        // Veränderter Inhalt, falscher Vorgänger oder fremde Kette schlagen fehl.
        assert!(!HashChain::check(&id, &GENESIS, b"entry-X", &e1));
        assert!(!HashChain::check(&id, &GENESIS, b"entry-2", &e2));
        assert!(!HashChain::check(&[8_u8; 32], &GENESIS, b"entry-1", &e1));
    }

    #[test]
    fn resume_continues_the_same_chain() {
        let id = [1_u8; 32];
        let mut a = HashChain::new(id);
        let first = a.append(b"x");
        let mut b = HashChain::resume(id, first);
        assert_eq!(a.append(b"y"), b.append(b"y"));
    }
}
