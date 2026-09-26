#![allow(deprecated)]

use contextra_crypto::wal_crypto::{IntegrityVerifier, WalEntrySnapshot, WalHmac};
use proptest::prelude::*;

fn compute_v2_checksum(
    key: &[u8],
    prev_hmac: [u8; 32],
    seq_no: u64,
    op_type: u8,
    k: &[u8],
    v: &[u8],
) -> [u8; 32] {
    let mut mac = WalHmac::new(key).expect("WalHmac init failed");
    mac.update(&prev_hmac);
    mac.update(&seq_no.to_le_bytes());
    mac.update(&[op_type]);
    if op_type == 0 {
        mac.update(k);
        mac.update(v);
    } else if op_type == 1 {
        mac.update(k);
    } else if op_type == 2 {
        let committed = if v.first().copied().unwrap_or(0) != 0 {
            1u8
        } else {
            0u8
        };
        mac.update(&[committed]);
    }
    mac.finalize()
}

fn compute_v3_checksum(
    key: &[u8],
    prev_hmac: [u8; 32],
    seq_no: u64,
    tx_id: u64,
    op_type: u8,
    k: &[u8],
    v: &[u8],
) -> [u8; 32] {
    let mut mac = WalHmac::new(key).expect("WalHmac init failed");
    mac.update(&prev_hmac);
    mac.update(&seq_no.to_le_bytes());
    mac.update(&tx_id.to_le_bytes());
    if op_type == 0 {
        mac.update(&[0u8]);
        mac.update(&(k.len() as u32).to_le_bytes());
        mac.update(k);
        mac.update(&(v.len() as u32).to_le_bytes());
        mac.update(v);
    } else if op_type == 1 {
        mac.update(&[1u8]);
        mac.update(&(k.len() as u32).to_le_bytes());
        mac.update(k);
    } else if op_type == 2 {
        mac.update(&[2u8]);
        let committed = if v.first().copied().unwrap_or(0) != 0 {
            1u8
        } else {
            0u8
        };
        mac.update(&[committed]);
    }
    mac.finalize()
}

/// Dokumentiert und beweist die bekannte HMAC-Kollisionslücke im Legacy-V2-Pfad.
///
/// DOKUMENTATION DER LÜCKE (Architektur-Audit Befund F2 / Spec §22.2b):
/// Im `verify_and_update_v2`-Pfad werden für `op_type == 0` (Put) der Schlüssel (`key`)
/// und der Wert (`value`) ohne Längenpräfix direkt hintereinander in den HMAC eingespeist.
/// Dadurch führen zwei unterschiedliche (Key, Value)-Paare mit identischer Byte-Konkatenation
/// (wie key="ab", value="c" vs. key="a", value="bc") zu demselben HMAC-Bytestrom.
///
/// Dieser Test stellt sicher, dass dieses Verhalten explizit dokumentiert ist.
/// Ein GLEICHER resultierender Hash ist hier das erwartete (wenn auch unerwünschte)
/// Verhalten des Legacy-v2-Pfades.
#[test]
fn v2_hmac_is_not_length_prefixed_documented_gap() {
    let integrity_key = b"integrity-key-32-bytes-v2-gap!!";
    let prev_hmac = [0u8; 32];
    let seq_no = 42u64;
    let tx_id = 42u64;
    let op_type = 0u8; // Put

    // Paar 1: key = b"ab", value = b"c"
    let key1 = b"ab".to_vec();
    let val1 = b"c".to_vec();
    let checksum1 = compute_v2_checksum(integrity_key, prev_hmac, seq_no, op_type, &key1, &val1);
    let entry1 = WalEntrySnapshot {
        tx_id,
        seq_no,
        op_type,
        key: key1,
        value: val1,
        checksum: checksum1,
        prev_hmac,
    };

    // Paar 2: key = b"a", value = b"bc"
    let key2 = b"a".to_vec();
    let val2 = b"bc".to_vec();
    let checksum2 = compute_v2_checksum(integrity_key, prev_hmac, seq_no, op_type, &key2, &val2);
    let entry2 = WalEntrySnapshot {
        tx_id,
        seq_no,
        op_type,
        key: key2,
        value: val2,
        checksum: checksum2,
        prev_hmac,
    };

    let mut verifier1 = IntegrityVerifier::new(integrity_key);
    let mut verifier2 = IntegrityVerifier::new(integrity_key);

    verifier1
        .verify_and_update_v2(&entry1, 0)
        .expect("entry1 verification should succeed");
    verifier2
        .verify_and_update_v2(&entry2, 0)
        .expect("entry2 verification should succeed");

    // DOKUMENTATION DER F2-LÜCKE: assert_eq! belegt, dass der V2-Pfad kollidierende HMACs liefert.
    assert_eq!(
        verifier1.last_hmac_snapshot(),
        verifier2.last_hmac_snapshot(),
        "Legacy V2 WAL HMAC MUST collide for boundary-shifted key/value pairs (F2 gap)"
    );
}

