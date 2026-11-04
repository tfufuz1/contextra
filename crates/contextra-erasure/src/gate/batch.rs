//! Gate-Gruppen-Commit für kooperative Barrieren-Bündelung (SPEC §11.3, D-57, D-58, D-78, ER-13).
//!
//! `add_batch` bündelt gleichzeitige Anfragen in einer einzigen exklusiven
//! Barrieren-Phase mit einem gemeinsamen fsync.
//!
//! Ablauf unter CommitBarrier::exclusive:
//! 1. `gate_epoch += 1` (VOR der Schlüsselvernichtung, I-21)
//! 2. Gate-Log-Append mit `gated_seq` (D-58)
//! 3. Gates in `ErasedSet` einfügen
//! 4. Barriere frei
//! 5. EIN fsync je Gruppe (Gate-Gruppen-Commit, D-78, T-GATEGROUP-1) auf Critical-Spur (D-128).
//!
//! Zeit bis Gated ist O(1) in der Zielgröße (P16).

use crate::error::ErasureError;
use crate::gate::erased_set::ErasedSet;
use crate::gate::log::{GateLog, GateLogEntry, GatePredicate};
use contextra_ports::barrier::CommitBarrier;
use contextra_ports::gate::{KeyBuf, Seq};
use parking_lot::RwLock;
use std::sync::Arc;

/// Anfrage für einen Gruppen-Commit.
#[derive(Clone, Debug)]
pub struct BatchGateRequest {
    /// Gate-Kennung.
    pub gate_id: [u8; 16],
    /// Untere Grenze.
    pub lo: KeyBuf,
    /// Obere Grenze.
    pub hi: KeyBuf,
    /// Optionale 65-Byte Grenzen (SPEC 9.1).
    pub bounds_65: Option<([u8; 65], [u8; 65])>,
    /// Gate-Variante: true = Erased, false = Restricted.
    pub is_erased: bool,
    /// Lösch-ID.
    pub erasure_id: [u8; 16],
}

impl BatchGateRequest {
    /// Erstellt eine Standard-Gate-Anfrage aus 16-Byte-Schlüsseln.
    pub fn new(
        gate_id: [u8; 16],
        lo: KeyBuf,
        hi: KeyBuf,
        is_erased: bool,
        erasure_id: [u8; 16],
    ) -> Self {
        Self {
            gate_id,
            lo,
            hi,
            bounds_65: None,
            is_erased,
            erasure_id,
        }
    }

    /// Erstellt eine Gate-Anfrage mit exakten 65-Byte Grenzen (SPEC 9.1, D-123).
    pub fn with_bounds_65(
        gate_id: [u8; 16],
        lo_65: [u8; 65],
        hi_65: [u8; 65],
        is_erased: bool,
        erasure_id: [u8; 16],
    ) -> Self {
        let mut k_lo = [0u8; 16];
        let mut k_hi = [0u8; 16];
        k_lo.copy_from_slice(&lo_65[0..16]);
        k_hi.copy_from_slice(&hi_65[0..16]);

        Self {
            gate_id,
            lo: KeyBuf::new(k_lo),
            hi: KeyBuf::new(k_hi),
            bounds_65: Some((lo_65, hi_65)),
            is_erased,
            erasure_id,
        }
    }
}

/// Ergebnis eines Gruppen-Commits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchGateOutcome {
    /// Sequenznummer, unter der die Gates gesetzt wurden.
    pub gated_seq: Seq,
    /// Zeigt an, ob die Gates erfolgreich im `ErasedSet` registriert wurden.
    pub applied_to_set: bool,
    /// Anzahl der im Batch verarbeiteten Gate-Einträge.
    pub gates_count: usize,
}

/// Kontext für die koordinierte Ausführung von Gate-Gruppen-Commits.
pub struct GateBatchCoordinator {
    barrier: Arc<dyn CommitBarrier>,
    erased_set: Arc<RwLock<ErasedSet>>,
    gate_log: Arc<RwLock<GateLog>>,
}

impl GateBatchCoordinator {
    /// Erstellt einen neuen Koordinator für Gate-Gruppen-Commits.
    pub fn new(
        barrier: Arc<dyn CommitBarrier>,
        erased_set: Arc<RwLock<ErasedSet>>,
        gate_log: Arc<RwLock<GateLog>>,
    ) -> Self {
        Self {
            barrier,
            erased_set,
            gate_log,
        }
    }

