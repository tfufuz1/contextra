//! Domänenlogik für Sonderziel Berichtigung mit Purge alter Versionen (`update_with(..., Purge)`, SPEC §11.2, D-117, Zieltyp 5).
//!
//! # Normative Referenzen (D-117, F.9)
//! - **Matcher `PrefixSeq { prefix[65], seq_lt }`:** Trifft jeden Schlüssel mit Präfix
//!   `DATA || tenant || coll || cluster || doc_tag` und commit_seq < seq_lt.
//! - **Protokoll (Zieltyp 5, Superseded):**
//!   - Journal `Requested+Intent` mit `intent_seq`.
//!   - `PreGate`: Engine committet die neue Version (`s_n`).
//!   - Gate `PrefixSeq { prefix, seq_lt = s_n }`, danach Purge alter Versionen.
//!   - Fehlt nach Absturz eine Version mit `seq > intent_seq`, bleibt alles unberührt
//!     und der Status ist `Pending { phase: Supersede, reason: UpdateMissing }`.
//! - **Klasse K nicht erreicht:** Der DEK bleibt gleich, kein Schlüssel wird vernichtet.
//!   Der Bericht führt `target = Superseded`, G und P mit Zeugen, K `not_reached`,
//!   `key_coverage` je Schicht `none` und das Restrisiko `SupersededWithSameKey`.

use crate::error::ErasureError;
use contextra_types::erasure::KeyCoverage;
use contextra_types::ids::ErasureId;
use serde::{Deserialize, Serialize};

/// Modus der Berichtigung (SPEC D-117).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupersedeMode {
    /// Standard: Alte Versionen bleiben in der Historie erhalten.
    Keep,
    /// Purge: Vorherige Versionen werden gegated und physisch entfernt (Art. 16 DSGVO).
    PurgeOldVersions,
}

mod prefix_serde {
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(prefix: &[u8; 65], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        prefix.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 65], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<u8> = Vec::deserialize(deserializer)?;
        if vec.len() != 65 {
            return Err(D::Error::custom(format!(
                "invalid prefix length: expected 65, got {}",
                vec.len()
            )));
        }
        let mut arr = [0u8; 65];
        arr.copy_from_slice(&vec);
        Ok(arr)
    }
}

/// Matcher für das Prädikat `PrefixSeq` (SPEC D-117, F.9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixSeqMatcher {
    #[serde(with = "prefix_serde")]
    prefix: [u8; 65],
    seq_lt: u64,
}

impl PrefixSeqMatcher {
    /// Erstellt einen neuen Matcher mit 65-Byte-Präfix und Sequenzobergrenze (exklusiv).
    pub const fn new(prefix: [u8; 65], seq_lt: u64) -> Self {
        Self { prefix, seq_lt }
    }

    /// Liefert das 65-Byte-Präfix.
    #[inline]
    pub const fn prefix(&self) -> &[u8; 65] {
        &self.prefix
    }

    /// Liefert die exklusive Sequenz-Obergrenze.
    #[inline]
    pub const fn seq_lt(&self) -> u64 {
        self.seq_lt
    }

    /// Prüft, ob ein gegebener Schlüssel mit dem Präfix übereinstimmt und die Sequenz unter `seq_lt` liegt.
    #[inline]
    pub fn matches(&self, candidate_prefix: &[u8; 65], candidate_seq: u64) -> bool {
        self.prefix == *candidate_prefix && candidate_seq < self.seq_lt
    }
}

/// Plan für die Berichtigung mit Purge alter Versionen (SPEC D-117).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupersedePlan {
    erasure_id: ErasureId,
    matcher: PrefixSeqMatcher,
    intent_seq: u64,
    mode: SupersedeMode,
}

impl SupersedePlan {
    /// Erstellt einen neuen Plan für `Superseded`.
    pub fn new(
        erasure_id: ErasureId,
        prefix: [u8; 65],
        split_seq: u64,
        mode: SupersedeMode,
    ) -> Self {
        Self {
            erasure_id,
            matcher: PrefixSeqMatcher::new(prefix, split_seq),
            intent_seq: split_seq,
            mode,
        }
    }

    /// Liefert die zugehörige Lösch-ID.
    #[inline]
    pub fn erasure_id(&self) -> ErasureId {
        self.erasure_id
    }

    /// Liefert die Intent-Sequenz vor dem Update-Commit.
    #[inline]
    pub fn intent_seq(&self) -> u64 {
        self.intent_seq
    }

    /// Liefert den Matcher für Gate und Purge.
    #[inline]
    pub fn matcher(&self) -> &PrefixSeqMatcher {
        &self.matcher
    }

    /// Liefert den Berichtigungsmodus.
    #[inline]
    pub fn mode(&self) -> SupersedeMode {
        self.mode
    }

    /// Validiert, dass eine neue Version mit `seq > intent_seq` committet wurde (D-117).
    ///
    /// Liefert `ErasureError::UpdateMissing`, wenn die neue Version fehlt oder `seq <= intent_seq` ist.
    pub fn verify_updated_seq(&self, latest_committed_seq: u64) -> Result<(), ErasureError> {
        if latest_committed_seq <= self.intent_seq {
            return Err(ErasureError::UpdateMissing);
        }
        Ok(())
    }

    /// Zeigt an, ob auf Schlüsselebene etwas vernichtet wurde.
    ///
    /// SPEC D-117: Bei Supersede bleibt der DEK gleich, kein Schlüssel wird vernichtet.
    #[inline]
    pub const fn is_key_destroyed(&self) -> bool {
        false
    }

    /// Schlüsselabdeckung der Speicherschicht (SPEC D-117: None / not_reached).
    #[inline]
    pub const fn key_coverage_store(&self) -> KeyCoverage {
        KeyCoverage::None
    }

    /// Schlüsselabdeckung des Textindex (SPEC D-117: None / not_reached).
    #[inline]
    pub const fn key_coverage_text(&self) -> KeyCoverage {
        KeyCoverage::None
    }

    /// Schlüsselabdeckung des Vektorindex (SPEC D-117: None / not_reached).
    #[inline]
    pub const fn key_coverage_vector(&self) -> KeyCoverage {
        KeyCoverage::None
    }
}
