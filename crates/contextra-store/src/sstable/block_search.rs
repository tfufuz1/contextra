use contextra_core::{ContextraError, Result};

pub fn binary_search_index_in_block(
    block_data: &[u8],
    offsets_start: usize,
    num_offsets: usize,
    key: &[u8],
) -> Result<std::result::Result<usize, usize>> {
    if num_offsets == 0 {
        return Ok(Err(0));
    }

    let mut low = 0;
    let mut high = num_offsets;

    while low < high {
        let mid = low + (high - low) / 2;
        let off_pos = offsets_start + mid * 2;
        let entry_off = u16::from_le_bytes(
            block_data
                .get(off_pos..off_pos + 2)
                .ok_or_else(|| ContextraError::Storage("malformed block: off_pos".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
        ) as usize;

        let k_len = u16::from_le_bytes(
            block_data
                .get(entry_off..entry_off + 2)
                .ok_or_else(|| ContextraError::Storage("malformed block: k_len".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
        ) as usize;

        let ep = entry_off + 2;
        let entry_key = block_data
            .get(ep..ep + k_len)
            .ok_or_else(|| ContextraError::Storage("malformed block: entry_key".into()))?;

        match entry_key.cmp(key) {
            std::cmp::Ordering::Less => {
                low = mid + 1;
            }
            std::cmp::Ordering::Greater => {
                high = mid;
            }
            std::cmp::Ordering::Equal => {
                return Ok(Ok(mid));
            }
        }
    }

    Ok(Err(low))
}

/// Binary search for a key inside a SSTable data block.
///
/// Returns `Ok(Some((entry_offset, key_len)))` if the key is found,
/// or `Ok(None)` if the key does not exist in the block.
/// Exposed as `pub` for fuzz testing in `contextra-store-fuzz`.
pub fn binary_search_entry_in_block(
    block_data: &[u8],
    offsets_start: usize,
    num_offsets: usize,
    key: &[u8],
) -> Result<Option<(usize, usize)>> {
    match binary_search_index_in_block(block_data, offsets_start, num_offsets, key)? {
        Ok(idx) => {
            let off_pos = offsets_start + idx * 2;
            let entry_off = u16::from_le_bytes(
                block_data
                    .get(off_pos..off_pos + 2)
                    .ok_or_else(|| ContextraError::Storage("malformed block: off_pos".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
            ) as usize;
            let k_len = u16::from_le_bytes(
                block_data
                    .get(entry_off..entry_off + 2)
                    .ok_or_else(|| ContextraError::Storage("malformed block: k_len".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
            ) as usize;
            Ok(Some((entry_off, k_len)))
        }
        Err(_) => Ok(None),
    }
}

/// # Preconditions
/// - The block entries and the offsets array MUST be sorted lexicographically by key.
///   This invariant is guaranteed by `BlockBuilder::build()` and SSTable file header contract.
///
/// Binary search over the sorted offsets array within a decoded block.
/// Returns `Ok(Some(entry_offset))` if found, `Ok(None)` if not present.
/// `block_data`: the raw (decrypted, CRC-stripped) block bytes.
/// `offsets_start`: byte position of the first 2-byte offset entry.
/// `num_offsets`: number of entries in the offsets array.
/// `key`: the key to search for.
/// Exposed as `pub` for fuzz testing in `contextra-store-fuzz`.
pub fn block_binary_search(
    block_data: &[u8],
    offsets_start: usize,
    num_offsets: usize,
    key: &[u8],
) -> Result<Option<usize>> {
    if num_offsets == 0 {
        return Ok(None);
    }
    let mut lo = 0usize;
    let mut hi = num_offsets;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let off_pos = offsets_start + mid * 2;
        let entry_off = u16::from_le_bytes(
            block_data
                .get(off_pos..off_pos + 2)
                .ok_or_else(|| ContextraError::Storage("malformed block: off_pos in bsearch".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid slice in bsearch".into()))?,
        ) as usize;
        let k_len = u16::from_le_bytes(
            block_data
                .get(entry_off..entry_off + 2)
                .ok_or_else(|| ContextraError::Storage("malformed block: k_len in bsearch".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid k_len slice in bsearch".into()))?,
        ) as usize;
        let entry_key = block_data
            .get(entry_off + 2..entry_off + 2 + k_len)
            .ok_or_else(|| ContextraError::Storage("malformed block: entry_key in bsearch".into()))?;
        match entry_key.cmp(key) {
            std::cmp::Ordering::Less => lo = mid + 1,
            std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => hi = mid,
        }
    }
    if lo < num_offsets {
        let off_pos = offsets_start + lo * 2;
        let entry_off = u16::from_le_bytes(
            block_data
                .get(off_pos..off_pos + 2)
                .ok_or_else(|| ContextraError::Storage("malformed block: off_pos in bsearch".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid slice in bsearch".into()))?,
        ) as usize;
        let k_len = u16::from_le_bytes(
            block_data
                .get(entry_off..entry_off + 2)
                .ok_or_else(|| ContextraError::Storage("malformed block: k_len in bsearch".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid k_len slice in bsearch".into()))?,
        ) as usize;
        let entry_key = block_data
            .get(entry_off + 2..entry_off + 2 + k_len)
            .ok_or_else(|| ContextraError::Storage("malformed block: entry_key in bsearch".into()))?;
        if entry_key == key {
            return Ok(Some(entry_off));
        }
    }
    Ok(None)
}
