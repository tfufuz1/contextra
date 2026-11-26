//! MAC-protected handles (SPEC-02 §2.6; IMPL-01 §3.1).
//!
//! Textform: `c:<gid>-<mac>`
//! - `<gid>`: Crockford base32 of the `Gid`, lowercase, at least 4 characters.
//! - `<mac>`: exactly 2 Crockford characters that encode a 10-bit MAC.
//!
//! This module only handles bytes. Computing the MAC needs the repository
//! seed and lives in `contextra-crypto`. Verifying a claimed MAC is also done
//! there. This type performs no cryptography.

use crate::ids2::Gid;
use core::fmt;

const PREFIX: &str = "c:";
const SEPARATOR: char = '-';
/// Minimum number of base32 characters for the gid part.
pub const HANDLE_MIN_GID_CHARS: usize = 4;
/// Number of bits in the MAC.
pub const HANDLE_MAC_BITS: u32 = 10;
const MAC_MASK: u16 = (1 << HANDLE_MAC_BITS) - 1;
const MAX_GID_CHARS: usize = 13;

/// Crockford base32 alphabet, lowercase. Excludes `i`, `l`, `o`, `u`.
const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

/// A handle for a `Gid` with a 10-bit MAC.
///
/// `mac` holds the 10-bit value big-endian in the low bits. The top 6 bits of
/// `mac[0]` are always zero. Use [`Handle::new`] to make sure of that.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Handle {
    /// Addressed global identifier.
    pub gid: Gid,
    /// 10-bit MAC, big-endian in the low bits.
    pub mac: [u8; 2],
}

/// Parse result. The MAC is only a *claim* until it has been verified.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ParsedHandle {
    /// Decoded global identifier. Never [`Gid::NONE`].
    pub gid: Gid,
    /// MAC as written in the handle. Unverified.
    pub mac_claimed: [u8; 2],
}

impl ParsedHandle {
    /// The claimed 10-bit MAC value.
    #[inline]
    pub fn mac_value(&self) -> u16 {
        u16::from_be_bytes(self.mac_claimed) & MAC_MASK
    }
}

/// Errors from [`Handle::parse`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HandleParseError {
    /// Missing `c:` prefix.
    MissingPrefix,
    /// Missing `-` separator before the MAC.
    MissingSeparator,
    /// Gid part shorter than [`HANDLE_MIN_GID_CHARS`] or longer than 13 characters.
    GidLength(usize),
    /// Gid value does not fit in 64 bits.
    GidOverflow,
    /// Gid value is the reserved sentinel `0`.
    GidZero,
    /// MAC part is not exactly two characters.
    MacLength(usize),
    /// A character outside the Crockford alphabet.
    InvalidCharacter(char),
}

impl fmt::Display for HandleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPrefix => f.write_str("handle must start with \"c:\""),
            Self::MissingSeparator => f.write_str("handle must contain '-' before the MAC"),
            Self::GidLength(n) => write!(f, "gid part has {n} characters, expected 4 to 13"),
            Self::GidOverflow => f.write_str("gid exceeds 64 bits"),
            Self::GidZero => f.write_str("gid 0 is reserved"),
            Self::MacLength(n) => write!(f, "MAC part has {n} characters, expected 2"),
            Self::InvalidCharacter(c) => write!(f, "character {c:?} is not in the handle alphabet"),
        }
    }
}

impl std::error::Error for HandleParseError {}

impl Handle {
    /// Builds a handle from a gid and a 10-bit MAC value. Higher bits are discarded.
    #[inline]
    pub fn new(gid: Gid, mac10: u16) -> Self {
        Self {
            gid,
            mac: (mac10 & MAC_MASK).to_be_bytes(),
        }
    }

    /// The 10-bit MAC value.
    #[inline]
    pub fn mac_value(&self) -> u16 {
        u16::from_be_bytes(self.mac) & MAC_MASK
    }

    /// Encodes the handle in its textform, e.g. `c:k7qd-3f`.
    pub fn encode(&self) -> String {
        let mut out = String::with_capacity(PREFIX.len() + MAX_GID_CHARS + 3);
        out.push_str(PREFIX);
        encode_gid(self.gid.get(), &mut out);
        out.push(SEPARATOR);
        let mac = self.mac_value();
        out.push(char::from(ALPHABET[usize::from(mac >> 5)]));
        out.push(char::from(ALPHABET[usize::from(mac & 31)]));
        out
    }

    /// Parses a textform handle. Accepts upper and lower case, and the Crockford
    /// aliases `o`→`0` and `i`/`l`→`1`. Does **not** verify the MAC.
    pub fn parse(s: &str) -> Result<ParsedHandle, HandleParseError> {
        let body = strip_prefix_ci(s).ok_or(HandleParseError::MissingPrefix)?;
        let (gid_part, mac_part) = body
            .rsplit_once(SEPARATOR)
            .ok_or(HandleParseError::MissingSeparator)?;

        let gid_chars = gid_part.len();
        if !(HANDLE_MIN_GID_CHARS..=MAX_GID_CHARS).contains(&gid_chars) {
            return Err(HandleParseError::GidLength(gid_chars));
        }
        let mut value: u64 = 0;
        for c in gid_part.chars() {
            let digit = decode_char(c)?;
            value = value
                .checked_mul(32)
                .and_then(|v| v.checked_add(u64::from(digit)))
                .ok_or(HandleParseError::GidOverflow)?;
        }
        if value == 0 {
            return Err(HandleParseError::GidZero);
        }

        let mac_chars: Vec<char> = mac_part.chars().collect();
        if mac_chars.len() != 2 {
            return Err(HandleParseError::MacLength(mac_chars.len()));
        }
        let hi = decode_char(mac_chars[0])?;
        let lo = decode_char(mac_chars[1])?;
        let mac10 = (u16::from(hi) << 5) | u16::from(lo);

        Ok(ParsedHandle {
            gid: Gid::from_raw(value),
            mac_claimed: mac10.to_be_bytes(),
        })
    }
}

