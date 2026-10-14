//! Facade wiring entry point for contextra-wire (Ring 0) per Rule 7.

use crate::contextra_generated::contextra::ipc::{
    search_response_buffer_has_identifier, search_response_size_prefixed_buffer_has_identifier,
    SEARCH_RESPONSE_IDENTIFIER,
};
use std::io;

/// Current wire format protocol version.
pub const WIRE_PROTOCOL_VERSION: u16 = 1;

/// Wiring container for IPC wire adapters and FlatBuffers serialization.
#[derive(Debug, Clone, Default)]
pub struct WireWiring;

impl WireWiring {
    /// Builds the `WireWiring` instance.
    pub fn build(_deps: ()) -> Result<Self, io::Error> {
        Ok(Self)
    }

    /// Returns the active wire protocol version.
    #[inline]
    pub fn wire_version(&self) -> u16 {
        WIRE_PROTOCOL_VERSION
    }

    /// Returns the schema identifier string ("CTRA").
    #[inline]
    pub fn schema_identifier(&self) -> &'static str {
        SEARCH_RESPONSE_IDENTIFIER
    }

    /// Validates whether the given buffer has the expected FlatBuffer schema identifier ("CTRA").
    #[inline]
    pub fn validate_buffer_identifier(&self, buf: &[u8]) -> bool {
        search_response_buffer_has_identifier(buf)
    }

    /// Validates whether the given size-prefixed buffer has the expected FlatBuffer schema identifier ("CTRA").
    #[inline]
    pub fn validate_size_prefixed_buffer_identifier(&self, buf: &[u8]) -> bool {
        search_response_size_prefixed_buffer_has_identifier(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contextra_generated::contextra::ipc::{
        SearchResponse, SearchResponseArgs, SEARCH_RESPONSE_IDENTIFIER,
    };
    use flatbuffers::FlatBufferBuilder;

    #[test]
    fn test_wire_wiring_build_and_validation() {
        let wiring = WireWiring::build(()).expect("WireWiring build failed");
        assert_eq!(wiring.wire_version(), 1);
        assert_eq!(wiring.schema_identifier(), SEARCH_RESPONSE_IDENTIFIER);

        let mut fbb = FlatBufferBuilder::new();
        let sr = SearchResponse::create(&mut fbb, &SearchResponseArgs::default());
        SearchResponse::finish_buffer(&mut fbb, sr);
        let finished_buf = fbb.finished_data();

        assert!(wiring.validate_buffer_identifier(finished_buf));
        assert!(!wiring.validate_buffer_identifier(b"INVALID_HEADER_BUF"));

        let mut fbb_sp = FlatBufferBuilder::new();
        let sr_sp = SearchResponse::create(&mut fbb_sp, &SearchResponseArgs::default());
        SearchResponse::finish_size_prefixed_buffer(&mut fbb_sp, sr_sp);
        let finished_sp_buf = fbb_sp.finished_data();

        assert!(wiring.validate_size_prefixed_buffer_identifier(finished_sp_buf));
    }
}