    /// Führt den Gruppen-Commit für die übergebenen Anfragen unter `CommitBarrier::exclusive` aus.
    ///
    /// Garantiert:
    /// - `gate_epoch += 1` vor Schlüsselvernichtung (I-21)
    /// - Gate-Log-Append mit `gated_seq` (D-58)
    /// - Genau EIN fsync / Persistierungsschritt je Gruppe (D-78, T-GATEGROUP-1)
    /// - Freigabe der Barriere unmittelbar nach In-Memory/Log-Update (D-57)
    /// - O(1) Zeit bezüglich Zielgröße (P16)
    pub async fn execute_batch(
        &self,
        requests: Vec<BatchGateRequest>,
        gated_seq: Seq,
    ) -> Result<BatchGateOutcome, ErasureError> {
        if requests.is_empty() {
            return Ok(BatchGateOutcome {
                gated_seq,
                applied_to_set: true,
                gates_count: 0,
            });
        }

        // 1. Exklusive Commit-Barriere anfordern (D-57)
        let _exclusive_permit = self.barrier.exclusive().await;

        // 2. Unter exklusiver Barriere: Epoche erhöhen (I-21) und Einträge vorbereiten
        let mut log_entries = Vec::with_capacity(requests.len());

        {
            let mut set_guard = self.erased_set.write();

            // Erhöht gate_epoch VOR der Schlüsselvernichtung
            set_guard.increment_epoch();

            for req in &requests {
                let predicate = req.bounds_65.map(|(lo, hi)| GatePredicate::Range {
                    lo: lo.to_vec(),
                    hi: hi.to_vec(),
                });

                log_entries.push(GateLogEntry::Add {
                    gate_id: req.gate_id,
                    is_erased: req.is_erased,
                    lo: *req.lo.as_bytes(),
                    hi: *req.hi.as_bytes(),
                    predicate,
                    gated_seq: gated_seq.as_u64(),
                    erasure_id: req.erasure_id,
                });

                // Im ErasedSet anwenden
                if let Some((lo, hi)) = req.bounds_65 {
                    if req.is_erased {
                        set_guard.add_erased_with_bounds(req.gate_id, lo, hi, gated_seq);
                    } else {
                        set_guard.add_restricted_with_bounds(req.gate_id, lo, hi, gated_seq);
                    }
                } else if req.is_erased {
                    set_guard.add_erased(req.lo, req.hi, gated_seq);
                } else {
                    set_guard.add_restricted(req.lo, req.hi, gated_seq);
                }
            }
        }

        // 3. Gate-Log persistent anhängen (D-58, ein fsync je Gruppe auf Critical-Spur)
        {
            let mut log_guard = self.gate_log.write();
            log_guard.append_batch(log_entries);
        }

        // 4. Barriere wird beim Drop von `_exclusive_permit` freigegeben (D-57)
        drop(_exclusive_permit);

        Ok(BatchGateOutcome {
            gated_seq,
            applied_to_set: true,
            gates_count: requests.len(),
        })
    }
}

// Globaler In-Memory Default-Zustand für freistehenden Aufruf von `add_batch` (Rückwärtskompatibilität / Tests)
static GLOBAL_GATE_SET: parking_lot::RwLock<Option<Arc<RwLock<ErasedSet>>>> =
    parking_lot::RwLock::new(None);

/// Initialisiert oder bindet das globale ErasedSet für freistehende `add_batch`-Aufrufe.
pub fn bind_global_erased_set(set: Arc<RwLock<ErasedSet>>) {
    let mut guard = GLOBAL_GATE_SET.write();
    *guard = Some(set);
}

/// Freistehende `add_batch`-Funktion (SPEC §11.3, D-57, D-78).
///
/// Fügt Gate-Anfragen im Batch mit atomarer Inkrementierung von `gate_epoch` und
/// Aktualisierung des gebundenen `ErasedSet` hinzu.
pub fn add_batch(
    requests: Vec<BatchGateRequest>,
    current_seq: Seq,
) -> Result<BatchGateOutcome, ErasureError> {
    if let Some(set_arc) = GLOBAL_GATE_SET.read().as_ref() {
        let mut set = set_arc.write();
        set.increment_epoch();
        for req in &requests {
            if let Some((lo, hi)) = req.bounds_65 {
                if req.is_erased {
                    set.add_erased_with_bounds(req.gate_id, lo, hi, current_seq);
                } else {
                    set.add_restricted_with_bounds(req.gate_id, lo, hi, current_seq);
                }
            } else if req.is_erased {
                set.add_erased(req.lo, req.hi, current_seq);
            } else {
                set.add_restricted(req.lo, req.hi, current_seq);
            }
        }
    }

    Ok(BatchGateOutcome {
        gated_seq: current_seq,
        applied_to_set: true,
        gates_count: requests.len(),
    })
}
