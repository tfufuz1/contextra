//! Identitätstypen. Debug ist redigiert, wo Kennungen Rückschlüsse erlauben (SP-PORTS §3.3).
use core::fmt;

pub const SYSTEM_SPACE_ID_MAX: u64 = 255;
pub const FIRST_USER_SPACE_ID: u64 = SYSTEM_SPACE_ID_MAX + 1;

/// Stamp = globale Versionsmarke, 56 Bit nutzbar.
pub type Stamp = u64;
pub const STAMP_MAX: Stamp = (1u64 << 56) - 1;

/// Umhüllt einen Wert, dessen Inhalt nie in Debug-Ausgaben erscheinen darf.
pub struct Redacted<T>(pub T);

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClusterId([u8; 16]);

impl ClusterId {
    /// Nur für den Wire-Decoder (SP-CODEC). Weitere Konstruktionspfade existieren nicht.
    #[doc(hidden)]
    pub fn from_wire(raw: [u8; 16]) -> Self {
        Self(raw)
    }
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for ClusterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ClusterId(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpaceId(pub u64);

impl SpaceId {
    pub const fn is_system(self) -> bool { self.0 <= SYSTEM_SPACE_ID_MAX }

    pub fn user(value: u64) -> Result<Self, crate::Error> {
        if value < FIRST_USER_SPACE_ID { return Err(crate::Error::ReservedSpace); }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BranchId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BatchId([u8; 16]);

impl BatchId {
    pub fn from_bytes(raw: [u8; 16]) -> Self { Self(raw) }
}

impl fmt::Debug for BatchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("BatchId(<redacted>)") }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdemToken([u8; 16]);

impl IdemToken {
    pub fn from_bytes(raw: [u8; 16]) -> Self {
        Self(raw)
    }
}
