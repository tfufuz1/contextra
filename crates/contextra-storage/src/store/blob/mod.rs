//! # Blob-Modul
//!
//! SPEC §9.6: Blob-Dateien für ausgelagerte Wertkörper, Records, Holes,
//! Generationstausch und O(1) Adressierung über BlobRef.

pub mod blob_ref;
pub mod reader;
pub mod record;
pub mod sweep;
pub mod writer;

pub use blob_ref::{BlobFooter, BlobRef, BLOB_FOOTER_LEN, BLOB_REF_LEN};
pub use reader::{BlobReadError, BlobReader};
pub use record::{
    BlobHeader, BlobRecord, BLOB_HEADER_LEN, BLOB_MAGIC, RECORD_KIND_BODY, RECORD_KIND_HOLE,
    RecordError,
};
pub use sweep::{sweep_blob, BlobSweepError};
pub use writer::{BlobWriteError, BlobWriter};
