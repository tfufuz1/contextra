//! # System-Limits und Schwellenwerte
//!
//! Die Konstanten sind die **Obergrenzen der Version** aus SPEC §6.8 (Spalte „Obergrenze“).
//! Die Standardwerte (Spalte „Standard“) liegen in `Config` (§20.2) und dürfen nie über
//! diesen Obergrenzen liegen (§20.1). Dekodierung prüft Längen gegen diese Werte, bevor
//! allokiert wird (INV-FMT-2, P-4).
//!
//! Werte ohne Eintrag in §6.8 sind als Annahme markiert (siehe `SA`-Vorschläge in der PR).

use std::fmt;

/// Obergrenze `max_key_bytes` (4 KiB, §6.8).
pub const MAX_KEY_BYTES: usize = 4 * 1024;

/// Obergrenze `max_header_bytes` (64 KiB, §6.8). Standard: 4 KiB.
pub const MAX_HEADER_LEN: usize = 64 * 1024;

/// Obergrenze `max_body_bytes` (256 MiB, §6.8). Standard: 16 MiB.
pub const MAX_VALUE_BYTES: usize = 256 * 1024 * 1024;

/// Obergrenze `max_change_bytes` (64 MiB, §6.8). Standard: 8 MiB.
pub const MAX_CHANGE_BYTES: usize = 64 * 1024 * 1024;

/// Obergrenze `max_change_ops` (65 536, §6.8). Standard: 4 096.
pub const MAX_OPS_PER_CHANGE: usize = 65_536;

/// Obergrenze `max_page_bytes` (16 MiB, §6.8). Standard: 1 MiB.
pub const MAX_PAGE_BYTES: usize = 16 * 1024 * 1024;

/// Obergrenze der Frame-Länge (§20.1, „Frame-Länge“). **Annahme:** §6.8 nennt keinen Wert.
/// Vorläufig gleich `MAX_CHANGE_BYTES`, weil ein Frame genau einen Change trägt (INV-LOG-1).
pub const MAX_FRAME_BYTES: usize = MAX_CHANGE_BYTES;

/// Maximale Anzahl an Gewichten je Eintrag (`weight_slots` 0…4, §6.3).
pub const MAX_WEIGHTS: usize = 4;

/// Maximale Größe des Spacenamens. **Annahme:** nicht in §6.8 spezifiziert.
pub const MAX_SPACE_NAME_LEN: usize = 64;

/// Maximale Anzahl Spaces. **Annahme:** nicht in §6.8 spezifiziert.
pub const MAX_SPACES: usize = 64;

/// Maximale Anzahl registrierter Consumer. **Annahme:** nicht in §6.8 spezifiziert.
pub const MAX_CONSUMERS: usize = 64;

/// Maximale Anzahl aktiver Holds je Instanz. **Annahme:** nicht in §6.8 spezifiziert.
pub const MAX_HOLDS: usize = 10_000;

/// Maximale Anzahl dauerhafter Holds. **Annahme:** nicht in §6.8 spezifiziert.
pub const MAX_DURABLE_HOLDS: usize = 1_000;

/// Maximaler Stamp (`< 2^56`, INV-STAMP-2).
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Obergrenzen stimmen mit SPEC §6.8 überein (Tabelle „Obergrenze“).
    #[test]
    fn obergrenzen_entsprechen_spec_6_8() {
        assert_eq!(MAX_KEY_BYTES, 4096);
        assert_eq!(MAX_HEADER_LEN, 65_536);
        assert_eq!(MAX_VALUE_BYTES, 256 * 1024 * 1024);
        assert_eq!(MAX_CHANGE_BYTES, 64 * 1024 * 1024);
        assert_eq!(MAX_OPS_PER_CHANGE, 65_536);
        assert_eq!(MAX_PAGE_BYTES, 16 * 1024 * 1024);
    }

    #[test]
    fn stamp_grenze_ist_2_hoch_56_minus_1() {
        assert_eq!(MAX_STAMP, (1u64 << 56) - 1);
    }

    #[test]
    fn bounded_lehnt_ueberschreitung_ab() {
        assert!(Bounded::<u64, 10>::try_new(10).is_ok());
        assert_eq!(
            Bounded::<u64, 10>::try_new(11),
            Err(LimitExceeded { max: 10, actual: 11 })
        );
        assert!(Bounded::<usize, 10>::try_new(11).is_err());
    }
}
