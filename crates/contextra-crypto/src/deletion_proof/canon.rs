//! Deterministic TLV (Type-Length-Value) canonical encoding for deletion proof v4 (ADR-13).

use crate::error::{CryptoError, Result};

/// TLV element representing tag (u16 BE), length (u32 BE), and payload value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlvField {
    /// Tag identifier. Top bit (0x8000) indicates a critical field.
    pub tag: u16,
    /// Field value payload.
    pub value: Vec<u8>,
}

impl TlvField {
    /// Creates a new TLV field.
    pub fn new(tag: u16, value: Vec<u8>) -> Self {
        Self { tag, value }
    }

    /// Returns whether this tag is critical (top bit 0x8000 set).
    pub fn is_critical(&self) -> bool {
        (self.tag & 0x8000) != 0
    }

    /// Encodes this field to byte stream: tag(u16 BE) | len(u32 BE) | value.
    pub fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.tag.to_be_bytes());
        let len = u32::try_from(self.value.len()).unwrap_or(u32::MAX);
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&self.value);
    }
}

/// Helper methods for building canonical TLV buffers with deterministic field ordering.
#[derive(Default)]
pub struct TlvEncoder {
    fields: Vec<TlvField>,
}

impl TlvEncoder {
    /// Creates a new TLV encoder.
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Adds a raw field value for a given tag.
    pub fn add_field(&mut self, tag: u16, value: Vec<u8>) {
        self.fields.push(TlvField::new(tag, value));
    }

    /// Adds a u8 value.
    pub fn add_u8(&mut self, tag: u16, val: u8) {
        self.add_field(tag, vec![val]);
    }

    /// Adds a u16 BE value.
    pub fn add_u16(&mut self, tag: u16, val: u16) {
        self.add_field(tag, val.to_be_bytes().to_vec());
    }

    /// Adds a u32 BE value.
    pub fn add_u32(&mut self, tag: u16, val: u32) {
        self.add_field(tag, val.to_be_bytes().to_vec());
    }

    /// Adds a u64 BE value.
    pub fn add_u64(&mut self, tag: u16, val: u64) {
        self.add_field(tag, val.to_be_bytes().to_vec());
    }

    /// Adds a bool value (1 byte: 0x01 or 0x00).
    pub fn add_bool(&mut self, tag: u16, val: bool) {
        self.add_u8(tag, if val { 1 } else { 0 });
    }

    /// Adds a String value (UTF-8 bytes).
    pub fn add_string(&mut self, tag: u16, val: &str) {
        self.add_field(tag, val.as_bytes().to_vec());
    }

    /// Adds a 32-byte hash/array value.
    pub fn add_bytes32(&mut self, tag: u16, val: &[u8; 32]) {
        self.add_field(tag, val.to_vec());
    }

    /// Adds an Option<[u8; 32]> value. Explicitly encoded as empty vector if None.
    pub fn add_option_bytes32(&mut self, tag: u16, val: Option<&[u8; 32]>) {
        match val {
            Some(b) => self.add_bytes32(tag, b),
            None => self.add_field(tag, Vec::new()),
        }
    }

    /// Encodes all fields in sorted tag order for strict canonical determinism.
    pub fn finish(mut self) -> Vec<u8> {
        self.fields.sort_by_key(|f| f.tag);
        let mut out = Vec::new();
        for field in &self.fields {
            field.encode(&mut out);
        }
        out
    }
}

/// TLV Decoder for streaming or parsing TLV byte slices.
pub struct TlvDecoder<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> TlvDecoder<'a> {
    /// Creates a new decoder for the input byte slice.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Parses all fields from the byte slice into a list of TlvFields.
    pub fn decode_all(&mut self) -> Result<Vec<TlvField>> {
        let mut fields = Vec::new();
        while self.offset < self.data.len() {
            if self.data.len() - self.offset < 6 {
                return Err(CryptoError::Crypto("Truncated TLV header".to_string()));
            }

            let mut tag_bytes = [0u8; 2];
            tag_bytes.copy_from_slice(&self.data[self.offset..self.offset + 2]);
            let tag = u16::from_be_bytes(tag_bytes);

            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&self.data[self.offset + 2..self.offset + 6]);
            let len = usize::try_from(u32::from_be_bytes(len_bytes))
                .map_err(|_| CryptoError::Crypto("TLV length overflow".to_string()))?;

            self.offset += 6;

            if self.data.len() - self.offset < len {
                return Err(CryptoError::Crypto("Truncated TLV payload".to_string()));
            }

            let value = self.data[self.offset..self.offset + len].to_vec();
            self.offset += len;

            fields.push(TlvField::new(tag, value));
        }
        Ok(fields)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_tlv_encoder_sorting_and_decoder_roundtrip() {
        let mut encoder = TlvEncoder::new();
        encoder.add_u32(0x0002, 42);
        encoder.add_u16(0x0001, 7);
        encoder.add_string(0x0003, "contextra");

        let encoded = encoder.finish();

        let mut decoder = TlvDecoder::new(&encoded);
        let fields = decoder.decode_all().expect("decode failed");

        assert_eq!(fields.len(), 3);
        // Assert sorted order
        assert_eq!(fields[0].tag, 0x0001);
        assert_eq!(fields[1].tag, 0x0002);
        assert_eq!(fields[2].tag, 0x0003);

        assert_eq!(fields[0].value, 7u16.to_be_bytes().to_vec());
        assert_eq!(fields[1].value, 42u32.to_be_bytes().to_vec());
        assert_eq!(fields[2].value, b"contextra".to_vec());
    }

    #[test]
    fn test_critical_tag_bit_detection() {
        let normal_field = TlvField::new(0x0010, vec![]);
        let critical_field = TlvField::new(0x8010, vec![]);

        assert!(!normal_field.is_critical());
        assert!(critical_field.is_critical());
    }
}
