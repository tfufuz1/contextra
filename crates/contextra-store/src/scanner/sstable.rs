// FILE-CONTEXT
// ZWECK: SSTable-Scanner für SsTableAllLevels-Schicht (§12.7.4, INV-WITNESS-MINT-1).
// INVARIANTEN: liest Roh-SSTables und Waisen-Dateien (Altdateien) via Block-Reader statt Merge-Iterator.
// HOTSPOTS: [1-100]

#![forbid(unsafe_code)]

use crate::lsm::LsmStorage;
use crate::sstable::{SstableBuilder, SstableReader};
use crate::tenant_codec::TenantKeyCodec;
use contextra_ports::scanner::{LayerScanner, ScanSession, ScanTechnique, WitnessLayer};
use contextra_types::{RequestContext, Result};
use std::collections::HashSet;
use std::sync::Arc;

pub(crate) fn block_on_future<F: std::future::Future>(fut: F) -> F::Output
where
    F: Send,
    F::Output: Send,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
            tokio::task::block_in_place(|| handle.block_on(fut))
        } else {
            let res = std::thread::scope(|s| {
                s.spawn(|| {
                    let mut builder = tokio::runtime::Builder::new_current_thread();
                    builder.enable_all();
                    if let Ok(rt) = builder.build() {
                        rt.block_on(fut)
                    } else {
                        handle.block_on(fut)
                    }
                })
                .join()
            });
            match res {
                Ok(val) => val,
                Err(e) => std::panic::resume_unwind(e),
            }
        }
    } else {
        let res = std::thread::scope(|s| {
            s.spawn(|| {
                let mut builder = tokio::runtime::Builder::new_current_thread();
                builder.enable_all();
                if let Ok(rt) = builder.build() {
                    rt.block_on(fut)
                } else if let Ok(rt2) = tokio::runtime::Runtime::new() {
                    rt2.block_on(fut)
                } else {
                    let mut b3 = tokio::runtime::Builder::new_current_thread();
                    b3.enable_all();
                    if let Ok(rt3) = b3.build() {
                        rt3.block_on(fut)
                    } else {
                        handle_runtime_error()
                    }
                }
            })
            .join()
        });
        match res {
            Ok(val) => val,
            Err(e) => std::panic::resume_unwind(e),
        }
    }
}

fn handle_runtime_error() -> ! {
    loop {
        std::thread::park();
    }
}

/// Scanner für die `SsTableAllLevels`-Schicht.
pub struct SstableScanner {
    storage: Arc<LsmStorage>,
}

impl SstableScanner {
    /// Erzeugt einen neuen `SstableScanner`.
    pub fn new(storage: Arc<LsmStorage>) -> Self {
        Self { storage }
    }

    /// Positivkontrolle für SSTables: Erzeugt eine temporäre Canary-Datei,
    /// führt die volle Reader-Iterationslogik darüber aus, verifiziert den Treffer
    /// und löscht die Canary-Datei rückstandslos.
    fn run_positive_control(&self, session: &mut ScanSession) {
        block_on_future(async {
            let temp_dir = std::env::temp_dir();
            let canary_path = temp_dir.join(format!("canary_{}.sst", uuid::Uuid::new_v4()));
            let canary_key = b"t:canary:1:chunk:1";
            let canary_val = b"canary_val";

            if let Ok(mut builder) = SstableBuilder::create(&canary_path).await {
                if builder.add(canary_key, canary_val, 1, 1).await.is_ok() {
                    if builder.finish().await.is_ok() {
                        let block_cache = self.storage.block_cache();
                        if let Ok(reader) =
                            SstableReader::open_with_key_manager(&canary_path, block_cache, None)
                                .await
                        {
                            if let Ok(entries) = reader.iter().await {
                                for (k, _v, seq) in entries {
                                    if k.as_ref() == canary_key {
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
                    }
                }
            }
            let _ = tokio::fs::remove_file(&canary_path).await;
        });
    }
}

impl LayerScanner for SstableScanner {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::SsTableAllLevels
    }

    fn technique(&self) -> ScanTechnique {
        ScanTechnique::RawFileScan
    }

    fn scan_units(&self, session: &mut ScanSession, ctx: &RequestContext) -> Result<()> {
        self.run_positive_control(session);

        let codec = TenantKeyCodec::new(ctx.tenant_id);
        let target_prefix = codec.scan_prefix();

        block_on_future(async {
            // 1. Offene / Aktive SSTables im Store erfassen
            let readers = self.storage.sstables_for_scanner().await;
            let mut active_paths = HashSet::new();

            for reader in &readers {
                active_paths.insert(reader.file_path().to_path_buf());

                // Metadaten- & Bloom-Pre-check vor Block-Scan
                let first_k = reader.first_key();
                let last_k = reader.last_key();

                let may_overlap = first_k.starts_with(target_prefix)
                    || last_k.starts_with(target_prefix)
                    || (first_k.as_ref() <= target_prefix && last_k.as_ref() >= target_prefix);

                if may_overlap || first_k.is_empty() {
                    if let Ok(entries) = reader.iter().await {
                        for (key, _val, seq_no) in entries {
                            let mut hasher = blake3::Hasher::new();
                            hasher.update(key.as_ref());
                            hasher.update(&seq_no.to_le_bytes());
                            let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();

                            let is_residual = key.starts_with(target_prefix);
                            session.record_unit(unit_hash, is_residual);
                        }
                    }
                }
            }

            // 2. Inventar-Diff: Suche nach Altdateien (.sst), die nicht im Manifest/Store stehen
            let data_dir = self.storage.data_path();
            if let Ok(mut dir) = tokio::fs::read_dir(data_dir).await {
                while let Ok(Some(entry)) = dir.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("sst")
                        && !active_paths.contains(&path)
                    {
                        // Orphan SSTable gefunden
                        let block_cache = self.storage.block_cache();
                        if let Ok(orphan_reader) =
                            SstableReader::open_with_key_manager(&path, block_cache, None).await
                        {
                            if let Ok(entries) = orphan_reader.iter().await {
                                for (key, _val, seq_no) in entries {
                                    let mut hasher = blake3::Hasher::new();
                                    hasher.update(key.as_ref());
                                    hasher.update(&seq_no.to_le_bytes());
                                    let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();

                                    let is_residual = key.starts_with(target_prefix);
                                    session.record_unit(unit_hash, is_residual);
                                }
                            }
                        } else {
                            // Wenn Datei nicht lesbar ist (z. B. unvollständig), zähle Datei-Hash als Rest
                            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                            let mut hasher = blake3::Hasher::new();
                            hasher.update(file_name.as_bytes());
                            let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();
                            session.record_unit(unit_hash, true);
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
