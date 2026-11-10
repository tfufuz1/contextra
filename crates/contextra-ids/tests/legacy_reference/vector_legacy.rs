#![allow(dead_code)]
// Origin: crates/contextra-vector/src/types.rs
// Commit: 1054c11b6
// Lines: 50-70

pub fn vector_doc_id_from_key(key: &str) -> Result<u64, &'static str> {
    if key.is_empty() {
        return Err("Key cannot be empty");
    }
    let hash = blake3::hash(key.as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash.as_bytes()[..8]);
    Ok(u64::from_le_bytes(bytes))
}
