//! Keyed BLAKE3 tags with domain-separated length-prefixed preimages.

use crate::hierarchy::IdxKey;

/// Tag preimage abstraction ensuring length prefixing per CR-11 / SPEC 9.3.
///
/// Encapsulates byte components for domain-separated keyed BLAKE3 tags.
pub struct TagPreimage<'a> {
    parts: Vec<&'a [u8]>,
}

impl<'a> TagPreimage<'a> {
    /// Constructs a new empty tag preimage.
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Appends a byte part to the preimage.
    pub fn push(&mut self, part: &'a [u8]) -> &mut Self {
        self.parts.push(part);
        self
    }

    /// Serializes parts with length prefixes into a canonical byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for p in &self.parts {
            let len = match u32::try_from(p.len()) {
                Ok(l) => l,
                Err(_) => continue,
            };
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
///
/// # Inputs
/// * `k`: Strongly-typed `IdxKey` derived from the master key.
/// * `pre`: Length-prefixed `TagPreimage`.
///
/// # Properties & Constant-Time
/// Uses BLAKE3 keyed hashing mode. Tag generation executes deterministically and
/// resists length extension and preimage collisions.
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
