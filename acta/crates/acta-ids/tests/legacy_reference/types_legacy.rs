#![allow(dead_code)]
// Origin: crates/acta-types/src/types/domain/ids.rs
// Commit: 1054c11b6
// Lines: 200-280

pub fn hash_key_u64(key: &str) -> u64 {
    let hash = blake3::hash(key.as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash.as_bytes()[..8]);
    u64::from_le_bytes(bytes)
}

pub fn hash_key_u128(key: &str) -> u128 {
    let hash = blake3::hash(key.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash.as_bytes()[..16]);
    u128::from_le_bytes(bytes)
}

pub fn doc_id_from_key_64(key: &str) -> Result<u64, &'static str> {
    if key.is_empty() {
        return Err("Key cannot be empty");
    }
    Ok(hash_key_u64(key))
}

pub fn doc_id_from_key_128(key: &str) -> Result<u128, &'static str> {
    if key.is_empty() {
        return Err("Key cannot be empty");
    }
    Ok(hash_key_u128(key))
}
