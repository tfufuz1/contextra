use contextra_core::{ContextraError, Result};

pub fn get_entry_at_index(
    block_data: &[u8],
    offsets_start: usize,
    idx: usize,
) -> Result<(usize, usize)> {
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

    Ok((entry_off, k_len))
}

pub fn get_entry_key_at_index<'a>(
    block_data: &'a [u8],
    offsets_start: usize,
    idx: usize,
) -> Result<&'a [u8]> {
    let (entry_off, k_len) = get_entry_at_index(block_data, offsets_start, idx)?;
    let ep = entry_off + 2;
    block_data
        .get(ep..ep + k_len)
        .ok_or_else(|| ContextraError::Storage("malformed block: entry_key".into()))
}

pub fn binary_search_first_index_in_block(
    block_data: &[u8],
    offsets_start: usize,
    num_offsets: usize,
    key: &[u8],
) -> Result<Option<usize>> {
    if num_offsets == 0 {
        return Ok(None);
    }
    let mut low = 0;
    let mut high = num_offsets;

    while low < high {
        let mid = low + (high - low) / 2;
        let entry_key = get_entry_key_at_index(block_data, offsets_start, mid)?;
        if entry_key < key {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    if low < num_offsets {
        let entry_key = get_entry_key_at_index(block_data, offsets_start, low)?;
        if entry_key == key {
            return Ok(Some(low));
        }
    }
    Ok(None)
}

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
        let entry_key = get_entry_key_at_index(block_data, offsets_start, mid)?;

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

pub fn binary_search_entry_in_block(
    block_data: &[u8],
    offsets_start: usize,
    num_offsets: usize,
    key: &[u8],
) -> Result<Option<(usize, usize)>> {
    match binary_search_first_index_in_block(block_data, offsets_start, num_offsets, key)? {
        Some(idx) => {
            let (entry_off, k_len) = get_entry_at_index(block_data, offsets_start, idx)?;
            Ok(Some((entry_off, k_len)))
        }
        None => Ok(None),
    }
}

pub fn block_binary_search(
    block_data: &[u8],
    offsets_start: usize,
    num_offsets: usize,
    key: &[u8],
) -> Result<Option<usize>> {
    match binary_search_first_index_in_block(block_data, offsets_start, num_offsets, key)? {
        Some(idx) => {
            let (entry_off, _) = get_entry_at_index(block_data, offsets_start, idx)?;
            Ok(Some(entry_off))
        }
        None => Ok(None),
    }
}
