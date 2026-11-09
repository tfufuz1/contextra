//! # WAL-Scanner (Ring 1)
//!
//! Scannt sequentiell durch das WAL zur Integritaetspruefung oder gezielten Datenextraktion.

// FILE-CONTEXT
// ZWECK: WAL-Scanner für WalAllSegments-Schicht (§12.7.4, INV-WITNESS-MINT-1).
// INVARIANTEN: dekodiert alle WAL-Segmente inklusive HMAC-Prüfung und misst Rohdaten.
// HOTSPOTS: [1-90]

#![forbid(unsafe_code)]

use super::runtime::ScannerRuntime;
use crate::lsm::LsmStorage;
use crate::tenant_codec::TenantKeyCodec;
use crate::wal::{Wal, WalEntry, WalOp, WAL_V3_HEADER};
use contextra_ports::scanner::{LayerScanner, ScanSession, WitnessLayer};
use contextra_types::{RequestContext, Result, TxId};
use std::sync::Arc;

/// Scanner für die `WalAllSegments`-Schicht.
pub struct WalScanner {
    storage: Arc<LsmStorage>,
    runtime: ScannerRuntime,
}

impl WalScanner {
    /// Erzeugt einen neuen `WalScanner` mit implizit erfasster Laufzeit.
    pub fn new(storage: Arc<LsmStorage>) -> Self {
        Self::with_runtime(storage, ScannerRuntime::capture())
    }

    /// Erzeugt einen neuen `WalScanner` mit expliziter `ScannerRuntime`.
    pub fn with_runtime(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Self {
        Self { storage, runtime }
    }

    /// Positivkontrolle für WAL-Segmente: Schreiben, Replay und Entfernen
    /// einer temporären Canary-WAL-Datei.
    /// Schreibt direkt Frame-Bytes, um Background-Flusher-Tasks im Scanner zu vermeiden.
    fn run_positive_control(&self, session: &mut ScanSession) -> Result<()> {
        let mut control_matched = false;
        let mut control_unit: Option<[u8; 32]> = None;

        self.runtime.block_on(async {
            let temp_dir = std::env::temp_dir();
            let canary_path = temp_dir.join(format!("canary_{}_{}.log", std::process::id(), crate::lsm::recovery::next_temp_file_counter()));
            let canary_key = b"t:canary:1:chunk:1";
            let canary_val = b"canary_val";

            let integrity_key = Wal::load_or_create_integrity_key(&canary_path)
                .await
                .unwrap_or([0u8; 32]);

            let op = WalOp::Put {
                key: canary_key.to_vec(),
                value: canary_val.to_vec(),
                tx_id: TxId(1),
            };

            if let Ok(entry) = WalEntry::try_new(op, 1, &integrity_key, [0u8; 32]) {
                if let Ok(entry_bytes) = entry.to_bytes() {
                    let mut file_bytes =
                        Vec::with_capacity(WAL_V3_HEADER.len() + entry_bytes.len());
                    file_bytes.extend_from_slice(&WAL_V3_HEADER);
                    file_bytes.extend_from_slice(&entry_bytes);

                    if tokio::fs::write(&canary_path, file_bytes).await.is_ok() {
                        if let Ok(reader_wal) = Wal::open_read_only(&canary_path, None).await {
                            if let Ok(entries) = reader_wal.replay().await {
                                for (seq, entry, _pos) in entries {
                                    if let WalOp::Put { key, .. } = &entry.op {
                                        if key == canary_key {
                                            let mut hasher = blake3::Hasher::new();
                                            hasher.update(key);
                                            hasher.update(&seq.to_le_bytes());
                                            let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();

                                            control_matched = true;
                                            control_unit = Some(unit_hash);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let _ = tokio::fs::remove_file(&canary_path).await;
            Ok::<(), contextra_types::ContextraError>(())
        })?;

        if control_matched {
            session.record_control_match();
        }
        if let Some(hash) = control_unit {
            session.record_unit(hash, false);
        }

        Ok(())
    }
}

impl LayerScanner for WalScanner {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::WalAllSegments
    }

    fn scan_units(&self, session: &mut ScanSession, ctx: &RequestContext) -> Result<()> {
        self.run_positive_control(session)?;

        let codec = TenantKeyCodec::new(ctx.tenant_id);
        let target_prefix = codec.scan_prefix().to_vec();

        let storage = Arc::clone(&self.storage);

        let units = self.runtime.block_on(async {
            let data_dir = storage.data_path();
            let mut wal_files = Vec::new();

            if let Ok(dir_entries) = self.storage.vfs().list_dir(data_dir) {
                for name_str in dir_entries {
                    let path = data_dir.join(&name_str);
                    let is_wal = (name_str.starts_with("wal-") && name_str.ends_with(".log"))
                        || name_str == "wal.log"
                        || name_str.ends_with(".wal");
                    if is_wal {
                        wal_files.push(path);
                    }
                }
            }

            let mut discovered = Vec::new();

            for wal_path in wal_files {
                if let Ok(wal) = Wal::open_read_only(&wal_path, None).await {
                    if let Ok(entries) = wal.replay().await {
                        for (seq, entry, _pos) in entries {
                            let key_bytes = match &entry.op {
                                WalOp::Put { key, .. } => Some(key.as_slice()),
                                WalOp::Delete { key, .. } => Some(key.as_slice()),
                                WalOp::TxEnd { .. } => None,
                            };

                            if let Some(key) = key_bytes {
                                let mut hasher = blake3::Hasher::new();
                                hasher.update(key);
                                hasher.update(&seq.to_le_bytes());
                                let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();

                                let is_canary = key.starts_with(b"t:canary:")
                                    || key
                                        .windows(b":scratch_canary_pctl:".len())
                                        .any(|w| w == b":scratch_canary_pctl:");
                                let is_residual = key.starts_with(&target_prefix) && !is_canary;
                                discovered.push((unit_hash, is_residual));
                            }
                        }
                    }
                }
            }

            Ok::<Vec<([u8; 32], bool)>, contextra_types::ContextraError>(discovered)
        })?;

        for (unit_hash, is_residual) in units {
            session.record_unit(unit_hash, is_residual);
        }

        Ok(())
    }
}
