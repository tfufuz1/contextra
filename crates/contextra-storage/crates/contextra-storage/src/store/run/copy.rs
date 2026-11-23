//! Raw section relocation without decoding or authenticating sealed payloads.

use super::builder::RunFileBuilder;
use super::reader::RunReader;
use contextra_storage_ports::{ClusterId, Error};

/// Format-specific scanner that checks only integrity fields available
/// without a key (for example CRCs), never an AEAD/MAC authentication tag.
pub trait RawSectionChecksumVerifier {
    fn verify_raw_checksums(&self, section: &[u8]) -> Result<(), Error>;
}

/// Copies selected complete cluster sections byte-for-byte. The verifier is
/// required so callers cannot accidentally treat an unchecked relocation as
/// a scrubbed copy. Any verifier failure aborts before that section is added.
pub fn copy_sections(
    src: &RunReader,
    keep: &[ClusterId],
    dst: &mut RunFileBuilder,
    max_section_bytes: usize,
    verifier: &dyn RawSectionChecksumVerifier,
) -> Result<(), Error> {
    for pair in keep.windows(2) {
        let left = pair.first().ok_or(Error::Limit { limit: "rawcopy cluster list" })?;
        let right = pair.get(1).ok_or(Error::Limit { limit: "rawcopy cluster list" })?;
        if left >= right { return Err(Error::Limit { limit: "rawcopy clusters must be unique and ascending" }); }
    }
    for cluster in keep {
        let metadata = src.section_meta(*cluster).ok_or(Error::UnknownCluster)?;
        let section = src.read_section(*cluster, max_section_bytes)?.ok_or(Error::UnknownCluster)?;
        verifier.verify_raw_checksums(&section)?;
        dst.append_section(*cluster, &section, metadata.block_count, metadata.entry_count,
            metadata.min_stamp, metadata.max_stamp)?;
    }
    Ok(())
}
