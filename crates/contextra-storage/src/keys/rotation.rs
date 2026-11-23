//! # Rotationsschranke je Schlüssel (SPEC §16.3, D-28)
//!
//! Die Kollisionswahrscheinlichkeit der 24-Byte-Zufallsnonces bleibt vernachlässigbar,
//! solange je Schlüssel höchstens `2^32` Versiegelungen stattfinden. Bei Erreichen der
//! Schranke beginnt eine neue Schlüsselepoche (`key_epoch`).
//!
//! Dieses Modul liefert nur die Schranke und die Entscheidung darüber. Die Zählung
//! selbst geschieht blockweise über die Reservierung `seal_hw` im Catalog (§10.8) und
//! gehört nicht hierher.

/// Maximale Anzahl Versiegelungen je Clusterschlüssel und je Space-DEK (SPEC §16.3).
///
/// Eine höhere Schranke ist nur über eine Spezifikationsänderung zulässig (D-28).
pub const SEAL_ROTATION_LIMIT: u64 = 1 << 32;

/// Entscheidet, ob ein Schlüssel nach `seals_on_key` Versiegelungen rotiert werden muss.
///
/// Gibt `true` zurück, sobald die Schranke erreicht ist (`>=`), damit die
/// Versiegelung `2^32` selbst nicht mehr unter demselben Schlüssel stattfindet.
pub fn rotation_due(seals_on_key: u64) -> bool {
    seals_on_key >= SEAL_ROTATION_LIMIT
}
