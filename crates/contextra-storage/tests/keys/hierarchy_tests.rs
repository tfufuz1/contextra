//! # Schlüsselhierarchie Tests
//!
//! SPEC §15.2:
//! - KEK -> KEYS-Schlüssel
//! - root_key -> Redo-, Catalog-, FORMAT-MAC-, Token-Schlüssel
//! - space_dek -> Run-, Blob-, Segment-Schlüssel

use contextra_crypto_ref::Blake3Kdf;
use contextra_storage::keys::hierarchy::*;
use contextra_storage::keys::secret::Secret32;
use std::collections::HashSet;

#[test]
fn test_keys_key_derivation_deterministic() {
    let kdf = Blake3Kdf;
    let kek = Secret32::new([0x42u8; 32]);

    let key1 = derive_keys_key(&kdf, &kek);
    let key2 = derive_keys_key(&kdf, &kek);

    assert_eq!(key1.expose(), key2.expose());
    assert_ne!(key1.expose(), kek.expose());
}

#[test]
fn test_root_key_derivations_disjoint() {
    let kdf = Blake3Kdf;
    let root = Secret32::new([0x55u8; 32]);

    let redo_key = derive_redo_key(&kdf, &root, 1);
    let cat_key = derive_catalog_key(&kdf, &root, 1);
    let fmt_key = derive_format_mac_key(&kdf, &root);
    let tok_key = derive_token_key(&kdf, &root);

    let mut keys = HashSet::new();
    assert!(keys.insert(*redo_key.expose()));
    assert!(keys.insert(*cat_key.expose()));
    assert!(keys.insert(*fmt_key.expose()));
    assert!(keys.insert(*tok_key.expose()));
    assert_eq!(keys.len(), 4, "Alle abgeleiteten Schlüssel müssen paarweise verschieden sein");
}

#[test]
fn test_space_dek_derivations_disjoint() {
    let kdf = Blake3Kdf;
    let dek = Secret32::new([0x77u8; 32]);

    let run_key = derive_run_key(&kdf, &dek, 0, 100);
    let blob_key = derive_blob_key(&kdf, &dek, 0, 100);
    let seg_key = derive_segment_key(&kdf, &dek, 0, 1, 100);

    let mut keys = HashSet::new();
    assert!(keys.insert(*run_key.expose()));
    assert!(keys.insert(*blob_key.expose()));
    assert!(keys.insert(*seg_key.expose()));
    assert_eq!(keys.len(), 3);
}

#[test]
fn test_different_epochs_produce_different_keys() {
    let kdf = Blake3Kdf;
    let dek = Secret32::new([0x88u8; 32]);

    let key_epoch_0 = derive_run_key(&kdf, &dek, 0, 1);
    let key_epoch_1 = derive_run_key(&kdf, &dek, 1, 1);

    assert_ne!(
        key_epoch_0.expose(),
        key_epoch_1.expose(),
        "Verschiedene Epochen müssen verschiedene Run-Schlüssel erzeugen"
    );
}
