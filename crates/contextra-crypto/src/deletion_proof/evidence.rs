//! Schichtspezifische Evidenz-Typen und Invarianten-Prüfungen gemäß Spezifikation C3.

#![forbid(unsafe_code)]

use crate::deletion_proof::{canon::TlvEncoder, claim_text::WitAssurance};
use serde::Serialize;
use std::fmt;

/// Fehler bei der Invarianten-Prüfung einer Schicht-Evidenz.
/// Nennt Schicht und Feldname, NIEMALS vertrauliche Datenbytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitInvariantViolation {
    /// Name der betroffenen Schicht.
    pub layer_name: &'static str,
    /// Name des verletzten Feldes oder der Invariante.
    pub field_name: &'static str,
    /// Beschreibung der Verletzung.
    pub details: String,
}

impl fmt::Display for WitInvariantViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WitInvariantViolation in layer '{}', field '{}': {}",
            self.layer_name, self.field_name, self.details
        )
    }
}

impl std::error::Error for WitInvariantViolation {}

/// Evidenz-Daten für eine spezifische Storage-Schicht gemäß Spezifikation C3.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum WitLayerEvidence {
    /// In-Memory Memtable Scan Evidenz.
    Memtable {
        /// Anzahl gescannter Memtables.
        memtables_scanned: u32,
        /// Gefundene Einträge.
        entries_found: u64,
        /// Davon gefundene Tombstones.
        tombstones_found: u64,
    },
    /// Persistent SSTables Scan Evidenz.
    SsTables {
        /// Manifest Generation.
        manifest_gen: u64,
        /// In Manifest verzeichnete SSTable-Dateien.
        files_in_manifest: u32,
        /// Gescannte SSTable-Dateien.
        files_scanned: u32,
        /// Pruned Dateien mit verifizierter Range.
        files_pruned_with_verified_range: u32,
        /// Gescannte Datenblöcke.
        blocks_scanned: u64,
        /// Gescannte Gesamtbytes.
        bytes_scanned: u64,
        /// Gefundene verwaiste SSTable-Dateien (muss 0 sein).
        orphan_files_found: u32,
        /// Unklassifizierte Dateien (muss 0 sein).
        unclassified_files: u32,
        /// Gefundene Live-Wertversionen.
        versions_found: u64,
        /// Index-Hits für Schlüsselbezeichner.
        identifier_hits_index: u64,
        /// Verbleibende Tombstones.
        tombstones_remaining: u64,
    },
    /// Manifest Journal / State Evidenz.
    Manifest {
        /// Aktuelle Manifest Generation.
        current_manifest_gen: u64,
        /// In-Flight Commits (muss 0 sein).
        in_flight_commits: u32,
        /// Unpersisted Edits (muss 0 sein).
        unpersisted_edits: u32,
    },
    /// Write-Ahead Log Evidenz.
    Wal {
        /// Gescannte WAL-Segmente.
        segments_scanned: u32,
        /// Gesamte WAL-Segmente.
        segments_total: u32,
        /// Entfernte/bereinigte WAL-Segmente.
        segments_removed: u32,
        /// Bereinigte/überschriebene Bytes.
        bytes_scrubbed: u64,
        /// Verbleibende unbereinigte Records (muss 0 sein).
        unscrubbed_records_found: u64,
    },
    /// HNSW Vektorindex Evidenz.
    HnswIndex {
        /// Knoten-ID.
        node_id: contextra_types::DocIdRaw,
        /// Inbound Edges (muss 0 sein).
        incoming_edges: u32,
        /// Outbound Edges.
        outgoing_edges: u32,
        /// Von Lesern gepinnt (muss false sein).
        pinned_by_reader: bool,
        /// In ZeroOverlay markiert.
        marked_zeroed: bool,
    },
    /// Text / Full-Text Index Evidenz.
    TextIndex {
        /// Gescannte Postings Lists.
        postings_lists_scanned: u32,
        /// Gefundene Term-Dokument-Paare (muss 0 sein).
        term_doc_hits: u64,
    },
    /// Graph / Relational Index Evidenz.
    Relations {
        /// Gescannte Kanten.
        edges_scanned: u64,
        /// Inbound/Outbound Referenzen (muss 0 sein).
        matching_edges_found: u64,
    },
    /// Process In-Memory Cache Evidenz.
    ProcessCaches {
        /// Gescannte Cache-Segmente.
        cache_segments_scanned: u32,
        /// Aktive Referenzen (muss 0 sein).
        active_references: u32,
        /// Invalidation In-Flight (muss false sein).
        invalidation_in_flight: bool,
    },
    /// Checkpoint Clones Evidenz.
    CheckpointClones {
        /// Gescannte Checkpoints.
        checkpoints_scanned: u32,
        /// Gefundene Snapshot Clones (muss 0 sein).
        clones_found: u32,
    },
    /// Keyring / Encryption Key Evidenz.
    Keyring {
        /// Epoch ID des Keys.
        kek_epoch: u64,
        /// Ob DEK gelöscht/nicht mehr im Speicher (muss false/destroyed sein, dek_present == false).
        dek_present: bool,
        /// Ob KEK-Epoch vernichtet.
        kek_epoch_destroyed: bool,
    },
    /// Erasure Ledger Evidenz.
    ErasureLedger {
        /// Head Sequenznummer des Ledgers.
        ledger_head_seq: u64,
        /// BLAKE3 Chain MAC Hash der Löschentscheidung.
        decision_chain_mac: [u8; 32],
        /// Bestätigter Eintrag im Ledger.
        entry_confirmed: bool,
    },
}

