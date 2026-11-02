//! Keyed BLAKE3 tags with domain-separated length-prefixed preimages.

use crate::hierarchy::IdxKey;

/// Tag preimage abstraction ensuring length prefixing per CR-11 / SPEC 9.3.
pub struct TagPreimage<'a> {
    parts: Vec<&'a [u8]>,
}

impl<'a> TagPreimage<'a> {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    pub fn push(&mut self, part: &'a [u8]) -> &mut Self {
        self.parts.push(part);
        self
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for p in &self.parts {
            let len = u32::try_from(p.len()).unwrap_or(u32::MAX);
            out.extend_from_slice(&len.to_be_bytes());
            out.extend_from_slice(p);
        }
        out
    }
}

impl<'a> Default for TagPreimage<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes a 16-byte BLAKE3 keyed tag.
pub fn tag(k: &IdxKey, pre: &TagPreimage) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new_keyed(k.0.as_bytes());
    let bytes = pre.to_bytes();
    hasher.update(&bytes);
    let mut out = [0u8; 16];
    let output = hasher.finalize();
    if let Some(src) = output.as_bytes().get(..16) {
        out.copy_from_slice(src);
    }
    out
}
