use super::MAX_MANIFEST_ENTRY_SIZE;

/// Evaluates whether a truncated tail frame is a plausible tail truncation resulting from power loss,
/// rather than a corrupted length header in the middle of a file.
///
/// `claimed_len` is the 4-byte frame length stored in the header (`crc` + `payload`).
/// `partial` is whatever payload bytes (including the initial 4-byte CRC) were read up to EOF.
#[deny(clippy::indexing_slicing)]
pub(super) fn is_valid_tail_truncation_candidate(claimed_len: usize, partial: &[u8]) -> bool {
    // Minimum possible frame size for any ManifestEntry is 10 bytes (Remove with 1-char path)
    if claimed_len < 10 || claimed_len > MAX_MANIFEST_ENTRY_SIZE as usize {
        return false;
    }

    if partial.len() >= 5 {
        let op_tag = match partial.get(4) {
            Some(&tag) => tag,
            None => return false,
        };
        let remaining = match partial.get(5..) {
            Some(rem) => rem,
            None => return false,
        };

        match op_tag {
            0 => {
                // Add: min size 18 bytes
                if claimed_len < 18 {
                    return false;
                }
                if remaining.len() >= 12 {
                    if let Some(slice) = remaining.get(8..12) {
                        if let Ok(bytes) = slice.try_into() {
                            let path_len = u32::from_le_bytes(bytes) as usize;
                            let expected_total_len = 4 + 1 + 8 + 4 + path_len;
                            if claimed_len != expected_total_len {
                                return false;
                            }
                        }
                    }
                }
            }
            1 => {
                // Remove: min size 10 bytes
                if claimed_len < 10 {
                    return false;
                }
                if remaining.len() >= 4 {
                    if let Some(slice) = remaining.get(0..4) {
                        if let Ok(bytes) = slice.try_into() {
                            let path_len = u32::from_le_bytes(bytes) as usize;
                            let expected_total_len = 4 + 1 + 4 + path_len;
                            if claimed_len != expected_total_len {
                                return false;
                            }
                        }
                    }
                }
            }
            2 => {
                // RollbackComplete: exact size 13 bytes
                if claimed_len != 13 {
                    return false;
                }
            }
            3 => {
                // Replace: min size 30 bytes
                if claimed_len < 30 {
                    return false;
                }
                if remaining.len() >= 20 {
                    if let Some(added_slice) = remaining.get(16..20) {
                        if let Ok(added_bytes) = added_slice.try_into() {
                            let added_path_len = u32::from_le_bytes(added_bytes) as usize;
                            if remaining.len() >= 20 + added_path_len + 4 {
                                let offset = 20 + added_path_len;
                                if let Some(cnt_slice) = remaining.get(offset..offset + 4) {
                                    if let Ok(cnt_bytes) = cnt_slice.try_into() {
                                        let removed_count = u32::from_le_bytes(cnt_bytes) as usize;

                                        let mut rem_offset = offset + 4;
                                        let mut calculated_len =
                                            4 + 1 + 8 + 8 + 4 + added_path_len + 4;
                                        let mut all_removed_parsed = true;
                                        for _ in 0..removed_count {
                                            if remaining.len() >= rem_offset + 4 {
                                                if let Some(r_slice) =
                                                    remaining.get(rem_offset..rem_offset + 4)
                                                {
                                                    if let Ok(r_bytes) = r_slice.try_into() {
                                                        let r_len =
                                                            u32::from_le_bytes(r_bytes) as usize;
                                                        rem_offset += 4 + r_len;
                                                        calculated_len += 4 + r_len;
                                                    } else {
                                                        all_removed_parsed = false;
                                                        break;
                                                    }
                                                } else {
                                                    all_removed_parsed = false;
                                                    break;
                                                }
                                            } else {
                                                all_removed_parsed = false;
                                                break;
                                            }
                                        }
                                        if all_removed_parsed && claimed_len != calculated_len {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                // Unknown op_tag is definitely corruption
                return false;
            }
        }
    }

    true
}
