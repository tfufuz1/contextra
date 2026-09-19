//! Zero-copy adapter abstractions for IPC wire messages.

use bytes::Bytes;

/// Zero-copy byte buffer slice wrapper for FlatBuffers messages.
#[derive(Debug, Clone)]
pub struct WireBuffer {
    data: Bytes,
}

impl WireBuffer {
    /// Creates a new `WireBuffer` wrapping the provided `Bytes`.
    #[inline]
    pub fn new(data: Bytes) -> Self {
        Self { data }
    }

    /// Returns a slice reference to the underlying byte payload.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Consumes the wrapper and returns the inner `Bytes`.
    #[inline]
    pub fn into_inner(self) -> Bytes {
        self.data
    }
}

impl AsRef<[u8]> for WireBuffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