impl WitLayerEvidence {
    /// Prüft die Invarianten der Schicht-Evidenz gemäß Spezifikation C3.
    pub fn check_invariants(
        &self,
        assurance: WitAssurance,
        key_identifiers_claimed: bool,
    ) -> Result<(), WitInvariantViolation> {
        match self {
            Self::Memtable {
                entries_found,
                tombstones_found,
                ..
            } => {
                if *entries_found > *tombstones_found {
                    return Err(WitInvariantViolation {
                        layer_name: "LsmMemtable",
                        field_name: "entries_found",
                        details: format!(
                            "live value entries found in memtable ({}) exceed tombstones ({})",
                            entries_found, tombstones_found
                        ),
                    });
                }
            }
            Self::SsTables {
                files_in_manifest,
                files_scanned,
                files_pruned_with_verified_range,
                orphan_files_found,
                unclassified_files,
                versions_found,
                identifier_hits_index,
                tombstones_remaining,
                ..
            } => {
                if *files_scanned + *files_pruned_with_verified_range != *files_in_manifest {
                    return Err(WitInvariantViolation {
                        layer_name: "SsTableAllLevels",
                        field_name: "files_scanned",
                        details: format!(
                            "files_scanned ({}) + files_pruned ({}) != files_in_manifest ({})",
                            files_scanned, files_pruned_with_verified_range, files_in_manifest
                        ),
                    });
                }
                if *orphan_files_found != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "SsTableAllLevels",
                        field_name: "orphan_files_found",
                        details: format!("orphan files found: {}", orphan_files_found),
                    });
                }
                if *unclassified_files != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "SsTableAllLevels",
                        field_name: "unclassified_files",
                        details: format!("unclassified files found: {}", unclassified_files),
                    });
                }
                if *versions_found != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "SsTableAllLevels",
                        field_name: "versions_found",
                        details: format!("value versions found in SSTables: {}", versions_found),
                    });
                }
                if (key_identifiers_claimed || assurance >= WitAssurance::S2)
                    && *identifier_hits_index != 0
                {
                    return Err(WitInvariantViolation {
                        layer_name: "SsTableAllLevels",
                        field_name: "identifier_hits_index",
                        details: format!(
                            "identifier index hits remaining: {}",
                            identifier_hits_index
                        ),
                    });
                }
                if key_identifiers_claimed && *tombstones_remaining != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "SsTableAllLevels",
                        field_name: "tombstones_remaining",
                        details: format!(
                            "tombstones remaining when key identifiers claimed: {}",
                            tombstones_remaining
                        ),
                    });
                }
            }
            Self::Manifest {
                in_flight_commits,
                unpersisted_edits,
                ..
            } => {
                if *in_flight_commits != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "Manifest",
                        field_name: "in_flight_commits",
                        details: format!("in-flight commits in manifest: {}", in_flight_commits),
                    });
                }
                if *unpersisted_edits != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "Manifest",
                        field_name: "unpersisted_edits",
                        details: format!("unpersisted edits in manifest: {}", unpersisted_edits),
                    });
                }
            }
            Self::Wal {
                segments_scanned,
                segments_total,
                segments_removed,
                unscrubbed_records_found,
                ..
            } => {
                if *segments_scanned != *segments_total - *segments_removed {
                    return Err(WitInvariantViolation {
                        layer_name: "WalAllSegments",
                        field_name: "segments_scanned",
                        details: format!(
                            "segments_scanned ({}) != segments_total ({}) - segments_removed ({})",
                            segments_scanned, segments_total, segments_removed
                        ),
                    });
                }
                if *unscrubbed_records_found != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "WalAllSegments",
                        field_name: "unscrubbed_records_found",
                        details: format!(
                            "unscrubbed records found in WAL: {}",
                            unscrubbed_records_found
                        ),
                    });
                }
            }
            Self::HnswIndex {
                incoming_edges,
                pinned_by_reader,
                ..
            } => {
                if *incoming_edges != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "HnswIndex",
                        field_name: "incoming_edges",
                        details: format!(
                            "incoming ghost edges remaining in HNSW index: {}",
                            incoming_edges
                        ),
                    });
                }
                if *pinned_by_reader {
                    return Err(WitInvariantViolation {
                        layer_name: "HnswIndex",
                        field_name: "pinned_by_reader",
                        details: "vector node is currently pinned by active reader".to_string(),
                    });
                }
            }
            Self::TextIndex { term_doc_hits, .. } => {
                if *term_doc_hits != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "TextIndex",
                        field_name: "term_doc_hits",
                        details: format!(
                            "term-document hits remaining in TextIndex: {}",
                            term_doc_hits
                        ),
                    });
                }
            }
            Self::Relations {
                matching_edges_found,
                ..
            } => {
                if *matching_edges_found != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "Relations",
                        field_name: "matching_edges_found",
                        details: format!(
                            "matching relational edges remaining: {}",
                            matching_edges_found
                        ),
                    });
                }
            }
            Self::ProcessCaches {
                active_references,
                invalidation_in_flight,
                ..
            } => {
                if *active_references != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "ProcessCaches",
                        field_name: "active_references",
                        details: format!(
                            "active cache references remaining: {}",
                            active_references
                        ),
                    });
                }
                if *invalidation_in_flight {
                    return Err(WitInvariantViolation {
                        layer_name: "ProcessCaches",
                        field_name: "invalidation_in_flight",
                        details: "cache invalidation in flight".to_string(),
                    });
                }
            }
            Self::CheckpointClones { clones_found, .. } => {
                if *clones_found != 0 {
                    return Err(WitInvariantViolation {
                        layer_name: "CheckpointClones",
                        field_name: "clones_found",
                        details: format!("snapshot clones found in checkpoints: {}", clones_found),
                    });
                }
            }
            Self::Keyring {
                dek_present,
                kek_epoch_destroyed,
                ..
            } => {
                if *dek_present {
                    return Err(WitInvariantViolation {
                        layer_name: "Keyring",
                        field_name: "dek_present",
                        details: "DEK is still present in keyring memory".to_string(),
                    });
                }
                if !*kek_epoch_destroyed {
                    return Err(WitInvariantViolation {
                        layer_name: "Keyring",
                        field_name: "kek_epoch_destroyed",
                        details: "KEK epoch was not marked as destroyed".to_string(),
                    });
                }
            }
            Self::ErasureLedger {
                entry_confirmed, ..
            } => {
                if !*entry_confirmed {
                    return Err(WitInvariantViolation {
                        layer_name: "ErasureLedger",
                        field_name: "entry_confirmed",
                        details: "erasure decision entry was not confirmed in ledger".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Kodiert die Evidenz deterministisch in ein TLV-Format mit festen Tags.
    pub fn to_canonical_tlv(&self) -> Vec<u8> {
        let mut encoder = TlvEncoder::new();

        match self {
            Self::Memtable {
                memtables_scanned,
                entries_found,
                tombstones_found,
            } => {
                encoder.add_u16(0x0001, 1); // Variant tag for Memtable
                encoder.add_u32(0x0002, *memtables_scanned);
                encoder.add_u64(0x0003, *entries_found);
                encoder.add_u64(0x0004, *tombstones_found);
            }
            Self::SsTables {
                manifest_gen,
                files_in_manifest,
                files_scanned,
                files_pruned_with_verified_range,
                blocks_scanned,
                bytes_scanned,
                orphan_files_found,
                unclassified_files,
                versions_found,
                identifier_hits_index,
                tombstones_remaining,
            } => {
                encoder.add_u16(0x0001, 2); // Variant tag for SsTables
                encoder.add_u64(0x0002, *manifest_gen);
                encoder.add_u32(0x0003, *files_in_manifest);
                encoder.add_u32(0x0004, *files_scanned);
                encoder.add_u32(0x0005, *files_pruned_with_verified_range);
                encoder.add_u64(0x0006, *blocks_scanned);
                encoder.add_u64(0x0007, *bytes_scanned);
                encoder.add_u32(0x0008, *orphan_files_found);
                encoder.add_u32(0x0009, *unclassified_files);
                encoder.add_u64(0x000A, *versions_found);
                encoder.add_u64(0x000B, *identifier_hits_index);
                encoder.add_u64(0x000C, *tombstones_remaining);
            }
            Self::Manifest {
                current_manifest_gen,
                in_flight_commits,
                unpersisted_edits,
            } => {
                encoder.add_u16(0x0001, 3); // Variant tag for Manifest
                encoder.add_u64(0x0002, *current_manifest_gen);
                encoder.add_u32(0x0003, *in_flight_commits);
                encoder.add_u32(0x0004, *unpersisted_edits);
            }
            Self::Wal {
                segments_scanned,
                segments_total,
                segments_removed,
                bytes_scrubbed,
                unscrubbed_records_found,
            } => {
                encoder.add_u16(0x0001, 4); // Variant tag for Wal
                encoder.add_u32(0x0002, *segments_scanned);
                encoder.add_u32(0x0003, *segments_total);
                encoder.add_u32(0x0004, *segments_removed);
                encoder.add_u64(0x0005, *bytes_scrubbed);
                encoder.add_u64(0x0006, *unscrubbed_records_found);
            }
            Self::HnswIndex {
                node_id,
                incoming_edges,
                outgoing_edges,
                pinned_by_reader,
                marked_zeroed,
            } => {
                encoder.add_u16(0x0001, 5); // Variant tag for HnswIndex
                encoder.add_field(0x0002, node_id.to_be_bytes().to_vec());
                encoder.add_u32(0x0003, *incoming_edges);
                encoder.add_u32(0x0004, *outgoing_edges);
                encoder.add_bool(0x0005, *pinned_by_reader);
                encoder.add_bool(0x0006, *marked_zeroed);
            }
            Self::TextIndex {
                postings_lists_scanned,
                term_doc_hits,
            } => {
                encoder.add_u16(0x0001, 6); // Variant tag for TextIndex
                encoder.add_u32(0x0002, *postings_lists_scanned);
                encoder.add_u64(0x0003, *term_doc_hits);
            }
            Self::Relations {
                edges_scanned,
                matching_edges_found,
            } => {
                encoder.add_u16(0x0001, 7); // Variant tag for Relations
                encoder.add_u64(0x0002, *edges_scanned);
                encoder.add_u64(0x0003, *matching_edges_found);
            }
            Self::ProcessCaches {
                cache_segments_scanned,
                active_references,
                invalidation_in_flight,
            } => {
                encoder.add_u16(0x0001, 8); // Variant tag for ProcessCaches
                encoder.add_u32(0x0002, *cache_segments_scanned);
                encoder.add_u32(0x0003, *active_references);
                encoder.add_bool(0x0004, *invalidation_in_flight);
            }
            Self::CheckpointClones {
                checkpoints_scanned,
                clones_found,
            } => {
                encoder.add_u16(0x0001, 9); // Variant tag for CheckpointClones
                encoder.add_u32(0x0002, *checkpoints_scanned);
                encoder.add_u32(0x0003, *clones_found);
            }
            Self::Keyring {
                kek_epoch,
                dek_present,
                kek_epoch_destroyed,
            } => {
                encoder.add_u16(0x0001, 10); // Variant tag for Keyring
                encoder.add_u64(0x0002, *kek_epoch);
                encoder.add_bool(0x0003, *dek_present);
                encoder.add_bool(0x0004, *kek_epoch_destroyed);
            }
            Self::ErasureLedger {
                ledger_head_seq,
                decision_chain_mac,
                entry_confirmed,
            } => {
                encoder.add_u16(0x0001, 11); // Variant tag for ErasureLedger
                encoder.add_u64(0x0002, *ledger_head_seq);
                encoder.add_bytes32(0x0003, decision_chain_mac);
                encoder.add_bool(0x0004, *entry_confirmed);
            }
        }

        encoder.finish()
    }

    /// Berechnet den BLAKE3 Witness-Digest der Evidenz:
    /// BLAKE3("contextra/witness/v2" || layer_id (u16 BE) || tlv_bytes)
    pub fn witness_digest(&self, layer_id: u16) -> [u8; 32] {
        let tlv = self.to_canonical_tlv();
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"contextra/witness/v2");
        hasher.update(&layer_id.to_be_bytes());
        hasher.update(&tlv);
        *hasher.finalize().as_bytes()
    }
}
