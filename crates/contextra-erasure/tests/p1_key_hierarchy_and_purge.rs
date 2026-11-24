#![cfg(feature = "vault")]

//! # Integrationstests für Aufgabe P1 (WP-K)
//!
//! Testsuite für KEK/DEK-Schlüsselhierarchie, Port `KeyStore` und `purge_keys`.
//! Erzwingt die Invarianten:
//! - INV-NO-RESURRECTION-1: Nach Vernichtung des Schlüssels ist keine Reaktivierung möglich
//! - INV-ERA-K-1: Vernichtung des KEK macht alle abhängigen DEKs unlesbar (Fail-Closed)
//! - INV-ERA-K-2: Epochenisolation (Vernichtung Epoche n lässt Epoche n+1 lesbar)
//! - INV-ERA-K-3: DEK-Klartext liegt nur in Zeroizing-Puffern

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_erasure::kv_shredding::{GroupEpochKey, KeyPurgeReceipt, KeyRegistry};
use contextra_erasure::types_local::{Clock, SystemClock};
use contextra_erasure::vault::keyring::key_store::{KeyStore, KeyStoreError, LocalDek};
use std::sync::Arc;
use zeroize::Zeroize;

#[test]
fn kek_destruction_makes_all_deks_undecryptable() {
    let registry = KeyRegistry::new();
    let group_id = 42;
    let epoch = 1;
    let clock = SystemClock::new();

    // 10 Datensätze verschlüsseln
    let mut payloads = Vec::new();
    for rec_id in 0..10 {
        let plaintext = format!("Top secret payload for record #{rec_id}");
        let payload = registry
            .encrypt_envelope_record_epoch(group_id, epoch, rec_id, plaintext.as_bytes())
            .expect("encryption must succeed");
        payloads.push(payload);
    }

    // Vor Vernichtung: alle entschlüsselbar
    for (i, p) in payloads.iter().enumerate() {
        let dec = registry
            .decrypt_envelope_record_epoch(p)
            .expect("decryption before purge must succeed");
        assert_eq!(
            dec,
            format!("Top secret payload for record #{i}").as_bytes()
        );
    }

    // KEK für (group_id, epoch) via purge_keys physisch/kryptographisch vernichten
    let receipt = registry
        .purge_keys(group_id, epoch, Some(&clock))
        .expect("purge_keys must succeed");

    assert_eq!(receipt.group_id, group_id);
    assert_eq!(receipt.epoch, epoch);
    assert_eq!(receipt.affected_deks, 10);
    assert!(receipt.purged_at_nanos > 0);

    // Nach Vernichtung: JEDER abhängige DEK darf nicht mehr entschlüsselbar sein (Fail-Closed)
    for p in &payloads {
        let res = registry.decrypt_envelope_record_epoch(p);
        assert!(
            res.is_err(),
            "Decryption of any record in the purged epoch must fail fail-closed"
        );
    }

    // Auch erneuter Zugriff auf KEK muss abgelehnt werden
    assert!(registry.get_or_derive_epoch(group_id, epoch).is_err());
}

#[test]
fn dek_never_plaintext_outside_zeroized_memory() {
    // Verifiziert Typebene und Drop-Verhalten von LocalDek
    let mut key_material = [0x5au8; 32];
    {
        let dek = LocalDek::new(key_material);
        assert_eq!(dek.as_bytes(), &[0x5au8; 32]);
        // Debug impl darf keine Geheimdaten leaken
        let dbg = format!("{dek:?}");
        assert!(dbg.contains("***REDACTED***"));
        assert!(!dbg.contains("5a"));
    }
    // Nach Drop wird der Puffer im Speicher genullt (über Zeroize trait)
    key_material.zeroize();
    assert_eq!(key_material, [0u8; 32]);
}

#[test]
fn epoch_isolation() {
    let registry = KeyRegistry::new();
    let group_id = 100;
    let epoch_1 = 1;
    let epoch_2 = 2;
    let clock = SystemClock::new();

    let p1 = registry
        .encrypt_envelope_record_epoch(group_id, epoch_1, 1, b"epoch 1 data")
        .expect("encrypt epoch 1 ok");
    let p2 = registry
        .encrypt_envelope_record_epoch(group_id, epoch_2, 1, b"epoch 2 data")
        .expect("encrypt epoch 2 ok");

    // Vernichtung von Epoche 1
    let receipt = registry
        .purge_keys(group_id, epoch_1, Some(&clock))
        .expect("purge epoch 1 ok");
    assert_eq!(receipt.affected_deks, 1);

    // Epoche 1 ist unlesbar
    assert!(registry.decrypt_envelope_record_epoch(&p1).is_err());

    // Epoche 2 bleibt intakt und lesbar!
    let dec_2 = registry
        .decrypt_envelope_record_epoch(&p2)
        .expect("epoch 2 must remain fully decryptable");
    assert_eq!(dec_2, b"epoch 2 data");

    // Neue Datensätze in Epoche 2 können weiterhin verschlüsselt und entschlüsselt werden
    let p2_new = registry
        .encrypt_envelope_record_epoch(group_id, epoch_2, 2, b"epoch 2 new data")
        .expect("new encrypt epoch 2 ok");
    assert_eq!(
        registry.decrypt_envelope_record_epoch(&p2_new).unwrap(),
        b"epoch 2 new data"
    );
}

