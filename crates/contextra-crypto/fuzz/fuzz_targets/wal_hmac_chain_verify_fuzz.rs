//! Fuzz targets for contextra-crypto:
//! - wal_hmac_chain_verify_fuzz: HMAC chain validation
//! - header_decode_fuzz: Header decoding robustness
//! - envelope_decode_fuzz: Record envelope decoding robustness
//! - slot_unwrap_fuzz: Keyslot unwrap robustness
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use contextra_crypto::*;

#[derive(Arbitrary, Debug)]
pub struct FuzzData {
    pub raw_header: [u8; 1024],
    pub raw_envelope: Vec<u8>,
    pub wrapped_slot: [u8; 48],
    pub kek: [u8; 32],
    pub store_id: [u8; 16],
    pub slot_salt: [u8; 32],
    pub nonce: [u8; 12],
    pub m_kib: u32,
    pub t: u8,
    pub p: u8,
    pub prev_chain: [u8; 32],
    pub log_entry: Vec<u8>,
    pub dek_bytes: [u8; 32],
    pub record_key: Vec<u8>,
}

fuzz_target!(|data: FuzzData| {
    // 1. Header decoding robustness (never panic, return Ok or Err(Corrupt))
    let _ = Header::decode_checked(&data.raw_header);

    // 2. Keyslot unwrap robustness
    let kek = SecretKey::<32>::new(data.kek);
    let aad = SlotAad {
        store_id: data.store_id,
        slot_id: 1,
        anchor_id: 1,
        kdf_id: 1,
        m_kib: data.m_kib,
        t: data.t,
        p: data.p,
        slot_salt: data.slot_salt,
    };
    let _ = unwrap_master(&kek, &aad, data.nonce, &data.wrapped_slot);

    // 3. Record envelope decoding robustness
    let dek = Dek(SecretKey::<32>::new(data.dek_bytes));
    let _ = open_record(&dek, &data.record_key, &data.raw_envelope);

    // 4. Chain next robustness
    let _ = chain_next(&data.prev_chain, &data.log_entry);

    // 5. KdfParams parsing
    let _ = KdfParams::from_header(data.m_kib, data.t, data.p);
});
