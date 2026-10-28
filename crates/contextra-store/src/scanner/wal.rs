// FILE-CONTEXT
// ZWECK: WAL-Scanner für WalAllSegments-Schicht (§12.7.4, INV-WITNESS-MINT-1).
// INVARIANTEN: dekodiert alle WAL-Segmente inklusive HMAC-Prüfung und misst Rohdaten.
// HOTSPOTS: [1-90]

#![forbid(unsafe_code)]

use super::sstable::block_on_future;
use crate::lsm::LsmStorage;
use crate::tenant_codec::TenantKeyCodec;
use crate::wal::{Wal, WalConfig, WalOp};
use contextra_ports::scanner::{LayerScanner, ScanSession, ScanTechnique, WitnessLayer};
use contextra_types::{RequestContext, Result, TxId};
use std::sync::Arc;

/// Scanner für die `WalAllSegments`-Schicht.
pub struct WalScanner {
    storage: Arc<LsmStorage>,
}

impl WalScanner {
    /// Erzeugt einen neuen `WalScanner`.
    pub fn new(storage: Arc<LsmStorage>) -> Self {
        Self { storage }
    }

    /// Positivkontrolle für WAL-Segmente: Schreiben, Replay und Löschen
    /// einer temporären Canary-WAL-Datei.
    fn run_positive_control(&self, session: &mut ScanSession) {
        block_on_future(async {
            let temp_dir = std::env::temp_dir();
            let canary_path = temp_dir.join(format!("canary_{}.log", uuid::Uuid::new_v4()));
            let canary_key = b"t:canary:1:chunk:1";
            let canary_val = b"canary_val";

            if let Ok(wal) = Wal::open_with_config(&canary_path, WalConfig::default()).await {
                let op = WalOp::Put {
                    key: canary_key.to_vec(),
                    value: canary_val.to_vec(),
                    tx_id: TxId(1),
                };
                if let Ok((batch, _)) = wal.prepare_batch(vec![(op, 1)]).await {
                    let _ = wal.append_batch(batch).await;
                }

                if let Ok(reader_wal) = Wal::open_read_only(&canary_path, None).await {
                    if let Ok(entries) = reader_wal.replay().await {
                        for (seq, entry, _pos) in entries {
                            if let WalOp::Put { key, .. } = &entry.op {
                                if key == canary_key {
                                    let mut hasher = blake3::Hasher::new();
                                    hasher.update(key);
                                    hasher.update(&seq.to_le_bytes());
                                    let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();

                                    session.record_control_match();
                                    session.record_unit(unit_hash, false);
                                }
                            }
                        }
                    }
                }
            }
            let _ = tokio::fs::remove_file(&canary_path).await;
        });
    }
}

impl LayerScanner for WalScanner {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::WalAllSegments
    }

    fn technique(&self) -> ScanTechnique {
        ScanTechnique::DecryptedViewScan
    }

    fn scan_units(&self, session: &mut ScanSession, ctx: &RequestContext) -> Result<()> {
        self.run_positive_control(session);

        let codec = TenantKeyCodec::new(ctx.tenant_id);
        let target_prefix = codec.scan_prefix();

        block_on_future(async {
            let data_dir = self.storage.data_path();
            let mut wal_files = Vec::new();

            if let Ok(mut dir) = tokio::fs::read_dir(data_dir).await {
                while let Ok(Some(entry)) = dir.next_entry().await {
                    let path = entry.path();
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    let is_wal = (name.starts_with("wal-") && name.ends_with(".log"))
                        || name == "wal.log"
                        || name.ends_with(".wal");
                    if is_wal {
                        wal_files.push(path);
                    }
                }
            }

            let mut max_seq = 0u64;

            for wal_path in wal_files {
                if let Ok(wal) = Wal::open_read_only(&wal_path, None).await {
                    if let Ok(entries) = wal.replay().await {
                        for (seq, entry, _pos) in entries {
                            if seq > max_seq {
                                max_seq = seq;
                            }
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

                                let is_residual = key.starts_with(target_prefix);
                                session.record_unit(unit_hash, is_residual);
                            }
                        }
                    }
                }
            }

            session.set_wal_seq_after(max_seq);
        });

        Ok(())
    }
}
