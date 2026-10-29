// FILE-CONTEXT
// ZWECK: Sicheres Entpacken und Validieren von Zip-Archiven (Zip-Bomb-Schutz).
// INVARIANTEN: Zero-Panic, typisierte IngestError, kein unsafe, striktes Einhalten von IngestLimits.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::{IngestError, LimitKind};
use crate::limits::IngestLimits;
use std::io::{Read, Seek};

/// Validiert ein Zip-Archiv vor der Verarbeitung gegen `IngestLimits`.
///
/// Prüft Eintragsanzahl, kumulierte unkomprimierte Bytes und Kompressionsverhältnis,
/// um Zip-Bomben und DoS-Angriffe frühzeitig abzulehnen.
pub fn validate_zip_archive<R: Read + Seek>(
    reader: R,
    limits: &IngestLimits,
) -> Result<(), IngestError> {
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| IngestError::Corrupt {
        format: None,
        detail: format!("Ungültiges Zip-Archiv: {e}"),
    })?;

    let num_entries = match u32::try_from(archive.len()) {
        Ok(v) => v,
        Err(_) => u32::MAX,
    };
    if num_entries > limits.max_archive_entries {
        return Err(IngestError::LimitExceeded {
            kind: LimitKind::ArchiveEntries,
            limit: limits.max_archive_entries as u64,
            observed: num_entries as u64,
        });
    }

    let mut total_uncompressed: u64 = 0;
    let mut total_compressed: u64 = 0;

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| IngestError::Corrupt {
            format: None,
            detail: format!("Zip-Eintrag {i} nicht lesbar: {e}"),
        })?;

        total_uncompressed = total_uncompressed.saturating_add(file.size());
        total_compressed = total_compressed.saturating_add(file.compressed_size());

        if total_uncompressed > limits.max_decompressed_bytes {
            return Err(IngestError::LimitExceeded {
                kind: LimitKind::DecompressedBytes,
                limit: limits.max_decompressed_bytes,
                observed: total_uncompressed,
            });
        }
    }

    if total_uncompressed > 0 {
        let compressed_denom = total_compressed.max(1);
        let ratio = total_uncompressed / compressed_denom;
        if ratio > limits.max_compression_ratio as u64 {
            return Err(IngestError::LimitExceeded {
                kind: LimitKind::CompressionRatio,
                limit: limits.max_compression_ratio as u64,
                observed: ratio,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::field_reassign_with_default
)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    #[test]
    fn test_validate_zip_archive_ok() {
        let mut buf = Vec::new();
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut buf));
            let options = FileOptions::default();
            writer.start_file("test.txt", options).unwrap();
            std::io::Write::write_all(&mut writer, b"hello world").unwrap();
            writer.finish().unwrap();
        }

        let limits = IngestLimits::default();
        assert!(validate_zip_archive(Cursor::new(&buf), &limits).is_ok());
    }

    #[test]
    fn test_validate_zip_archive_exceeds_entries() {
        let mut buf = Vec::new();
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut buf));
            let options = FileOptions::default();
            for i in 0..5 {
                writer.start_file(format!("file_{i}.txt"), options).unwrap();
                std::io::Write::write_all(&mut writer, b"x").unwrap();
            }
            writer.finish().unwrap();
        }

        let mut limits = IngestLimits::default();
        limits.max_archive_entries = 3;
        let err = validate_zip_archive(Cursor::new(&buf), &limits).unwrap_err();
        assert!(matches!(
            err,
            IngestError::LimitExceeded {
                kind: LimitKind::ArchiveEntries,
                ..
            }
        ));
    }
}
