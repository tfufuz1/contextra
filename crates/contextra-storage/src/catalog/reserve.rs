//! # ID-Reservierung und Hochwassermarken
//!
//! SPEC §9.7 (INV-KEY-2): IDs (run_id, blob_no, segment_id, tmp_id) werden erst nach
//! dauerhaftem Hochwassermarken-Edit benutzt und nach Absturz nie wiederverwendet.

use crate::catalog::state::CatalogState;
use std::fmt;

/// Blockgröße für die schrittweise Vorab-Reservierung von IDs.
pub const ID_RESERVE_BLOCK: u64 = 256;

/// Maximale ID für run_id, blob_no, tmp_id (< 2^56).
pub const MAX_ID_56: u64 = (1u64 << 56) - 1;

/// Maximale ID für segment_id (< 2^32).
pub const MAX_SEGMENT_ID_32: u64 = (1u64 << 32) - 1;

/// Fehler bei der ID-Vergabe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdError {
    /// ID-Vorrat ist erschöpft; ein neuer Catalog-Edit mit Hochwassermarke ist erforderlich.
    NeedsReservation,
    /// ID-Raum ist permanent aufgebraucht (SPEC §9.7 Exhausted).
    Exhausted,
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NeedsReservation => write!(f, "ID-Vorrat erschöpft: Catalog-Reservierung nötig"),
            Self::Exhausted => write!(f, "ID-Grenze überschritten (Exhausted)"),
        }
    }
}

impl std::error::Error for IdError {}

/// Verwalter für die deterministische, crash-sichere ID-Vergabe (INV-KEY-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdReserve {
    next_run_id: u64,
    /// Hochwassermarke für run_id.
    pub run_id_hwm: u64,

    next_blob_no: u64,
    /// Hochwassermarke für blob_no.
    pub blob_no_hwm: u64,

    next_segment_id: u64,
    /// Hochwassermarke für segment_id.
    pub segment_id_hwm: u64,

    next_tmp_id: u64,
    /// Hochwassermarke für tmp_id.
    pub tmp_id_hwm: u64,

    /// Minimale Schlüsselgeneration nach KEK-Rotation.
    pub keys_generation_min: u64,
}

impl IdReserve {
    /// Initialisiert die Reservierung aus dem Katalogzustand nach Recovery.
    ///
    /// INV-KEY-2: Vergabe beginnt nach Recovery bei der Hochwassermarke.
    /// Zuvor allokierte, aber nicht genutzte IDs verfallen.
    pub fn new(state: &CatalogState) -> Self {
        Self {
            next_run_id: state.run_id_hwm,
            run_id_hwm: state.run_id_hwm,
            next_blob_no: state.blob_no_hwm,
            blob_no_hwm: state.blob_no_hwm,
            next_segment_id: state.segment_id_hwm,
            segment_id_hwm: state.segment_id_hwm,
            next_tmp_id: state.tmp_id_hwm,
            tmp_id_hwm: state.tmp_id_hwm,
            keys_generation_min: state.keys_generation_min,
        }
    }

    /// Erstellt eine Reservierungsinstanz direkt aus Hochwassermarken (z.B. für Tests/Recovery).
    pub fn recover(
        run_id_hwm: u64,
        blob_no_hwm: u64,
        segment_id_hwm: u64,
        tmp_id_hwm: u64,
        keys_generation_min: u64,
    ) -> Self {
        Self {
            next_run_id: run_id_hwm,
            run_id_hwm,
            next_blob_no: blob_no_hwm,
            blob_no_hwm,
            next_segment_id: segment_id_hwm,
            segment_id_hwm,
            next_tmp_id: tmp_id_hwm,
            tmp_id_hwm,
            keys_generation_min,
        }
    }

    /// Allokiert eine neue run_id und reserviert bei Bedarf einen neuen Block.
    pub fn ensure_run_id(&mut self) -> Result<u64, IdError> {
        if self.next_run_id >= self.run_id_hwm {
            let (new_r, _, _, _, _) = self.plan_reservation()?;
            self.run_id_hwm = new_r;
        }
        self.alloc_run_id()
    }

