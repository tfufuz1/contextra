use contextra_core::{ContextraError, Result};

/// Entry offset accessor supporting both 16-bit (legacy v0..v2) and 32-bit (v3+) block formats.
pub fn get_entry_at_index(
    block_data: &[u8],
    offsets_start: usize,
    idx: usize,
    is_v3: bool,
) -> Result<(usize, usize)> {
    let entry_off = if is_v3 {
        let off_pos = offsets_start
            .checked_add(idx.checked_mul(4).ok_or_else(|| {
                ContextraError::Storage("overflow calculating offset position".into())
            })?)
            .ok_or_else(|| {
                ContextraError::Storage("overflow calculating offset position".into())
            })?;

        let end_pos = off_pos.checked_add(4).ok_or_else(|| {
            ContextraError::Storage("overflow calculating end offset position".into())
        })?;

        let slice = block_data.get(off_pos..end_pos).ok_or_else(|| {
            ContextraError::Storage("malformed block: off_pos out of bounds".into())
        })?;

        usize::try_from(u32::from_le_bytes(
            slice
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
        ))
        .map_err(|_| ContextraError::Storage("entry offset exceeds platform usize".into()))?
    } else {
        let off_pos = offsets_start
            .checked_add(idx.checked_mul(2).ok_or_else(|| {
                ContextraError::Storage("overflow calculating offset position".into())
            })?)
            .ok_or_else(|| {
                ContextraError::Storage("overflow calculating offset position".into())
            })?;

        let end_pos = off_pos.checked_add(2).ok_or_else(|| {
            ContextraError::Storage("overflow calculating end offset position".into())
        })?;

        let slice = block_data.get(off_pos..end_pos).ok_or_else(|| {
            ContextraError::Storage("malformed block: off_pos out of bounds".into())
        })?;

        usize::from(u16::from_le_bytes(
            slice
                .try_into()
                .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
        ))
    };

    let k_len_end = entry_off.checked_add(2).ok_or_else(|| {
        ContextraError::Storage("overflow calculating key length end position".into())
    })?;

    let k_len_slice = block_data
        .get(entry_off..k_len_end)
        .ok_or_else(|| ContextraError::Storage("malformed block: k_len out of bounds".into()))?;

    let k_len = usize::from(u16::from_le_bytes(
        k_len_slice
            .try_into()
            .map_err(|_| ContextraError::Storage("invalid slice".into()))?,
    ));

    Ok((entry_off, k_len))
}

pub fn get_entry_key_at_index(
    block_data: &[u8],
    offsets_start: usize,
    idx: usize,
    is_v3: bool,
) -> Result<&[u8]> {
    let (entry_off, k_len) = get_entry_at_index(block_data, offsets_start, idx, is_v3)?;
    let ep = entry_off
        .checked_add(2)
        .ok_or_else(|| ContextraError::Storage("overflow calculating entry key position".into()))?;
    block_data
        .get(
            ep..ep.checked_add(k_len).ok_or_else(|| {
                ContextraError::Storage("overflow calculating entry key length".into())
            })?,
        )
        .ok_or_else(|| ContextraError::Storage("malformed block: entry_key out of bounds".into()))
}

// TODO(Implementer): [P02 / F-02 / HIGH / JULES-P02-02]
// Binärsuche in SSTable-Blöcken: Erste/neueste Version bei Mehrfachversionen garantieren (Invariante I-1):
// Bei mehreren Versionen desselben Keys in einem Datenblock MUSS die Binärsuche deterministisch
// den ersten Index (die neueste Version mit höchster Sequenznummer, da im Block absteigend sortiert)
// zurückgeben. Sicherstellen, dass die Partition-Point-Logik (Left-Rewind) für alle Block-Suchfunktionen
// (`binary_search_first_index_in_block`, `binary_search_index_in_block`, `binary_search_entry_in_block`)
// ausnahmslos zum ersten Vorkommen führt, um Stale Reads zu verhindern.
pub fn binary_search_first_index_in_block(
    block_data: &[u8],
    offsets_start: usize,
    num_offsets: usize,
    key: &[u8],
    is_v3: bool,
) -> Result<Option<usize>> {
    if num_offsets == 0 {
        return Ok(None);
    }
    let mut low = 0;
    let mut high = num_offsets;

    while low < high {
        let mid = low + (high - low) / 2;
        let entry_key = get_entry_key_at_index(block_data, offsets_start, mid, is_v3)?;
        if entry_key < key {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    if low < num_offsets {
        let entry_key = get_entry_key_at_index(block_data, offsets_start, low, is_v3)?;
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
    is_v3: bool,
) -> Result<std::result::Result<usize, usize>> {
    if num_offsets == 0 {
        return Ok(Err(0));
    }

    let mut low = 0;
    let mut high = num_offsets;

    while low < high {
        let mid = low + (high - low) / 2;
        let entry_key = get_entry_key_at_index(block_data, offsets_start, mid, is_v3)?;

        match entry_key.cmp(key) {
            std::cmp::Ordering::Less => {
                low = mid + 1;
            }
            std::cmp::Ordering::Greater => {
                high = mid;
            }
            std::cmp::Ordering::Equal => {
                let mut first = mid;
                while first > 0 {
                    let prev_key =
                        get_entry_key_at_index(block_data, offsets_start, first - 1, is_v3)?;
                    if prev_key == key {
                        first -= 1;
                    } else {
                        break;
                    }
                }
                return Ok(Ok(first));
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
    is_v3: bool,
) -> Result<Option<(usize, usize)>> {
    match binary_search_first_index_in_block(block_data, offsets_start, num_offsets, key, is_v3)? {
        Some(idx) => {
            let (entry_off, k_len) = get_entry_at_index(block_data, offsets_start, idx, is_v3)?;
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
    is_v3: bool,
) -> Result<Option<usize>> {
    match binary_search_first_index_in_block(block_data, offsets_start, num_offsets, key, is_v3)? {
        Some(idx) => {
            let (entry_off, _) = get_entry_at_index(block_data, offsets_start, idx, is_v3)?;
            Ok(Some(entry_off))
        }
        None => Ok(None),
    }
}
