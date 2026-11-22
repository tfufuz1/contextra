//! # System-Limits und Schwellenwerte
//!
//! Normative Obergrenzen aus SPEC §6.7.

use std::fmt;

/// Maximale Länge eines Nutzerschlüssels (4 KiB).
pub const MAX_KEY_BYTES: usize = 4096;

/// Maximale Headerlänge je Eintrag (255 B).
pub const MAX_HEADER_LEN: usize = 255;

/// Maximaler Wertkörper standardmäßig (1 MiB).
pub const MAX_VALUE_BYTES: usize = 1024 * 1024;

/// Maximale Größe eines Changes (32 MiB).
pub const MAX_CHANGE_BYTES: usize = 32 * 1024 * 1024;

/// Maximale Anzahl von Operationen in einem Change.
pub const MAX_OPS_PER_CHANGE: usize = 100_000;

/// Maximale Größe eines Redo-Frames (64 MiB).
pub const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;

/// Maximale Anzahl an Gewichten je Eintrag.
pub const MAX_WEIGHTS: usize = 4;

/// Maximale Anzahl an Spaces (64).
pub const MAX_SPACES: usize = 64;

/// Maximale Anzahl registrierter Consumer (64).
pub const MAX_CONSUMERS: usize = 64;

/// Maximale Anzahl aktiver Holds je Instanz (10 000).
pub const MAX_HOLDS: usize = 10_000;

/// Maximale Anzahl dauerhafter Holds (1 000).
pub const MAX_DURABLE_HOLDS: usize = 1_000;

/// Maximale Länge eines Spacenamens in Bytes.
pub const MAX_SPACE_NAME_LEN: usize = 64;

/// Maximale Stamp-Grenze (< 2^56, INV-STAMP-2).
pub const MAX_STAMP: u64 = (1 << 56) - 1;

/// Fehler beim Überschreiten einer Systemgrenze.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitExceeded {
    /// Überschrittene Obergrenze.
    pub max: u64,
    /// Tatsächlicher Wert.
    pub actual: u64,
}

impl fmt::Display for LimitExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Limit überschritten: Wert {} übersteigt Maximum {}",
            self.actual, self.max
        )
    }
}

impl std::error::Error for LimitExceeded {}

/// Typ-sichere Begrenzung für numerische Größen (Parse, don't validate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bounded<T, const MAX: u64>(T);

impl<const MAX: u64> Bounded<u64, MAX> {
    /// Erstellt einen neuen begrenzten Wert, falls `val <= MAX`.
    pub fn try_new(val: u64) -> Result<Self, LimitExceeded> {
        if val <= MAX {
            Ok(Self(val))
        } else {
            Err(LimitExceeded { max: MAX, actual: val })
        }
    }

    /// Liefert den Rohwert.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl<const MAX: u64> Bounded<usize, MAX> {
    /// Erstellt einen neuen begrenzten `usize`-Wert, falls `val <= MAX`.
    pub fn try_new(val: usize) -> Result<Self, LimitExceeded> {
        match u64::try_from(val) {
            Ok(v) if v <= MAX => Ok(Self(val)),
            Ok(v) => Err(LimitExceeded { max: MAX, actual: v }),
            Err(_) => Err(LimitExceeded { max: MAX, actual: u64::MAX }),
        }
    }

    /// Liefert den Rohwert.
    pub const fn get(self) -> usize {
        self.0
    }
}