/// Beweist, dass der V3-Pfad durch Längenpräfixe kollisionssicher ist.
#[test]
fn v3_hmac_is_length_prefixed_no_collision() {
    let integrity_key = b"integrity-key-32-bytes-v3-ok!!!";
    let prev_hmac = [0u8; 32];
    let seq_no = 42u64;
    let tx_id = 42u64;
    let op_type = 0u8; // Put

    // Paar 1: key = b"ab", value = b"c"
    let key1 = b"ab".to_vec();
    let val1 = b"c".to_vec();
    let checksum1 =
        compute_v3_checksum(integrity_key, prev_hmac, seq_no, tx_id, op_type, &key1, &val1);
    let entry1 = WalEntrySnapshot {
        tx_id,
        seq_no,
        op_type,
        key: key1,
        value: val1,
        checksum: checksum1,
        prev_hmac,
    };

    // Paar 2: key = b"a", value = b"bc"
    let key2 = b"a".to_vec();
    let val2 = b"bc".to_vec();
    let checksum2 =
        compute_v3_checksum(integrity_key, prev_hmac, seq_no, tx_id, op_type, &key2, &val2);
    let entry2 = WalEntrySnapshot {
        tx_id,
        seq_no,
        op_type,
        key: key2,
        value: val2,
        checksum: checksum2,
        prev_hmac,
    };

    let mut verifier1 = IntegrityVerifier::new(integrity_key);
    let mut verifier2 = IntegrityVerifier::new(integrity_key);

    verifier1
        .verify_and_update_v3(&entry1, 0)
        .expect("entry1 V3 verification should succeed");
    verifier2
        .verify_and_update_v3(&entry2, 0)
        .expect("entry2 V3 verification should succeed");

    // V3 liefert aufgrund der Längenpräfixe unterschiedliche HMAC-Zustände.
    assert_ne!(
        verifier1.last_hmac_snapshot(),
        verifier2.last_hmac_snapshot(),
        "V3 WAL HMAC MUST NOT collide for boundary-shifted key/value pairs"
    );
}

proptest! {
    /// Property-basierter Test, der für beliebige Byteströme und Aufteilungen beweist,
    /// dass `verify_and_update_v2` bei verschobener Feldgrenze dieselben HMAC-Zustände erzeugt.
    #[test]
    fn prop_v2_arbitrary_boundary_shift_collides(
        a in prop::collection::vec(any::<u8>(), 0..100),
        b in prop::collection::vec(any::<u8>(), 0..100),
        n in any::<usize>(),
    ) {
        let combined = [a.as_slice(), b.as_slice()].concat();
        let total_len = combined.len();

        if total_len >= 2 {
            // Position n innerhalb der Grenzen des kombinierten Vektors verankern
            let split1 = n % (total_len + 1);
            // Position m != split1 wählen
            let split2 = (split1 + 1) % (total_len + 1);

            let key1 = combined[..split1].to_vec();
            let val1 = combined[split1..].to_vec();

            let key2 = combined[..split2].to_vec();
            let val2 = combined[split2..].to_vec();

            let integrity_key = b"integrity-key-32-bytes-proptest";
            let prev_hmac = [0u8; 32];
            let seq_no = 1u64;
            let tx_id = 1u64;
            let op_type = 0u8;

            let checksum1 = compute_v2_checksum(integrity_key, prev_hmac, seq_no, op_type, &key1, &val1);
            let entry1 = WalEntrySnapshot {
                tx_id,
                seq_no,
                op_type,
                key: key1,
                value: val1,
                checksum: checksum1,
                prev_hmac,
            };

            let checksum2 = compute_v2_checksum(integrity_key, prev_hmac, seq_no, op_type, &key2, &val2);
            let entry2 = WalEntrySnapshot {
                tx_id,
                seq_no,
                op_type,
                key: key2,
                value: val2,
                checksum: checksum2,
                prev_hmac,
            };

            let mut verifier1 = IntegrityVerifier::new(integrity_key);
            let mut verifier2 = IntegrityVerifier::new(integrity_key);

            verifier1.verify_and_update_v2(&entry1, 0).unwrap();
            verifier2.verify_and_update_v2(&entry2, 0).unwrap();

            // Beweist, dass V2 für zwei beliebige unterschiedliche Aufteilungen desselben Bytestroms kollidiert.
            prop_assert_eq!(verifier1.last_hmac_snapshot(), verifier2.last_hmac_snapshot());
        }
    }
}
