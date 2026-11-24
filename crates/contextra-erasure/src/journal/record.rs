//! Journal-Eintragstypen und Zustandsdefinitionen (SPEC §11.6, Anhang F.9, ER-29).

use crate::error::ErasureError;
use serde::{Deserialize, Serialize};

pub use crate::saga::SagaState;

/// Record-Codes nach SPEC Anhang F.9.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RecordCode {
    /// 1: Requested+Intent
    RequestedIntent = 1,
    /// 2: Gated
    Gated = 2,
    /// 3: KeyDestroyed
    KeyDestroyed = 3,
    /// 4: Purging
    Purging = 4,
    /// 5: Verifying
    Verifying = 5,
    /// 6: Completed
    Completed = 6,
    /// 7: Pending (mit phase, reason)
    Pending = 7,
    /// 8: Refused (mit reason)
    Refused = 8,
    /// 9: RestrictedRead
    RestrictedRead = 9,
    /// 10: ObserverAck
    ObserverAck = 10,
    /// 11: Lifted (mit reason)
    Lifted = 11,
    /// 12: Restricted
    Restricted = 12,
}

impl RecordCode {
    /// Konvertiert ein Byte in `RecordCode`.
    pub fn from_u8(val: u8) -> Result<Self, ErasureError> {
        match val {
            1 => Ok(Self::RequestedIntent),
            2 => Ok(Self::Gated),
            3 => Ok(Self::KeyDestroyed),
            4 => Ok(Self::Purging),
            5 => Ok(Self::Verifying),
            6 => Ok(Self::Completed),
            7 => Ok(Self::Pending),
            8 => Ok(Self::Refused),
            9 => Ok(Self::RestrictedRead),
            10 => Ok(Self::ObserverAck),
            11 => Ok(Self::Lifted),
            12 => Ok(Self::Restricted),
            _ => Err(ErasureError::Corrupt),
        }
    }

    /// Gibt den numerischen Code zurück.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Mappt RecordCode auf den entsprechenden SagaState.
    pub fn to_saga_state(self) -> SagaState {
        match self {
            Self::RequestedIntent => SagaState::Requested,
            Self::Gated => SagaState::Gated,
            Self::KeyDestroyed => SagaState::KeyDestroyed,
            Self::Purging => SagaState::Purging,
            Self::Verifying => SagaState::Verifying,
            Self::Completed => SagaState::Completed,
            Self::Pending => SagaState::Pending,
            Self::Refused => SagaState::Refused,
            Self::RestrictedRead | Self::Restricted => SagaState::Restricted,
            Self::ObserverAck => SagaState::Completed,
            Self::Lifted => SagaState::Lifted,
        }
    }
}

/// Zieltypen (target_type) nach SPEC Anhang F.9.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TargetType {
    /// 1: Document
    Document = 1,
    /// 2: Subject
    Subject = 2,
    /// 3: Collection
    Collection = 3,
    /// 4: Tenant
    Tenant = 4,
    /// 5: Superseded
    Superseded = 5,
    /// 6: DocumentReplace
    DocumentReplace = 6,
}

impl TargetType {
    /// Konvertiert ein Byte in `TargetType`.
    pub fn from_u8(val: u8) -> Result<Self, ErasureError> {
        match val {
            1 => Ok(Self::Document),
            2 => Ok(Self::Subject),
            3 => Ok(Self::Collection),
            4 => Ok(Self::Tenant),
            5 => Ok(Self::Superseded),
            6 => Ok(Self::DocumentReplace),
            _ => Err(ErasureError::Corrupt),
        }
    }

    /// Gibt den numerischen Code zurück.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Vollständiger Journal-Eintrag nach SPEC Anhang F.9.
///
/// Klartext:
/// `erasure_id(16) || record_code(u8) || time(u64) || commitment(16) || target_type(u8) || retention_code(u8) || phase(u8) || reason(u8) || scope_tags(Liste lo/hi) || counters || witness_hashes`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalRecord {
    /// Lösch-Kennung.
    pub erasure_id: [u8; 16],
    /// Record-Code nach F.9.
    pub record_code: RecordCode,
    /// Zeitstempel in Mikrosekunden UTC.
    pub timestamp: u64,
    /// 16-Byte Commitment.
    pub commitment: [u8; 16],
    /// Zieltyp der Löschung.
    pub target_type: TargetType,
    /// Aufbewahrungscode.
    pub retention_code: u8,
    /// Phase (1..8).
    pub phase: u8,
    /// Pending- oder Refusal-Grund.
    pub reason: u8,
    /// Eingebettete Scope-Tags.
    pub scope_tags: Vec<u8>,
    /// Metrikzähler.
    pub counters: u64,
    /// Zeugen-Hashes der beteiligten Schichten.
    pub witness_hashes: Vec<[u8; 32]>,
}

