//! # Clusterschlüssel-, Cache-, Rotations- und Nonce-Tests
//!
//! Verifiziert SPEC §10.1, §10.10, §16.2 bis §16.4 sowie INV-KEY-1.

use contextra_ports::persistence::Entropy;
use contextra_storage::keys::cluster_keys::{ClusterId, ClusterKeyError, KeyState};
use contextra_storage::keys::hierarchy::{derive_cluster_seal_key, derive_keys_key};
use contextra_storage::keys::key_cache::KeyCache;
use contextra_storage::keys::nonce::{seal_nonce_from_entropy, SEAL_NONCE_LEN};
use contextra_storage::keys::protocol::KeysFile;
use contextra_storage::keys::rotation::{rotation_due, SEAL_ROTATION_LIMIT};
use contextra_storage::keys::secret::Secret32;
use contextra_storage::testing::refimpl::Blake3Kdf;
use std::io;
use std::sync::atomic::{AtomicU8, Ordering};

/// Deterministische Entropie für Tests: füllt mit einem wachsenden Byte.
struct CountingEntropy {
    next: AtomicU8,
}

impl CountingEntropy {
    fn new() -> Self {
        Self {
            next: AtomicU8::new(1),
        }
    }
}

impl Entropy for CountingEntropy {
    fn fill(&self, dst: &mut [u8]) -> io::Result<()> {
        let v = self.next.fetch_add(1, Ordering::SeqCst);
        for b in dst.iter_mut() {
            *b = v;
        }
        Ok(())
    }
}

/// Entropie, die immer fehlschlägt (Fehler muss durchgereicht werden).
struct FailingEntropy;

impl Entropy for FailingEntropy {
    fn fill(&self, _dst: &mut [u8]) -> io::Result<()> {
        Err(io::Error::other("ausgefallen"))
    }
}

fn cid(b: u8) -> ClusterId {
    ClusterId([b; 16])
}

fn key(b: u8) -> Secret32 {
    Secret32::new([b; 32])
}

// ---- ClusterKeyError (SA-12) -------------------------------------------------

#[test]
fn cluster_key_error_variants_are_distinct() {
    let all = [
        ClusterKeyError::NotFound,
        ClusterKeyError::Unavailable,
        ClusterKeyError::DestroyFailed,
        ClusterKeyError::Unsupported,
    ];
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            assert_eq!(i == j, a == b, "Varianten {i} und {j}");
        }
    }
}

#[test]
fn cluster_key_error_display_is_nonempty_and_without_key_material() {
    let all = [
        ClusterKeyError::NotFound,
        ClusterKeyError::Unavailable,
        ClusterKeyError::DestroyFailed,
        ClusterKeyError::Unsupported,
    ];
    for e in all {
        let text = e.to_string();
        assert!(!text.is_empty());
        // Keine Bytes oder Hex-Blöcke in Fehlertexten.
        assert!(!text.contains("0x"));
    }
}

#[test]
fn key_state_is_comparable() {
    assert_ne!(KeyState::Present, KeyState::Absent);
}

#[test]
fn cluster_id_orders_bytewise() {
    assert!(cid(1) < cid(2));
    assert_eq!(cid(7), cid(7));
}

// ---- Nonce: 24 Byte, frisch je Versiegelung (SD-14, §10.1) -------------------

#[test]
fn seal_nonce_has_24_bytes() {
    assert_eq!(SEAL_NONCE_LEN, 24);
    let e = CountingEntropy::new();
    let n = seal_nonce_from_entropy(&e).expect("nonce");
    assert_eq!(n.len(), 24);
}

#[test]
fn seal_nonce_is_fresh_per_call() {
    let e = CountingEntropy::new();
    let a = seal_nonce_from_entropy(&e).expect("a");
    let b = seal_nonce_from_entropy(&e).expect("b");
    assert_ne!(a, b, "jede Versiegelung braucht eine neue Nonce");
}

#[test]
fn seal_nonce_propagates_entropy_error() {
    let r = seal_nonce_from_entropy(&FailingEntropy);
    assert!(r.is_err());
}

// ---- Rotationsschranke (§16.3, D-28) -----------------------------------------

#[test]
fn rotation_limit_is_2_pow_32() {
    assert_eq!(SEAL_ROTATION_LIMIT, 1u64 << 32);
}

#[test]
fn rotation_boundary() {
    assert!(!rotation_due(0));
    assert!(!rotation_due(SEAL_ROTATION_LIMIT - 1));
    assert!(rotation_due(SEAL_ROTATION_LIMIT));
    assert!(rotation_due(SEAL_ROTATION_LIMIT + 1));
}

