use serde::{Deserialize, Serialize};

/// Pointer referencing a blob stored within an append-only blob segment file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobPointer {
    pub segment_id: u64,
    pub offset: u64,
    pub len: u32,
    pub blake3: [u8; 32],
}
