use super::error::FormatError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatHeader {
    pub magic: [u8; 4],
    pub version: u16,
}

impl FormatHeader {
    pub const LAYOUT_SIZE: usize = 6;

    pub fn new(magic: [u8; 4], version: u16) -> Self {
        Self { magic, version }
    }

    /// Total function: parses bytes without panic. Returns error if slice is under 6 bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, FormatError> {
        if bytes.len() < Self::LAYOUT_SIZE {
            return Err(FormatError::Corrupt {
                reason: "Header slice too short for FormatHeader".into(),
            });
        }
        let magic: [u8; 4] = match bytes.get(0..4) {
            Some(s) => match s.try_into() {
                Ok(arr) => arr,
                Err(_) => {
                    return Err(FormatError::Corrupt {
                        reason: "Invalid magic slice".into(),
                    })
                }
            },
            None => {
                return Err(FormatError::Corrupt {
                    reason: "Missing magic slice".into(),
                })
            }
        };

        let version_bytes = match bytes.get(4..6) {
            Some(s) => s,
            None => {
                return Err(FormatError::Corrupt {
                    reason: "Missing version slice".into(),
                })
            }
        };
        let version = u16::from_le_bytes(match version_bytes.try_into() {
            Ok(arr) => arr,
            Err(_) => {
                return Err(FormatError::Corrupt {
                    reason: "Invalid version slice".into(),
                })
            }
        });

        Ok(Self { magic, version })
    }

    pub fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&self.version.to_le_bytes())?;
        Ok(())
    }

    pub fn to_bytes(&self) -> [u8; 6] {
        let mut buf = [0u8; 6];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());
        buf
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_format_header_roundtrip() {
        let header = FormatHeader::new(*b"MFSX", 4);
        let bytes = header.to_bytes();
        let parsed = FormatHeader::parse(&bytes).expect("roundtrip parse");
        assert_eq!(header, parsed);
    }

    #[test]
    fn test_format_header_parse_total_no_panic() {
        assert!(FormatHeader::parse(&[]).is_err());
        assert!(FormatHeader::parse(&[0u8; 5]).is_err());
        let res = FormatHeader::parse(&[0xFF; 100]);
        assert!(res.is_ok());
        let h = res.unwrap();
        assert_eq!(h.magic, [0xFF; 4]);
        assert_eq!(h.version, u16::MAX);
    }
}