// ---- Clusterseal-Schlüssel (§10.10) und KEYS-Ableitung (§10.3) ---------------

#[test]
fn cluster_seal_key_depends_on_epoch_and_cluster_key() {
    let kdf = Blake3Kdf;
    let ck_a = key(0xA1);
    let ck_b = key(0xB2);
    let e0 = derive_cluster_seal_key(&kdf, &ck_a, 0);
    let e1 = derive_cluster_seal_key(&kdf, &ck_a, 1);
    let other = derive_cluster_seal_key(&kdf, &ck_b, 0);
    assert_ne!(
        e0.expose(),
        e1.expose(),
        "key_epoch muss den Schlüssel ändern"
    );
    assert_ne!(
        e0.expose(),
        other.expose(),
        "Clusterschlüssel muss den Schlüssel ändern"
    );
    assert_ne!(e0.expose(), ck_a.expose());
}

#[test]
fn keys_key_is_bound_to_instance_id() {
    let kdf = Blake3Kdf;
    let kek = key(0x42);
    let k1 = derive_keys_key(&kdf, &kek, &[0x01u8; 16]);
    let k2 = derive_keys_key(&kdf, &kek, &[0x02u8; 16]);
    assert_ne!(
        k1.expose(),
        k2.expose(),
        "KEYS-Schlüssel muss an instance_id gebunden sein"
    );
}

#[test]
fn keys_aad_is_30_bytes_with_magic_prefix() {
    let aad = KeysFile::build_aad(&[0x33u8; 16], 1, 9);
    assert_eq!(aad.len(), 30);
    assert_eq!(aad.get(0..4), Some(b"CTXK".as_slice()));
    assert_eq!(aad.get(4..6), Some(1u16.to_le_bytes().as_slice()));
    assert_eq!(aad.get(6..22), Some([0x33u8; 16].as_slice()));
    assert_eq!(aad.get(22..30), Some(9u64.to_le_bytes().as_slice()));
}

// ---- Begrenzter Cache (§16.4) -------------------------------------

#[test]
fn cache_insert_and_get() {
    let mut c = KeyCache::new(4);
    assert!(c.is_empty());
    c.insert(cid(1), 0, key(10));
    assert_eq!(c.len(), 1);
    let got = c.get(cid(1), 0).map(|k| *k.expose());
    assert_eq!(got, Some([10u8; 32]));
    assert!(
        c.get(cid(1), 1).is_none(),
        "andere Epoche ist nicht enthalten"
    );
    assert!(
        c.get(cid(2), 0).is_none(),
        "anderer Cluster ist nicht enthalten"
    );
}

#[test]
fn cache_replaces_same_cluster_epoch() {
    let mut c = KeyCache::new(4);
    c.insert(cid(1), 0, key(10));
    c.insert(cid(1), 0, key(11));
    assert_eq!(c.len(), 1);
    assert_eq!(c.get(cid(1), 0).map(|k| *k.expose()), Some([11u8; 32]));
}

#[test]
fn cache_evicts_oldest_when_full() {
    let mut c = KeyCache::new(2);
    c.insert(cid(1), 0, key(1));
    c.insert(cid(2), 0, key(2));
    c.insert(cid(3), 0, key(3));
    assert_eq!(c.len(), 2);
    assert!(c.get(cid(1), 0).is_none(), "ältester Eintrag ist verdrängt");
    assert!(c.get(cid(2), 0).is_some());
    assert!(c.get(cid(3), 0).is_some());
}

#[test]
fn cache_capacity_zero_stores_nothing() {
    let mut c = KeyCache::new(0);
    c.insert(cid(1), 0, key(1));
    assert!(c.is_empty());
}

#[test]
fn cache_invalidate_removes_all_epochs_of_cluster() {
    let mut c = KeyCache::new(8);
    c.insert(cid(1), 0, key(1));
    c.insert(cid(1), 1, key(2));
    c.insert(cid(2), 0, key(3));
    c.invalidate_cluster(cid(1));
    assert_eq!(c.len(), 1);
    assert!(c.get(cid(1), 0).is_none());
    assert!(c.get(cid(1), 1).is_none());
    assert!(c.get(cid(2), 0).is_some());
}

#[test]
fn cache_clear_empties() {
    let mut c = KeyCache::new(3);
    c.insert(cid(1), 0, key(1));
    c.clear();
    assert!(c.is_empty());
}
