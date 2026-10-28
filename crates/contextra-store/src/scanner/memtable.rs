// FILE-CONTEXT
// ZWECK: Memtable-Scanner für LsmMemtable-Schicht (§12.7.4, INV-WITNESS-MINT-1).
// INVARIANTEN: liest Roh-Einträge (inkl. Tombstones & Versionen) ohne logische Aggregation.
// HOTSPOTS: [1-70]

#![forbid(unsafe_code)]

use super::sstable::block_on_future;
use crate::lsm::LsmStorage;
use crate::memtable::MemTable;
use crate::tenant_codec::TenantKeyCodec;
use bytes::Bytes;
use contextra_ports::scanner::{LayerScanner, ScanSession, WitnessLayer};
use contextra_types::{RequestContext, Result};
use std::sync::Arc;

/// Scanner für die `LsmMemtable`-Schicht.
pub struct MemtableScanner {
    storage: Arc<LsmStorage>,
}

impl MemtableScanner {
    /// Erzeugt einen neuen `MemtableScanner`.
    pub fn new(storage: Arc<LsmStorage>) -> Self {
        Self { storage }
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
        let target_prefix = codec.scan_prefix();

        // 3. Aktive und immutable Memtables erfassen
        let (active_mt, immutable_mts) = block_on_future(async {
            self.storage.memtables_for_scanner().await
        });

        let mut all_mts = Vec::with_capacity(1 + immutable_mts.len());
        all_mts.push(active_mt);
        all_mts.extend(immutable_mts);

        // 4. Alle Memtables auf Rohebene iterieren
        for mt in all_mts {
            let entries = mt.iter();
            for (key, _val, seq_no, _tx) in entries {
                let mut hasher = blake3::Hasher::new();
                hasher.update(key.as_ref());
                hasher.update(&seq_no.to_le_bytes());
                let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();

                let is_residual = key.starts_with(target_prefix);
                session.record_unit(unit_hash, is_residual);
            }
        }

        Ok(())
    }
}
