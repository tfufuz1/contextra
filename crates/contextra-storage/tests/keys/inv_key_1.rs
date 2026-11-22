//! # INV-KEY-1 Property Test
//!
//! SPEC §15.2, §18 (INV-KEY-1):
//! "Kein (Schlüssel, Nonce)-Paar kommt je zweimal vor."
//!
//! Über alle Konstruktoren und beliebige Eingaben darf kein Paar (Schlüssel, Nonce)
//! zweimal vorkommen, sofern die Eingaben verschieden sind.

use contextra_crypto_ref::Blake3Kdf;
use contextra_storage::keys::hierarchy::*;
use contextra_storage::keys::nonce::*;
use contextra_storage::keys::secret::Secret32;
use contextra_storage_ports::Kdf;
use proptest::prelude::*;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CryptoOp {
    RunBlock {
        epoch: u32,
        run_id: u64,
        offset: u64,
    },
    RedoFrame {
        segment_id: u32,
        frame_index: u32,
    },
    RedoOpBody {
        domain: u32,
        segment_id: u32,
        frame_index: u32,
        op_index: u32,
    },
    BlobRecord {
        epoch: u32,
        blob_no: u64,
        offset: u64,
    },
    BlobBody {
        domain: u32,
        blob_no: u64,
        offset: u32,
    },
    CatalogRecord {
        catalog_no: u64,
        record_index: u64,
    },
    Keys {
        random: [u8; 12],
    },
}

fn compute_pair(
    op: &CryptoOp,
    kdf: &Blake3Kdf,
    kek: &Secret32,
    root_key: &Secret32,
    space_dek: &Secret32,
) -> ([u8; 32], [u8; 12]) {
    match *op {
        CryptoOp::RunBlock {
            epoch,
            run_id,
            offset,
        } => {
            let key = derive_run_key(kdf, space_dek, epoch, run_id);
            let nonce = run_block_nonce(offset);
            (*key.expose(), nonce)
        }
        CryptoOp::RedoFrame {
            segment_id,
            frame_index,
        } => {
            let key = derive_redo_key(kdf, root_key, segment_id);
            let nonce = redo_frame_nonce(frame_index);
            (*key.expose(), nonce)
        }
        CryptoOp::RedoOpBody {
            domain,
            segment_id,
            frame_index,
            op_index,
        } => {
            // Domänenschlüssel simuliert über Kdf
            let domain_bytes = domain.to_le_bytes();
            let key_bytes = kdf.derive(
                root_key.expose(),
                &[b"ctxs domain v1".as_slice(), domain_bytes.as_slice()],
            );
            let nonce = redo_op_body_nonce(segment_id, frame_index, op_index);
            (key_bytes, nonce)
        }
        CryptoOp::BlobRecord {
            epoch,
            blob_no,
            offset,
        } => {
            let key = derive_blob_key(kdf, space_dek, epoch, blob_no);
            let nonce = blob_record_nonce(offset);
            (*key.expose(), nonce)
        }
        CryptoOp::BlobBody {
            domain,
            blob_no,
            offset,
        } => {
            let domain_bytes = domain.to_le_bytes();
            let key_bytes = kdf.derive(
                root_key.expose(),
                &[b"ctxs domain v1".as_slice(), domain_bytes.as_slice()],
            );
            let nonce = blob_body_nonce(blob_no, offset);
            (key_bytes, nonce)
        }
        CryptoOp::CatalogRecord {
            catalog_no,
            record_index,
        } => {
            let key = derive_catalog_key(kdf, root_key, catalog_no);
            let nonce = catalog_record_nonce(record_index);
            (*key.expose(), nonce)
        }
        CryptoOp::Keys { random } => {
            let key = derive_keys_key(kdf, kek);
            let nonce = keys_nonce(random);
            (*key.expose(), nonce)
        }
    }
}

fn arb_crypto_op() -> impl Strategy<Value = CryptoOp> {
    prop_oneof![
        (any::<u32>(), any::<u64>(), any::<u64>()).prop_map(|(epoch, run_id, offset)| {
            CryptoOp::RunBlock {
                epoch,
                run_id,
                offset,
            }
        }),
        (any::<u32>(), any::<u32>()).prop_map(|(segment_id, frame_index)| CryptoOp::RedoFrame {
            segment_id,
            frame_index,
        }),
        (any::<u32>(), any::<u32>(), any::<u32>(), 0..0x00FF_FFFFu32).prop_map(
            |(domain, segment_id, frame_index, op_index)| CryptoOp::RedoOpBody {
                domain,
                segment_id,
                frame_index,
                op_index,
            }
        ),
        (any::<u32>(), any::<u64>(), any::<u64>()).prop_map(|(epoch, blob_no, offset)| {
            CryptoOp::BlobRecord {
                epoch,
                blob_no,
                offset,
            }
        }),
        (any::<u32>(), 0..0x00FF_FFFF_FFFF_FFFFu64, any::<u32>()).prop_map(
            |(domain, blob_no, offset)| CryptoOp::BlobBody {
                domain,
                blob_no,
                offset,
            }
        ),
        (any::<u64>(), any::<u64>()).prop_map(|(catalog_no, record_index)| {
            CryptoOp::CatalogRecord {
                catalog_no,
                record_index,
            }
        }),
        any::<[u8; 12]>().prop_map(|random| CryptoOp::Keys { random }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_inv_key_1_uniqueness(ops in prop::collection::vec(arb_crypto_op(), 1..64)) {
        // // INV-KEY-1: Über alle Konstruktoren und zufällige Eingaben darf kein Paar (Schlüssel, Nonce) zweimal vorkommen
        let kdf = Blake3Kdf;
        let kek = Secret32::new([0x01u8; 32]);
        let root_key = Secret32::new([0x02u8; 32]);
        let space_dek = Secret32::new([0x03u8; 32]);

        let mut seen_inputs = HashSet::new();
        let mut seen_pairs = HashSet::new();

        for op in ops {
            if seen_inputs.insert(op.clone()) {
                let pair = compute_pair(&op, &kdf, &kek, &root_key, &space_dek);
                prop_assert!(
                    seen_pairs.insert(pair),
                    "INV-KEY-1 verletzt: Doppeltes (Schlüssel, Nonce)-Paar für verschiedene Eingabe {:?}",
                    op
                );
            }
        }
    }
}

#[test]
fn test_inv_key_1_pairwise_cross_constructor_disjoint_nonces() {
    // // INV-KEY-1: Prüft deterministisch, dass Nonces mit verschiedenen Präfixen nie kollidieren
    let run = run_block_nonce(0);
    let redo = redo_frame_nonce(0);
    let redo_op = redo_op_body_nonce(0, 0, 0);
    let blob_rec = blob_record_nonce(0);
    let blob_body = blob_body_nonce(0, 0);

    assert_eq!(run[0], 0x01);
    assert_eq!(redo[0], 0x02);
    assert_eq!(redo_op[0], 0x03);
    assert_eq!(blob_rec[0], 0x04);
    assert_eq!(blob_body[0], 0x05);

    let nonces = [run, redo, redo_op, blob_rec, blob_body];
    for i in 0..nonces.len() {
        for j in (i + 1)..nonces.len() {
            assert_ne!(
                nonces[i], nonces[j],
                "Nonces zwischen Konstruktor {i} und {j} dürfen nicht übereinstimmen"
            );
        }
    }
}
