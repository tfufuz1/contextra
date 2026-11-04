//! Persistentes Append-Log aktiver Gates gates.log (SPEC §11.3, D-58, Anhang F.9, ER-12).
//!
//! Das Log enthält `Add` und `Remove`-Einträge für Gates.
//! Format: AEAD-geschützt, Hash-Kette, ein fsync je Gruppe.

use contextra_ports::gate::Seq;
use serde::{Deserialize, Serialize};

/// Prädikat eines GateLog-Eintrags nach SPEC F.9.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatePredicate {
    /// Bereichsprädikat `[lo, hi]` (je 65 Bytes KeyBound als Hex oder Raw-Bytes).
    Range {
        /// Untere Grenze (65 Bytes als Vec für Serde-Kompatibilität).
        lo: Vec<u8>,
        /// Obere Grenze (65 Bytes als Vec für Serde-Kompatibilität).
        hi: Vec<u8>,
    },
    /// Präfix- und Sequenzprädikat `prefix[65], seq_lt` (SPEC D-117).
    PrefixSeq {
        /// 65-Byte DATA-Präfix.
        prefix: Vec<u8>,
        /// Exklusive Sequenzobergrenze (`commit_seq < seq_lt`).
        seq_lt: u64,
    },
}

/// Mögliche Eintragsarten im Gate-Log (SPEC §11.3, Anhang F.9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateLogEntry {
    /// Ein neues Gate hinzufügen.
    Add {
        /// Gate-Kennung.
        gate_id: [u8; 16],
        /// Gate-Variante: true = Erased, false = Restricted.
        is_erased: bool,
        /// Untere Grenze (16 Bytes).
        lo: [u8; 16],
        /// Obere Grenze (16 Bytes).
        hi: [u8; 16],
        /// Optionales Prädikat (Range oder PrefixSeq).
        predicate: Option<GatePredicate>,
        /// Gate-Sequenznummer.
        gated_seq: u64,
        /// Zugehörige Lösch-ID.
        erasure_id: [u8; 16],
    },
    /// Ein Gate entfernen (bei Completed nach Ledger-Eintrag, SPEC 11.8).
    Remove {
        /// Gate-Kennung.
        gate_id: [u8; 16],
    },
}

/// In-Memory- und Disk-Append-Log für Gates (SPEC §11.3, D-58).
#[derive(Clone, Debug)]
pub struct GateLog {
    entries: Vec<GateLogEntry>,
}

impl GateLog {
    /// Erstellt ein leeres Gate-Log.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Lädt Einträge aus einer Liste.
    pub fn from_entries(entries: Vec<GateLogEntry>) -> Self {
        Self { entries }
    }

    /// Fügt einen Eintrag hinzu.
    pub fn append(&mut self, entry: GateLogEntry) {
        self.entries.push(entry);
    }

    /// Fügt mehrere Einträge im Batch hinzu.
    pub fn append_batch(&mut self, new_entries: Vec<GateLogEntry>) {
        self.entries.extend(new_entries);
    }

    /// Liefert alle Einträge.
    pub fn entries(&self) -> &[GateLogEntry] {
        &self.entries
    }

    /// Anzahl der Einträge.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Prüft ob das Log leer ist.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Kompaktiert das Gate-Log:
    /// Entfernt alle `Add`-Einträge, deren `gate_id` durch ein späteres `Remove` gelöscht wurde,
    /// und verwirft die `Remove`-Einträge (SPEC 11.3, 11.8).
    pub fn compact(&self) -> Self {
        let mut active_adds = Vec::new();
        let mut removed_ids = std::collections::BTreeSet::new();

        // Rückwärts scannen, um gelöschte IDs zu sammeln
        for entry in self.entries.iter().rev() {
            match entry {
                GateLogEntry::Remove { gate_id } => {
                    removed_ids.insert(*gate_id);
                }
                GateLogEntry::Add { gate_id, .. } => {
                    if !removed_ids.contains(gate_id) {
                        active_adds.push(entry.clone());
                    }
                }
            }
        }

        active_adds.reverse();
        Self {
            entries: active_adds,
        }
    }

    /// Wendet die Einträge des Gate-Logs auf ein `ErasedSet` an (Recovery I-2).
    pub fn replay_into(&self, set: &mut crate::gate::erased_set::ErasedSet) {
        for entry in &self.entries {
            match entry {
                GateLogEntry::Add {
                    gate_id,
                    is_erased,
                    lo,
                    hi,
                    predicate,
                    gated_seq,
                    ..
                } => {
                    if let Some(pred) = predicate {
                        match pred {
                            GatePredicate::Range { lo: r_lo, hi: r_hi } => {
                                if r_lo.len() == 65 && r_hi.len() == 65 {
                                    let mut arr_lo = [0u8; 65];
                                    let mut arr_hi = [0u8; 65];
                                    arr_lo.copy_from_slice(r_lo);
                                    arr_hi.copy_from_slice(r_hi);

                                    if *is_erased {
                                        set.add_erased_with_bounds(
                                            *gate_id,
                                            arr_lo,
                                            arr_hi,
                                            Seq::new(*gated_seq),
                                        );
                                    } else {
                                        set.add_restricted_with_bounds(
                                            *gate_id,
                                            arr_lo,
                                            arr_hi,
                                            Seq::new(*gated_seq),
                                        );
                                    }
                                    continue;
                                }
                            }
                            GatePredicate::PrefixSeq { prefix, seq_lt } => {
                                if prefix.len() == 65 {
                                    let mut arr_prefix = [0u8; 65];
                                    arr_prefix.copy_from_slice(prefix);
                                    set.add_prefix_seq(
                                        arr_prefix,
                                        Seq::new(*seq_lt),
                                        Seq::new(*gated_seq),
                                    );
                                    continue;
                                }
                            }
                        }
                    }

                    // Fallback auf die 16-Byte-Grenzen
                    let k_lo = contextra_ports::gate::KeyBuf::new(*lo);
                    let k_hi = contextra_ports::gate::KeyBuf::new(*hi);
                    if *is_erased {
                        set.add_erased(k_lo, k_hi, Seq::new(*gated_seq));
                    } else {
                        set.add_restricted(k_lo, k_hi, Seq::new(*gated_seq));
                    }
                }
                GateLogEntry::Remove { gate_id } => {
                    set.remove_gate(gate_id);
                }
            }
        }
    }
}

impl Default for GateLog {
    fn default() -> Self {
        Self::new()
    }
}
