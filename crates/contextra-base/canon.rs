//! Canonical, injective encoding primitives for signed structures.
//!
//! Every optional field carries an explicit tag byte, so `None` and
//! `Some(0)` can never produce identical bytes.

/// Append-only canonical writer.
#[derive(Debug, Default)]
pub struct CanonWriter {
    buf: Vec<u8>,
}

impl CanonWriter {
    /// New empty writer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// One raw byte.
    pub fn u8(&mut self, v: u8) -> &mut Self {
        self.buf.push(v);
        self
    }
    /// Big-endian u64.
    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.buf.extend_from_slice(&v.to_be_bytes());
        self
    }
    /// Fixed-size digest (length is implied by the type).
    pub fn fixed<const N: usize>(&mut self, v: &[u8; N]) -> &mut Self {
        self.buf.extend_from_slice(v);
        self
    }
    /// Length-prefixed bytes.
    pub fn bytes(&mut self, v: &[u8]) -> &mut Self {
        self.u64(v.len() as u64);
        self.buf.extend_from_slice(v);
        self
    }
    /// Tagged optional u64: `0x00` for `None`, `0x01 ++ value` for `Some`.
    pub fn opt_u64(&mut self, v: Option<u64>) -> &mut Self {
        match v {
            None => self.u8(0),
            Some(x) => self.u8(1).u64(x),
        }
    }
    /// Finish and take the bytes.
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_and_zero_differ() {
        let mut a = CanonWriter::new();
        a.opt_u64(None);
        let mut b = CanonWriter::new();
        b.opt_u64(Some(0));
        assert_ne!(a.finish(), b.finish());
    }

    #[test]
    fn length_prefix_prevents_concatenation_ambiguity() {
        let mut a = CanonWriter::new();
        a.bytes(b"ab").bytes(b"c");
        let mut b = CanonWriter::new();
        b.bytes(b"a").bytes(b"bc");
        assert_ne!(a.finish(), b.finish());
    }
}
