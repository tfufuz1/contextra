//! Canonical TLV serialization and deserialization (`CanonicalWriter`, `CanonicalReader`).
//!
//! Tag: u16 LE, Length: u32 LE, fixed-width LE integers.
//! Rejects duplicate tags and unknown critical tags (top bit 0x8000 set).

#![forbid(unsafe_code)]

use contextra_types::{ContextraError, Result};
use std::collections::BTreeSet;

/// Represents a single TLV entry with u16 LE tag and u32 LE length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalTlvEntry {
    /// Tag identifier (u16 LE). Top bit (0x8000) indicates a critical field.
    pub tag: u16,
    /// Value bytes.
    pub value: Vec<u8>,
}

impl CanonicalTlvEntry {
    /// Returns true if tag's critical bit (0x8000) is set.
    pub fn is_critical(&self) -> bool {
        (self.tag & 0x8000) != 0
    }
}

/// Writer for constructing canonical TLV byte buffers.
#[derive(Debug, Default)]
pub struct CanonicalWriter {
    entries: Vec<CanonicalTlvEntry>,
}

impl CanonicalWriter {
    /// Creates a new empty `CanonicalWriter`.
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Appends a raw value for tag (u16 LE).
    pub fn write_bytes(&mut self, tag: u16, value: Vec<u8>) {
        self.entries.push(CanonicalTlvEntry { tag, value });
    }

    /// Appends a u8 value.
    pub fn write_u8(&mut self, tag: u16, val: u8) {
        self.write_bytes(tag, vec![val]);
    }

    /// Appends a u16 LE value.
    pub fn write_u16(&mut self, tag: u16, val: u16) {
        self.write_bytes(tag, val.to_le_bytes().to_vec());
    }

    /// Appends a u32 LE value.
    pub fn write_u32(&mut self, tag: u16, val: u32) {
        self.write_bytes(tag, val.to_le_bytes().to_vec());
    }

    /// Appends a u64 LE value.
    pub fn write_u64(&mut self, tag: u16, val: u64) {
        self.write_bytes(tag, val.to_le_bytes().to_vec());
    }

    /// Appends a bool value (1 byte: 1 for true, 0 for false).
    pub fn write_bool(&mut self, tag: u16, val: bool) {
        self.write_u8(tag, if val { 1 } else { 0 });
    }

    /// Appends a UTF-8 string value.
    pub fn write_string(&mut self, tag: u16, val: &str) {
        self.write_bytes(tag, val.as_bytes().to_vec());
    }

    /// Appends a 32-byte fixed array value.
    pub fn write_bytes32(&mut self, tag: u16, val: &[u8; 32]) {
        self.write_bytes(tag, val.to_vec());
    }

    /// Sorts entries by tag (u16) deterministically and encodes to byte vector.
    ///
    /// Checks for duplicate tags during finishing and returns error if duplicates are found.
    pub fn finish(mut self) -> Result<Vec<u8>> {
        self.entries.sort_by_key(|e| e.tag);

        let mut seen_tags = BTreeSet::new();
        let mut out = Vec::new();

        for entry in self.entries {
            if !seen_tags.insert(entry.tag) {
                return Err(ContextraError::InvalidInput(format!(
                    "CanonicalWriter: duplicate tag {:#06x}",
                    entry.tag
                )));
            }

            let len_u32 = u32::try_from(entry.value.len()).map_err(|_| {
                ContextraError::InvalidInput("CanonicalWriter: value length overflow u32::MAX".into())
            })?;

            out.extend_from_slice(&entry.tag.to_le_bytes());
            out.extend_from_slice(&len_u32.to_le_bytes());
            out.extend_from_slice(&entry.value);
        }

        Ok(out)
    }
}

/// Reader for decoding canonical TLV byte buffers.
#[derive(Debug)]
pub struct CanonicalReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> CanonicalReader<'a> {
    /// Creates a new `CanonicalReader` for the given byte slice.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Reads all TLV entries, enforcing strict canonical ordering, no duplicates,
    /// and rejecting unknown critical tags if provided in `known_critical_tags`.
    pub fn read_all(
        &mut self,
        known_critical_tags: &[u16],
    ) -> Result<Vec<CanonicalTlvEntry>> {
        let mut entries = Vec::new();
        let mut seen_tags = BTreeSet::new();
        let mut last_tag: Option<u16> = None;

        while self.offset < self.data.len() {
            let remaining = self.data.len() - self.offset;
            if remaining < 6 {
                return Err(ContextraError::InvalidInput(
                    "CanonicalReader: truncated TLV header".into(),
                ));
            }

            let mut tag_bytes = [0u8; 2];
            tag_bytes.copy_from_slice(&self.data[self.offset..self.offset + 2]);
            let tag = u16::from_le_bytes(tag_bytes);

            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&self.data[self.offset + 2..self.offset + 6]);
            let len = usize::try_from(u32::from_le_bytes(len_bytes))
                .map_err(|_| ContextraError::InvalidInput("CanonicalReader: u32 length overflow".into()))?;

            self.offset += 6;

            if self.data.len() - self.offset < len {
                return Err(ContextraError::InvalidInput(
                    "CanonicalReader: truncated TLV value payload".into(),
                ));
            }

            let value = self.data[self.offset..self.offset + len].to_vec();
            self.offset += len;

            // Enforcement 1: Strict tag ordering (ascending)
            if let Some(prev) = last_tag {
                if tag <= prev {
                    return Err(ContextraError::InvalidInput(format!(
                        "CanonicalReader: non-canonical tag order or duplicate tag {:#06x} <= {:#06x}",
                        tag, prev
                    )));
                }
            }
            last_tag = Some(tag);

            // Enforcement 2: Reject duplicate tags
            if !seen_tags.insert(tag) {
                return Err(ContextraError::InvalidInput(format!(
                    "CanonicalReader: duplicate tag {:#06x}",
                    tag
                )));
            }

            let entry = CanonicalTlvEntry { tag, value };

            // Enforcement 3: Reject unknown critical tags (top bit set 0x8000)
            if entry.is_critical() && !known_critical_tags.contains(&tag) {
                return Err(ContextraError::InvalidInput(format!(
                    "CanonicalReader: unknown critical tag {:#06x}",
                    tag
                )));
            }

            entries.push(entry);
        }

        Ok(entries)
    }
}
