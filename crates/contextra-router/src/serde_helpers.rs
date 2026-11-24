// FILE-CONTEXT
// STAND: 2026-10-29
// ZWECK: Serde-Hilfsfunktionen für deterministische JSON-Serialisierung.
// INVARIANTEN: BTreeSet<u64> wird in natürlicher Reihenfolge serialisiert.

//! # Serde-Hilfsfunktionen für contextra-router
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Bereitstellung von Hilfsfunktionen für die deterministische Serialisierung von Mengen.
//! Garantiert, dass `BTreeSet<u64>` sortiert als Sequenz/Array serialisiert und deserialisiert wird.
//!
//! **Abhängigkeiten:**
//! - Upstream: `serde`, `std::collections::BTreeSet`.
//! - Downstream: Genutzt für persistierte Zustände und Prüfungen.

// ANCHOR[contextra-router:SERDE-001]
/// Modul zur deterministischen Serialisierung von `BTreeSet<u64>` als sortierter Vektor.
pub mod sorted_u64_set {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::BTreeSet;

    // ANCHOR[contextra-router:SERDE-002]
    /// Serialisiert ein `BTreeSet<u64>` deterministisch als sortierten `Vec<u64>`.
    ///
    /// # Ablauf
    /// 1. Elemente des Sets über geordneten Iterator in einen `Vec<u64>` kopieren.
    /// 2. Den Vektor an den Serializer übergeben.
    ///
    /// # Vorbedingungen
    /// Keine. Das Set garantiert bereits die aufsteigende Sortierung.
    ///
    /// # Nachbedingungen
    /// Das Ergebnis im Datenstrom ist ein geordnetes Array.
    ///
    /// # Fehler
    /// Gibt `S::Error` zurück, wenn der zugrundeliegende Serializer fehlschlägt.
    ///
    /// # Nebenwirkungen
    /// Keine (reine Speicheroperation und Delegation an den Serializer).
    // NOTE: Determinismus ist entscheidend für reproduzierbare Hashes und Audits.
    pub fn serialize<S: Serializer>(set: &BTreeSet<u64>, s: S) -> Result<S::Ok, S::Error> {
        let v: Vec<u64> = set.iter().copied().collect();
        v.serialize(s)
    }

    // ANCHOR[contextra-router:SERDE-003]
    /// Deserialisiert eine Sequenz von `u64`-Werten in ein geordnetes `BTreeSet<u64>`.
    ///
    /// # Ablauf
    /// 1. Sequenz aus dem Deserializer als `Vec<u64>` einlesen.
    /// 2. Vektor in ein `BTreeSet` überführen (Duplikate werden automatisch dedupliziert).
    ///
    /// # Vorbedingungen
    /// Der Deserializer liefert eine gültige Sequenz von ganzzahligen 64-Bit-Werten.
    ///
    /// # Nachbedingungen
    /// Liefert ein vollständiges `BTreeSet<u64>` zurück.
    ///
    /// # Fehler
    /// Gibt `D::Error` zurück, falls das Eingabeformat ungültig ist.
    ///
    /// # Nebenwirkungen
    /// Keine.
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeSet<u64>, D::Error> {
        Vec::<u64>::deserialize(d).map(|v| v.into_iter().collect())
    }
}