impl JournalRecord {
    /// Erzeugt einen neuen `JournalRecord`.
    #[allow(clippy::too_many_arguments)]
    pub fn new_v2(
        erasure_id: [u8; 16],
        record_code: RecordCode,
        timestamp: u64,
        commitment: [u8; 16],
        target_type: TargetType,
        retention_code: u8,
        phase: u8,
        reason: u8,
        scope_tags: Vec<u8>,
        counters: u64,
        witness_hashes: Vec<[u8; 32]>,
    ) -> Self {
        Self {
            erasure_id,
            record_code,
            timestamp,
            commitment,
            target_type,
            retention_code,
            phase,
            reason,
            scope_tags,
            counters,
            witness_hashes,
        }
    }

    /// Serialisiert den Klartext nach SPEC F.9 in Bytes:
    /// `erasure_id(16) ‖ record_code(u8) ‖ time(u64) ‖ commitment(16) ‖ target_type(u8) ‖ retention_code(u8) ‖ phase(u8) ‖ reason(u8) ‖ scope_tags(len u16 + bytes) ‖ counters(u64) ‖ witness_hashes(len u16 + [u8; 32]*)`
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.erasure_id);
        out.push(self.record_code.as_u8());
        out.extend_from_slice(&self.timestamp.to_be_bytes());
        out.extend_from_slice(&self.commitment);
        out.push(self.target_type.as_u8());
        out.push(self.retention_code);
        out.push(self.phase);
        out.push(self.reason);

        let tag_len = u16::try_from(self.scope_tags.len()).unwrap_or(u16::MAX);
        out.extend_from_slice(&tag_len.to_be_bytes());
        out.extend_from_slice(&self.scope_tags);

        out.extend_from_slice(&self.counters.to_be_bytes());

        let w_count = u16::try_from(self.witness_hashes.len()).unwrap_or(u16::MAX);
        out.extend_from_slice(&w_count.to_be_bytes());
        for h in &self.witness_hashes {
            out.extend_from_slice(h);
        }

        out
    }

    /// Deserialisiert den Klartext aus Bytes nach SPEC F.9.
    pub fn decode(bytes: &[u8]) -> Result<Self, ErasureError> {
        let min_len = 16 + 1 + 8 + 16 + 1 + 1 + 1 + 1 + 2 + 8 + 2;
        if bytes.len() < min_len {
            return Err(ErasureError::Corrupt);
        }

        let mut offset = 0;

        let erasure_id_slice = bytes
            .get(offset..offset + 16)
            .ok_or(ErasureError::Corrupt)?;
        let mut erasure_id = [0u8; 16];
        erasure_id.copy_from_slice(erasure_id_slice);
        offset += 16;

        let code_byte = bytes.get(offset).copied().ok_or(ErasureError::Corrupt)?;
        let record_code = RecordCode::from_u8(code_byte)?;
        offset += 1;

        let time_slice = bytes.get(offset..offset + 8).ok_or(ErasureError::Corrupt)?;
        let mut time_arr = [0u8; 8];
        time_arr.copy_from_slice(time_slice);
        let timestamp = u64::from_be_bytes(time_arr);
        offset += 8;

        let commitment_slice = bytes
            .get(offset..offset + 16)
            .ok_or(ErasureError::Corrupt)?;
        let mut commitment = [0u8; 16];
        commitment.copy_from_slice(commitment_slice);
        offset += 16;

        let target_byte = bytes.get(offset).copied().ok_or(ErasureError::Corrupt)?;
        let target_type = TargetType::from_u8(target_byte)?;
        offset += 1;

        let retention_code = bytes.get(offset).copied().ok_or(ErasureError::Corrupt)?;
        offset += 1;

        let phase = bytes.get(offset).copied().ok_or(ErasureError::Corrupt)?;
        offset += 1;

        let reason = bytes.get(offset).copied().ok_or(ErasureError::Corrupt)?;
        offset += 1;

        let tag_len_slice = bytes.get(offset..offset + 2).ok_or(ErasureError::Corrupt)?;
        let mut tag_len_arr = [0u8; 2];
        tag_len_arr.copy_from_slice(tag_len_slice);
        let tag_len = usize::from(u16::from_be_bytes(tag_len_arr));
        offset += 2;

        if bytes.len() < offset + tag_len + 8 + 2 {
            return Err(ErasureError::Corrupt);
        }
        let scope_tags = bytes
            .get(offset..offset + tag_len)
            .ok_or(ErasureError::Corrupt)?
            .to_vec();
        offset += tag_len;

        let counters_slice = bytes.get(offset..offset + 8).ok_or(ErasureError::Corrupt)?;
        let mut counters_arr = [0u8; 8];
        counters_arr.copy_from_slice(counters_slice);
        let counters = u64::from_be_bytes(counters_arr);
        offset += 8;

        let w_count_slice = bytes.get(offset..offset + 2).ok_or(ErasureError::Corrupt)?;
        let mut w_count_arr = [0u8; 2];
        w_count_arr.copy_from_slice(w_count_slice);
        let w_count = usize::from(u16::from_be_bytes(w_count_arr));
        offset += 2;

        let w_bytes_len = w_count.checked_mul(32).ok_or(ErasureError::Corrupt)?;
        if bytes.len() < offset + w_bytes_len {
            return Err(ErasureError::Corrupt);
        }

        let mut witness_hashes = Vec::with_capacity(w_count);
        for _ in 0..w_count {
            let h_slice = bytes
                .get(offset..offset + 32)
                .ok_or(ErasureError::Corrupt)?;
            let mut h = [0u8; 32];
            h.copy_from_slice(h_slice);
            witness_hashes.push(h);
            offset += 32;
        }

        Ok(Self {
            erasure_id,
            record_code,
            timestamp,
            commitment,
            target_type,
            retention_code,
            phase,
            reason,
            scope_tags,
            counters,
            witness_hashes,
        })
    }
}

