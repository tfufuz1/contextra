use contextra_crypto_ref::mac::Blake3Mac;
use contextra_storage::erase::ledger::{decode_record, encode_record, LedgerEvent, LedgerEventKind, LedgerRecord};
use contextra_storage::ports::{Error, Key32, Ticket, TombId};
use contextra_storage::ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage::env::memfs::MemFs;
use contextra_storage::erase::ledger::LedgerWriter;
use std::path::Path;

fn record() -> LedgerRecord {
    LedgerRecord {
        sequence: 1,
        previous_hash: [0; 32],
        clock_sample: 42,
        event: LedgerEvent {
            kind: LedgerEventKind::EraseIntent,
            tomb_epoch: 0,
            ticket: Ticket::from_bytes([4; 16]),
            tombs: vec![TombId::from_bytes([1; 32]), TombId::from_bytes([2; 32])],
            evidence_digest: [3; 32],
            scope_ciphertext: vec![9, 8, 7],
        },
        mac: [0; 32],
    }
}

#[test]
fn ledger_record_roundtrips_and_returns_chain_head() {
    let key = Key32::from_array([9; 32]);
    let bytes = encode_record(&record(), &key, &Blake3Mac).unwrap();
    let (decoded, used) = decode_record(&bytes, 1, [0; 32], &key, &Blake3Mac).unwrap();
    assert_eq!(used, bytes.len());
    assert_eq!(decoded.event.tombs.len(), 2);
    assert_ne!(decoded.mac, [0; 32]);
}

#[test]
fn ledger_rejects_tampering_and_wrong_chain_predecessor() {
    let key = Key32::from_array([9; 32]);
    let mut bytes = encode_record(&record(), &key, &Blake3Mac).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    assert!(matches!(decode_record(&bytes, 1, [0; 32], &key, &Blake3Mac), Err(Error::Corrupt { .. })));

    let bytes = encode_record(&record(), &key, &Blake3Mac).unwrap();
    assert!(matches!(decode_record(&bytes, 1, [1; 32], &key, &Blake3Mac), Err(Error::Corrupt { .. })));
}

#[test]
fn ledger_requires_canonical_tomb_order() {
    let key = Key32::from_array([9; 32]);
    let mut bad = record();
    bad.event.tombs.reverse();
    assert!(matches!(encode_record(&bad, &key, &Blake3Mac), Err(Error::Limit { .. })));
}

#[test]
fn synced_ledger_appends_recover_as_one_valid_chain() {
    let fs = MemFs::new();
    let ctx = IoCtx { role: FileRole::Ledger, purpose: "ledger_test" };
    let file = fs.create_new(Path::new("/ledger"), ctx).unwrap();
    file.preallocate(64 * 1024, ctx).unwrap();
    fs.sync_dir(Path::new("/"), ctx).unwrap();
    let key = Key32::from_array([5; 32]);
    let mut writer = LedgerWriter::recover(file.as_ref(), &key, &Blake3Mac).unwrap().0;
    writer.append(1, record().event).unwrap();
    let crashed = fs.crash_image().unwrap();
    let replay = crashed.open(Path::new("/ledger"), ctx).unwrap();
    let (recovered, records) = LedgerWriter::recover(replay.as_ref(), &key, &Blake3Mac).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(recovered.next_sequence(), 2);
}
