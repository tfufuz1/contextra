use super::error::FormatError;
use super::header::FormatHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    Sstable,
    Blob,
    Manifest,
    Wal,
    Ledger,
}

impl FileKind {
    pub fn expected_magic(&self) -> [u8; 4] {
        match self {
            FileKind::Sstable => *b"MFSX",
            FileKind::Blob => *b"MFBL",
            FileKind::Manifest => *b"MFMN",
            FileKind::Wal => *b"MFW3",
            FileKind::Ledger => *b"CTRA",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            FileKind::Sstable => "SSTable",
            FileKind::Blob => "Blob",
            FileKind::Manifest => "Manifest",
            FileKind::Wal => "WAL",
            FileKind::Ledger => "Ledger",
        }
    }
}

pub fn check_openable(
    kind: FileKind,
    header: &FormatHeader,
    max_supported: u16,
) -> Result<(), FormatError> {
    let expected_magic = kind.expected_magic();
    if header.magic != expected_magic {
        return Err(FormatError::Corrupt {
            reason: format!(
                "Invalid magic for {}: expected {:?}, got {:?}",
                kind.name(),
                expected_magic,
                header.magic
            ),
        });
    }

    if header.version > max_supported {
        return Err(FormatError::UnsupportedNewerVersion {
            kind,
            file_version: header.version,
            max_supported,
        });
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_check_openable_valid() {
        let header = FormatHeader::new(*b"MFSX", 4);
        assert!(check_openable(FileKind::Sstable, &header, 4).is_ok());
    }

    #[test]
    fn test_check_openable_invalid_magic() {
        let header = FormatHeader::new(*b"XXXX", 1);
        let err = check_openable(FileKind::Sstable, &header, 4);
        assert!(matches!(err, Err(FormatError::Corrupt { .. })));
    }

    #[test]
    fn test_check_openable_newer_version() {
        let header = FormatHeader::new(*b"MFSX", 5);
        let err = check_openable(FileKind::Sstable, &header, 4);
        assert!(matches!(
            err,
            Err(FormatError::UnsupportedNewerVersion { .. })
        ));
    }
}