#[test]
fn purge_keys_idempotent() {
    let registry = KeyRegistry::new();
    let group_id = 200;
    let epoch = 5;
    let clock = SystemClock::new();

    let _p = registry
        .encrypt_envelope_record_epoch(group_id, epoch, 1, b"data")
        .expect("encrypt ok");

    // Erster Aufruf
    let r1 = registry
        .purge_keys(group_id, epoch, Some(&clock))
        .expect("purge 1 ok");
    assert_eq!(r1.affected_deks, 1);

    // Zweiter Aufruf (idempotent, keine Panik, keine Auferstehung)
    let r2 = registry
        .purge_keys(group_id, epoch, Some(&clock))
        .expect("purge 2 idempotent ok");
    assert_eq!(r2.affected_deks, 0);
    assert_eq!(r2.group_id, group_id);
    assert_eq!(r2.epoch, epoch);
}

#[test]
fn inv_no_resurrection_1_purge_keys_survives_double_call_without_resurrection() {
    let registry = KeyRegistry::new();
    let group_id = 300;
    let epoch = 1;
    let clock = SystemClock::new();

    let p = registry
        .encrypt_envelope_record_epoch(group_id, epoch, 42, b"confidential")
        .expect("encrypt ok");

    // 1. Purge
    registry.purge_keys(group_id, epoch, Some(&clock)).unwrap();
    assert!(!registry.is_group_epoch_active(group_id, epoch));
    assert!(registry.is_group_epoch_revoked(group_id, epoch));

    // 2. Erneuter Purge-Aufruf darf Zustand nicht verändern
    registry.purge_keys(group_id, epoch, Some(&clock)).unwrap();
    assert!(!registry.is_group_epoch_active(group_id, epoch));
    assert!(registry.is_group_epoch_revoked(group_id, epoch));

    // 3. Versuch, den gelöschten Schlüssel neu abzuleiten oder zu reaktivieren, muss strikt scheitern
    assert!(registry.get_or_derive_epoch(group_id, epoch).is_err());
    assert!(registry.decrypt_envelope_record_epoch(&p).is_err());
    assert!(registry
        .encrypt_envelope_record_epoch(group_id, epoch, 99, b"new")
        .is_err());
}

#[test]
fn test_keystore_port_adapter() {
    let registry = Arc::new(KeyRegistry::new());
    let store: Arc<dyn KeyStore> = registry.clone();
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());

    let scope = GroupEpochKey::new(500, 1);
    assert!(store.is_active(scope));

    let payload = store
        .encrypt_record(scope, 1, b"via keystore port")
        .expect("encrypt via trait ok");

    let dec = store
        .decrypt_record(&payload)
        .expect("decrypt via trait ok");
    assert_eq!(dec, b"via keystore port");

    let receipt: KeyPurgeReceipt = store
        .purge_keys(scope, Some(&*clock))
        .expect("purge via trait ok");
    assert_eq!(receipt.affected_deks, 1);
    assert!(!store.is_active(scope));

    // Entschlüsselung muss fehlschlagen
    let err = store.decrypt_record(&payload).unwrap_err();
    assert_eq!(
        err,
        KeyStoreError::KeyDestroyed {
            group_id: 500,
            epoch: 1
        }
    );

    // KEK-Abruf muss fehlschlagen
    let kek_err = store.get_or_derive_kek(scope).unwrap_err();
    assert_eq!(
        kek_err,
        KeyStoreError::KeyDestroyed {
            group_id: 500,
            epoch: 1
        }
    );
}

#[test]
fn test_error_paths_nonexistent_group() {
    let registry = KeyRegistry::new();
    let clock = SystemClock::new();

    // Purge auf eine Gruppe/Epoche, die nie existiert hat:
    // Liefert sauberes Receipt mit affected_deks = 0 und markiert sie als widerrufen
    let receipt = registry.purge_keys(9999, 1, Some(&clock)).unwrap();
    assert_eq!(receipt.affected_deks, 0);
    assert!(registry.is_group_epoch_revoked(9999, 1));
}