fn strip_prefix_ci(s: &str) -> Option<&str> {
    let head = s.get(..PREFIX.len())?;
    if head.eq_ignore_ascii_case(PREFIX) {
        s.get(PREFIX.len()..)
    } else {
        None
    }
}

/// Maps one character to its base32 value, applying the Crockford aliases.
fn decode_char(c: char) -> Result<u8, HandleParseError> {
    let lower = c.to_ascii_lowercase();
    let normalized = match lower {
        'o' => '0',
        'i' | 'l' => '1',
        other => other,
    };
    ALPHABET
        .iter()
        .position(|&a| char::from(a) == normalized)
        .and_then(|p| u8::try_from(p).ok())
        .ok_or(HandleParseError::InvalidCharacter(c))
}

/// Appends the minimal base32 digits of `value`, left-padded to 4 characters.
fn encode_gid(mut value: u64, out: &mut String) {
    let mut digits = [0u8; MAX_GID_CHARS];
    let mut n = 0usize;
    loop {
        #[allow(clippy::cast_possible_truncation)]
        let d = (value & 31) as usize;
        digits[n] = ALPHABET[d];
        n += 1;
        value >>= 5;
        if value == 0 || n == MAX_GID_CHARS {
            break;
        }
    }
    while n < HANDLE_MIN_GID_CHARS {
        digits[n] = b'0';
        n += 1;
    }
    for i in (0..n).rev() {
        out.push(char::from(digits[i]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn encodes_known_shape_and_lowercase() {
        let h = Handle::new(Gid::from_raw(1), 0x3F);
        let s = h.encode();
        assert_eq!(s, "c:0001-1z");
        assert!(s[2..].chars().all(|c| !c.is_ascii_uppercase()));
    }

    #[test]
    fn parse_roundtrip_fixed() -> Result<(), HandleParseError> {
        let h = Handle::new(Gid::from_raw(123_456_789), 0x2AB);
        let p = Handle::parse(&h.encode())?;
        assert_eq!(p.gid, h.gid);
        assert_eq!(p.mac_value(), h.mac_value());
        Ok(())
    }

    #[test]
    fn parse_accepts_uppercase_and_aliases() -> Result<(), HandleParseError> {
        let canonical = Handle::parse("c:k7qd-3f")?;
        let upper = Handle::parse("C:K7QD-3F")?;
        assert_eq!(canonical, upper);
        // 'o' is an alias for '0', 'l' for '1': "oool" == "0001".
        let a = Handle::parse("c:0001-0v")?;
        let b = Handle::parse("c:oool-0V")?;
        assert_eq!(a.gid, Gid::from_raw(1));
        assert_eq!(b.gid, Gid::from_raw(1));
        Ok(())
    }

    #[test]
    fn parse_rejects_malformed_input() {
        assert_eq!(
            Handle::parse("k7qd-3f"),
            Err(HandleParseError::MissingPrefix)
        );
        assert_eq!(
            Handle::parse("c:k7qd3f"),
            Err(HandleParseError::MissingSeparator)
        );
        assert_eq!(
            Handle::parse("c:k7-3f"),
            Err(HandleParseError::GidLength(2))
        );
        assert_eq!(
            Handle::parse("c:k7qd-3"),
            Err(HandleParseError::MacLength(1))
        );
        assert_eq!(
            Handle::parse("c:k7qd-3fx"),
            Err(HandleParseError::MacLength(3))
        );
        assert_eq!(Handle::parse("c:0000-00"), Err(HandleParseError::GidZero));
        assert_eq!(
            Handle::parse("c:k7qu-3f"),
            Err(HandleParseError::InvalidCharacter('u'))
        );
        assert_eq!(
            Handle::parse("c:zzzzzzzzzzzzzz-00"),
            Err(HandleParseError::GidLength(14))
        );
        assert_eq!(
            Handle::parse("c:zzzzzzzzzzzzz-00"),
            Err(HandleParseError::GidOverflow)
        );
    }

    #[test]
    fn new_discards_bits_above_ten() {
        let h = Handle::new(Gid::from_raw(9), 0xFFFF);
        assert_eq!(h.mac_value(), 0x3FF);
        assert_eq!(h.mac[0] & 0xFC, 0);
    }

    proptest! {
        #[test]
        fn encode_parse_roundtrip(gid in 1u64..=u64::MAX, mac in 0u16..=0x3FF) {
            let h = Handle::new(Gid::from_raw(gid), mac);
            let p = Handle::parse(&h.encode());
            prop_assert!(p.is_ok());
            if let Ok(p) = p {
                prop_assert_eq!(p.gid, h.gid);
                prop_assert_eq!(p.mac_value(), mac);
            }
        }
    }
}
