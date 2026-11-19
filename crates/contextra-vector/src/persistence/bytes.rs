// FILE-CONTEXT
// ZWECK: Read-only Dateiinhalt im Speicher; ersetzt das frühere mmap-Mapping ohne unsafe.
// INVARIANTEN: Zero-Unsafe (#![forbid(unsafe_code)]); Inhalt ist nach dem Laden unveränderlich.
// NICHT-OFFENSICHTLICH: Die Datei wird vollständig gelesen (kein Zero-Copy). Speicherbedarf = Dateigröße.
// HOTSPOTS: FileBytes::from_file
// STAND: 2026-11-19

//! Unveränderlicher, vollständig gelesener Dateiinhalt.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::ops::Deref;

use crate::error::{Result, VectorError};

// ANCHOR[contextra-vector:PERSIST-FILE-BYTES]
/// Unveränderlicher Dateiinhalt, per `Deref` als Byte-Slice zugänglich.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileBytes(Vec<u8>);

impl FileBytes {
    /// Liest die gesamte Datei ab Offset 0 in den Speicher.
    ///
    /// Die übergebene Datei wird nicht verändert (Lesezeiger wird auf einem Klon bewegt).
    pub fn from_file(file: &File) -> Result<Self> {
        let mut handle = file
            .try_clone()
            .map_err(|e| VectorError::Io(e.to_string()))?;
        handle
            .seek(SeekFrom::Start(0))
            .map_err(|e| VectorError::Io(e.to_string()))?;
        let mut buf = Vec::new();
        handle
            .read_to_end(&mut buf)
            .map_err(|e| VectorError::Io(e.to_string()))?;
        Ok(Self(buf))
    }
}

impl Deref for FileBytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}