/// Ein abwärtskompatibler Journal-Eintrag (SPEC §11.6, ER-29).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Lösch-ID.
    pub erasure_id: [u8; 16],
    /// Saga-Zustand.
    pub state: SagaState,
    /// Zeitstempel (Mikrosekunden UTC).
    pub timestamp: u64,
    /// Commitment-Hash (BLAKE3-keyed).
    pub commit_hash: [u8; 32],
}

impl JournalEntry {
    /// Additiver Konstruktor `new_v2` für erweiterte Initialisierung.
    #[allow(clippy::too_many_arguments)]
    pub fn new_v2(
        erasure_id: [u8; 16],
        record_code: RecordCode,
        timestamp: u64,
        commitment: [u8; 16],
        target_type: TargetType,
        retention_code: u8,
        phase: u8,
        reason: u8,
        scope_tags: Vec<u8>,
        counters: u64,
        witness_hashes: Vec<[u8; 32]>,
    ) -> JournalRecord {
        JournalRecord::new_v2(
            erasure_id,
            record_code,
            timestamp,
            commitment,
            target_type,
            retention_code,
            phase,
            reason,
            scope_tags,
            counters,
            witness_hashes,
        )
    }

    /// Konvertiert einen `JournalRecord` in einen `JournalEntry`.
    pub fn from_record(rec: &JournalRecord) -> Self {
        let mut commit_hash = [0u8; 32];
        if let Some(dest) = commit_hash.get_mut(0..16) {
            dest.copy_from_slice(&rec.commitment);
        }
        Self {
            erasure_id: rec.erasure_id,
            state: rec.record_code.to_saga_state(),
            timestamp: rec.timestamp,
            commit_hash,
        }
    }

    /// Konvertiert `self` in einen `JournalRecord`.
    pub fn to_record(&self) -> JournalRecord {
        let record_code = match self.state {
            SagaState::Requested => RecordCode::RequestedIntent,
            SagaState::Gated => RecordCode::Gated,
            SagaState::KeyDestroyed => RecordCode::KeyDestroyed,
            SagaState::Purging => RecordCode::Purging,
            SagaState::Verifying => RecordCode::Verifying,
            SagaState::Completed => RecordCode::Completed,
            SagaState::Pending => RecordCode::Pending,
            SagaState::Refused => RecordCode::Refused,
            SagaState::Restricted => RecordCode::Restricted,
            SagaState::Lifted => RecordCode::Lifted,
        };

        let mut commitment = [0u8; 16];
        if let (Some(dest), Some(src)) = (commitment.get_mut(0..16), self.commit_hash.get(0..16)) {
            dest.copy_from_slice(src);
        }

        JournalRecord {
            erasure_id: self.erasure_id,
            record_code,
            timestamp: self.timestamp,
            commitment,
            target_type: TargetType::Document,
            retention_code: 0,
            phase: 0,
            reason: 0,
            scope_tags: Vec::new(),
            counters: 0,
            witness_hashes: Vec::new(),
        }
    }
}