    /// Allokiert eine neue run_id.
    pub fn alloc_run_id(&mut self) -> Result<u64, IdError> {
        if self.next_run_id >= self.run_id_hwm {
            return Err(IdError::NeedsReservation);
        }
        if self.next_run_id >= MAX_ID_56 {
            return Err(IdError::Exhausted);
        }
        let id = self.next_run_id;
        self.next_run_id = self.next_run_id.checked_add(1).ok_or(IdError::Exhausted)?;
        Ok(id)
    }

    /// Allokiert eine neue blob_no und reserviert bei Bedarf einen neuen Block.
    pub fn ensure_blob_no(&mut self) -> Result<u64, IdError> {
        if self.next_blob_no >= self.blob_no_hwm {
            let (_, new_b, _, _, _) = self.plan_reservation()?;
            self.blob_no_hwm = new_b;
        }
        self.alloc_blob_no()
    }

    /// Allokiert eine neue blob_no.
    pub fn alloc_blob_no(&mut self) -> Result<u64, IdError> {
        if self.next_blob_no >= self.blob_no_hwm {
            return Err(IdError::NeedsReservation);
        }
        if self.next_blob_no >= MAX_ID_56 {
            return Err(IdError::Exhausted);
        }
        let id = self.next_blob_no;
        self.next_blob_no = self.next_blob_no.checked_add(1).ok_or(IdError::Exhausted)?;
        Ok(id)
    }

    /// Allokiert eine neue segment_id.
    pub fn alloc_segment_id(&mut self) -> Result<u64, IdError> {
        if self.next_segment_id >= self.segment_id_hwm {
            return Err(IdError::NeedsReservation);
        }
        if self.next_segment_id >= MAX_SEGMENT_ID_32 {
            return Err(IdError::Exhausted);
        }
        let id = self.next_segment_id;
        self.next_segment_id = self.next_segment_id.checked_add(1).ok_or(IdError::Exhausted)?;
        Ok(id)
    }

    /// Allokiert eine neue tmp_id.
    pub fn alloc_tmp_id(&mut self) -> Result<u64, IdError> {
        if self.next_tmp_id >= self.tmp_id_hwm {
            return Err(IdError::NeedsReservation);
        }
        if self.next_tmp_id >= MAX_ID_56 {
            return Err(IdError::Exhausted);
        }
        let id = self.next_tmp_id;
        self.next_tmp_id = self.next_tmp_id.checked_add(1).ok_or(IdError::Exhausted)?;
        Ok(id)
    }

    /// Berechnet die nächsten Hochwassermarken zur Reservierung in Blöcken von 256.
    pub fn plan_reservation(&self) -> Result<(u64, u64, u64, u64, u64), IdError> {
        let new_r = self.run_id_hwm.checked_add(ID_RESERVE_BLOCK).ok_or(IdError::Exhausted)?;
        if new_r > MAX_ID_56 { return Err(IdError::Exhausted); }

        let new_b = self.blob_no_hwm.checked_add(ID_RESERVE_BLOCK).ok_or(IdError::Exhausted)?;
        if new_b > MAX_ID_56 { return Err(IdError::Exhausted); }

        let new_s = self.segment_id_hwm.checked_add(ID_RESERVE_BLOCK).ok_or(IdError::Exhausted)?;
        if new_s > MAX_SEGMENT_ID_32 { return Err(IdError::Exhausted); }

        let new_t = self.tmp_id_hwm.checked_add(ID_RESERVE_BLOCK).ok_or(IdError::Exhausted)?;
        if new_t > MAX_ID_56 { return Err(IdError::Exhausted); }

        Ok((new_r, new_b, new_s, new_t, self.keys_generation_min))
    }

    /// Aktualisiert die Hochwassermarken nach erfolgreichem dauerhaftem Commit des Edits.
    pub fn apply_reservation(&mut self, state: &CatalogState) {
        self.run_id_hwm = state.run_id_hwm;
        self.blob_no_hwm = state.blob_no_hwm;
        self.segment_id_hwm = state.segment_id_hwm;
        self.tmp_id_hwm = state.tmp_id_hwm;
        self.keys_generation_min = state.keys_generation_min;
    }
}
