//! # SSTable-Scanner (Ring 1)
//!
//! Ermoeglicht geordnete Bereichsscans ueber eine oder mehrere SSTables.

// FILE-CONTEXT
// ZWECK: SSTable-Scanner für SsTableAllLevels-Schicht (§12.7.4, INV-WITNESS-MINT-1).
// INVARIANTEN: liest Roh-SSTables und Waisen-Dateien (Altdateien) via Block-Reader statt Merge-Iterator.
// HOTSPOTS: [1-100]

#![forbid(unsafe_code)]

use super::runtime::ScannerRuntime;
use crate::lsm::LsmStorage;
use crate::sstable::{SstableBuilder, SstableReader};
use crate::tenant_codec::TenantKeyCodec;
use contextra_ports::scanner::{LayerScanner, ScanSession, WitnessLayer};
use contextra_types::{RequestContext, Result};
use ahash::AHashSet;
use std::sync::Arc;

/// Scanner für die `SsTableAllLevels`-Schicht.
pub struct SstableScanner {
    storage: Arc<LsmStorage>,
    runtime: ScannerRuntime,
}

impl SstableScanner {
    /// Erzeugt einen neuen `SstableScanner` mit implizit erfasster Laufzeit.
    pub fn new(storage: Arc<LsmStorage>) -> Self {
        Self::with_runtime(storage, ScannerRuntime::capture())
    }

    /// Erzeugt einen neuen `SstableScanner` mit expliziter `ScannerRuntime`.
    pub fn with_runtime(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Self {
        Self { storage, runtime }
    }

    /// Positivkontrolle für SSTables: Erzeugt eine temporäre Canary-Datei,
    /// führt die volle Reader-Iterationslogik darüber aus, verifiziert den Treffer
    /// und entfernt die Canary-Datei rückstandslos.
    fn run_positive_control(&self, session: &mut ScanSession) -> Result<()> {
        let storage = Arc::clone(&self.storage);
        let mut control_matched = false;
        let mut control_unit: Option<[u8; 32]> = None;

        self.runtime.block_on(async {
            let temp_dir = std::env::temp_dir();
            let canary_path = temp_dir.join(format!("canary_{}.sst", uuid::Uuid::new_v4()));
            let canary_key = b"t:canary:1:chunk:1";
            let canary_val = b"canary_val";

            if let Ok(mut builder) = SstableBuilder::create(&canary_path).await {
                if builder.add(canary_key, canary_val, 1, 1).await.is_ok()
                    && builder.finish().await.is_ok()
                {
                    let block_cache = storage.block_cache();
                    if let Ok(reader) =
                        SstableReader::open_with_key_manager(&canary_path, block_cache, None).await
                    {
                        if let Ok(entries) = reader.iter().await {
                            for (k, _v, seq) in entries {
                                if k.as_ref() == canary_key {
                                    let mut hasher = blake3::Hasher::new();
                                    hasher.update(k.as_ref());
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

impl LayerScanner for SstableScanner {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::SsTableAllLevels
    }

    fn scan_units(&self, session: &mut ScanSession, ctx: &RequestContext) -> Result<()> {
        self.run_positive_control(session)?;

        let codec = TenantKeyCodec::new(ctx.tenant_id);
        let target_prefix = codec.scan_prefix().to_vec();

        let storage = Arc::clone(&self.storage);

        let discovered_units = self.runtime.block_on(async {
            let mut units = Vec::new();

            // 1. Offene / Aktive SSTables im Store erfassen
            let readers = storage.sstables_for_scanner().await;
            let mut active_paths = AHashSet::new();

            for reader in &readers {
                active_paths.insert(reader.file_path().to_path_buf());

                // Metadaten- & Bloom-Pre-check vor Block-Scan
                let first_k = reader.first_key();
                let last_k = reader.last_key();

                let may_overlap = first_k.starts_with(&target_prefix)
                    || last_k.starts_with(&target_prefix)
                    || (first_k.as_ref() <= target_prefix.as_slice()
                        && last_k.as_ref() >= target_prefix.as_slice());

                if may_overlap || first_k.is_empty() {
                    if let Ok(entries) = reader.iter().await {
                        for (key, _val, seq_no) in entries {
                            let mut hasher = blake3::Hasher::new();
                            hasher.update(key.as_ref());
                            hasher.update(&seq_no.to_le_bytes());
                            let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();

                            let is_canary = key.starts_with(b"t:canary:")
                                || key
                                    .windows(b":scratch_canary_pctl:".len())
                                    .any(|w| w == b":scratch_canary_pctl:");
                            let is_residual = key.starts_with(&target_prefix) && !is_canary;
                            units.push((unit_hash, is_residual));
                        }
                    }
                }
            }

            // 2. Inventar-Diff: Suche nach Altdateien (.sst), die nicht im Manifest/Store stehen
            let data_dir = storage.data_path();
            if let Ok(mut dir) = tokio::fs::read_dir(data_dir).await {
                while let Ok(Some(entry)) = dir.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("sst")
                        && !active_paths.contains(&path)
                    {
                        // Orphan SSTable gefunden
                        let block_cache = storage.block_cache();
                        if let Ok(orphan_reader) =
                            SstableReader::open_with_key_manager(&path, block_cache, None).await
                        {
                            if let Ok(entries) = orphan_reader.iter().await {
                                for (key, _val, seq_no) in entries {
                                    let mut hasher = blake3::Hasher::new();
                                    hasher.update(key.as_ref());
                                    hasher.update(&seq_no.to_le_bytes());
                                    let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();

                                    let is_canary = key.starts_with(b"t:canary:")
                                        || key
                                            .windows(b":scratch_canary_pctl:".len())
                                            .any(|w| w == b":scratch_canary_pctl:");
                                    let is_residual = key.starts_with(&target_prefix) && !is_canary;
                                    units.push((unit_hash, is_residual));
                                }
                            }
                        } else {
                            // Wenn Datei nicht lesbar ist (z. B. unvollständig), zähle Datei-Hash als Rest
                            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                            let mut hasher = blake3::Hasher::new();
                            hasher.update(file_name.as_bytes());
                            let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();
                            units.push((unit_hash, true));
                        }
                    }
                }
            }

            Ok::<Vec<([u8; 32], bool)>, contextra_types::ContextraError>(units)
        })?;

        for (unit_hash, is_residual) in discovered_units {
            session.record_unit(unit_hash, is_residual);
        }

        Ok(())
    }
}
