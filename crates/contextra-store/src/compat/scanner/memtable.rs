//! # MemTable-Scanner (Ring 1)
//!
//! Liest Eintraege aus MemTable-Strukturen ohne Panic-Gefahr fuer Snapshot-Suchen.
//!
//! ## Entkopplung
//! Hält [`StoreScanView`] statt `Arc<LsmStorage>`. Der Scan-Kontext wird über
//! [`LsmStorage::scan_view`] erzeugt — die einzige Stelle mit direkter Kopplung.

// FILE-CONTEXT
// ZWECK: Memtable-Scanner für LsmMemtable-Schicht (§12.7.4, INV-WITNESS-MINT-1).
// INVARIANTEN: liest Roh-Einträge (inkl. Tombstones & Versionen) ohne logische Aggregation.
// HOTSPOTS: [1-70]

#![forbid(unsafe_code)]

use super::runtime::ScannerRuntime;
use super::store_scan_view::StoreScanView;
use crate::lsm::LsmStorage;
use crate::memtable::MemTable;
use crate::tenant_codec::TenantKeyCodec;
use bytes::Bytes;
use contextra_ports::scanner::{LayerScanner, ScanSession, WitnessLayer};
use contextra_types::{RequestContext, Result};
use std::sync::Arc;

/// Scanner für die `LsmMemtable`-Schicht.
///
/// Hält [`StoreScanView`] statt `Arc<LsmStorage>` — keine direkte Kopplung an die
/// Speicher-Engine. Kann daher mit isolierten Test-Daten instanziiert werden.
pub struct MemtableScanner {
    view: StoreScanView,
    #[allow(dead_code)]
    runtime: ScannerRuntime,
}

impl MemtableScanner {
    /// Erzeugt einen neuen `MemtableScanner` aus einer [`StoreScanView`].
    pub fn new(view: StoreScanView) -> Self {
        Self::with_runtime(view, ScannerRuntime::capture())
    }

    /// Erzeugt einen neuen `MemtableScanner` aus einer [`StoreScanView`] mit expliziter Runtime.
    pub fn with_runtime(view: StoreScanView, runtime: ScannerRuntime) -> Self {
        Self { view, runtime }
    }

    /// Erzeugt einen `MemtableScanner` aus einem `Arc<LsmStorage>` (Kompatibilitätspfad).
    ///
    /// Intern wird [`LsmStorage::scan_view`] aufgerufen — die Kopplung bleibt auf
    /// diese Fabrikmethode beschränkt.
    pub fn from_storage_blocking(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Result<Self> {
        let view = runtime.block_on(async { Ok(storage.scan_view().await) })?;
        Ok(Self::with_runtime(view, runtime))
    }

    /// Führt die Positivkontrolle (Canary Check) in einer isolierten Scratch-Memtable aus.
    fn run_positive_control(&self, session: &mut ScanSession) {
        let canary_mt = MemTable::new();
        let canary_key = Bytes::from("t:canary:control_key");
        let canary_val = Bytes::from("canary_val");
        canary_mt.put(canary_key.clone(), canary_val, 1, 1);

        for (k, _v, seq, _tx) in canary_mt.iter() {
            if k == canary_key {
                let mut hasher = blake3::Hasher::new();
                hasher.update(k.as_ref());
                hasher.update(&seq.to_le_bytes());
                let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();

                session.record_control_match();
                session.record_unit(unit_hash, false);
            }
        }
    }
}

impl LayerScanner for MemtableScanner {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::LsmMemtable
    }

    fn scan_units(&self, session: &mut ScanSession, ctx: &RequestContext) -> Result<()> {
        // 1. Positivkontrolle ausführen
        self.run_positive_control(session);

        // 2. Tenant Scan-Präfix vorbereiten
        let codec = TenantKeyCodec::new(ctx.tenant_id);
        let target_prefix = codec.scan_prefix().to_vec();

        // 3. Alle MemTables aus der View iterieren (kein Lock auf LsmStorage nötig)
        for mt in self.view.all_memtables() {
            let entries = mt.iter();
            for (key, _val, seq_no, _tx) in entries {
                let mut hasher = blake3::Hasher::new();
                hasher.update(key.as_ref());
                hasher.update(&seq_no.to_le_bytes());
                let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();

                let is_canary = key.starts_with(b"t:canary:")
                    || key
                        .windows(b":scratch_canary_pctl:".len())
                        .any(|w| w == b":scratch_canary_pctl:");
                let is_residual = key.starts_with(&target_prefix) && !is_canary;
                session.record_unit(unit_hash, is_residual);
            }
        }

        Ok(())
    }
}
