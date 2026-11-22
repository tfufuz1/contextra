//! # Redo-Treiber für Dateisystem-E/A
//!
//! SPEC §9.4, §10.2, §10.5 und §18:
//! Führt die Effekte der Sans-I/O-Writer-Zustandsmaschine über `Fs` und `File` aus.

use contextra_storage_ports::{Entropy, File, Fs, OpenMode, Perm};
use crate::redo::error::RedoError;
use crate::redo::frame::{Durability, RedoFrame};
use crate::redo::header::{RedoHeader, HEADER_SIZE};
use crate::redo::op::RedoOp;
use crate::redo::writer_sm::{WriterEffect, WriterEvent, WriterStateMachine};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Standardgröße eines vorallokierten Redo-Segments (64 MiB nach SPEC §9.4, §10.5).
pub const DEFAULT_SEGMENT_BYTES: u64 = 64 * 1024 * 1024;

/// Formatierter Dateiname eines Redo-Segments nach SPEC §9.4: `redo-<segment_id:016x>.log`.
pub fn segment_file_name(segment_id: u64) -> String {
    format!("redo-{:016x}.log", segment_id)
}

/// Formatierter temporärer Dateiname für die Vorallokation: `.building.redo-<segment_id:016x>.log`.
pub fn building_file_name(segment_id: u64) -> String {
    format!(".building.redo-{:016x}.log", segment_id)
}

/// Treiber für das synchrone Redo-Log.
pub struct RedoWriter<F: Fs> {
    fs: Arc<F>,
    base_dir: PathBuf,
    current_segment_id: u64,
    current_file: Box<dyn File>,
    sm: WriterStateMachine,
    prealloc_len: u64,
    is_poisoned: Arc<AtomicBool>,
}

impl<F: Fs> RedoWriter<F> {
    /// Initialisiert einen neuen Redo-Writer für eine neue Prozessinstanz (INV-LOG-4).
    ///
    /// Jede Prozessinstanz beginnt ein neues Segment; es wird nie an ein bestehendes angehängt.
    pub fn open_new(
        fs: Arc<F>,
        base_dir: &Path,
        segment_id: u64,
        prealloc_len: u64,
        entropy: &dyn Entropy,
    ) -> Result<Self, RedoError> {
        // // INV-LOG-4: Jede Prozessinstanz beginnt ein neues Segment; nie anhängen.
        let mut seed_bytes = [0u8; 4];
        entropy.fill(&mut seed_bytes).map_err(|e| RedoError::Io(e.to_string()))?;
        let chain_seed = u32::from_le_bytes(seed_bytes);

        fs.create_dir_all(base_dir, Perm::Private)?;

        let segment_name = segment_file_name(segment_id);
        let segment_path = base_dir.join(&segment_name);

        let file = fs.open(&segment_path, OpenMode::CreateNew(Perm::Private))?;
        file.preallocate(prealloc_len)?;

        let header = RedoHeader::new(segment_id, 0, chain_seed, prealloc_len);
        let header_bytes = header.to_bytes();
        file.write_all_at(&header_bytes, 0)?;
        file.sync_data()?;

        let sm = WriterStateMachine::new(segment_id, chain_seed, prealloc_len);

        Ok(Self {
            fs,
            base_dir: base_dir.to_path_buf(),
            current_segment_id: segment_id,
            current_file: file,
            sm,
            prealloc_len,
            is_poisoned: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Übernimmt ein vorallokiertes Reservesegment (INV-LOG-7).
    pub fn adopt_reserve(
        fs: Arc<F>,
        base_dir: &Path,
        segment_id: u64,
    ) -> Result<Self, RedoError> {
        // // INV-LOG-7: Ein Segment ohne Frames mit gültigem Header und id >= redo_floor ist lebendig (Reserve).
        let segment_name = segment_file_name(segment_id);
        let segment_path = base_dir.join(&segment_name);

        let file = fs.open(&segment_path, OpenMode::ReadWrite)?;
        let mut header_buf = [0u8; 48];
        let bytes_read = file.read_at(&mut header_buf, 0)?;
        if bytes_read < HEADER_SIZE {
            return Err(RedoError::Corrupt("Reserve-Segmentkopf zu kurz".into()));
        }
        let header = RedoHeader::decode(&header_buf)?;

        let sm = WriterStateMachine::new(segment_id, header.chain_seed, header.prealloc_len);

        Ok(Self {
            fs,
            base_dir: base_dir.to_path_buf(),
            current_segment_id: segment_id,
            current_file: file,
            sm,
            prealloc_len: header.prealloc_len,
            is_poisoned: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Allokiert ein Reservesegment im Hintergrund vor (SPEC §10.5, INV-LOG-7).
    pub fn preallocate_reserve(
        fs: &F,
        base_dir: &Path,
        segment_id: u64,
        prealloc_len: u64,
        entropy: &dyn Entropy,
    ) -> Result<(), RedoError> {
        // // INV-LOG-7: Vorallokation eines Reservesegments
        let building_name = building_file_name(segment_id);
        let building_path = base_dir.join(&building_name);
        let target_name = segment_file_name(segment_id);
        let target_path = base_dir.join(&target_name);

        let mut seed_bytes = [0u8; 4];
        entropy.fill(&mut seed_bytes).map_err(|e| RedoError::Io(e.to_string()))?;
        let chain_seed = u32::from_le_bytes(seed_bytes);

        let file = fs.open(&building_path, OpenMode::CreateNew(Perm::Private))?;
        file.preallocate(prealloc_len)?;

        let header = RedoHeader::new(segment_id, 0, chain_seed, prealloc_len);
        let header_bytes = header.to_bytes();
        file.write_all_at(&header_bytes, 0)?;
        file.sync_all()?;

        fs.rename(&building_path, &target_path)?;
        fs.sync_dir(base_dir)?;

        Ok(())
    }

    /// Schreibt eine Gruppe von Operationen als Frame in das Redo-Log.
    pub fn write_change(
        &mut self,
        stamp: u64,
        durability: Durability,
        ops: &[RedoOp],
        next_segment_id: u64,
        entropy: &dyn Entropy,
    ) -> Result<u64, RedoError> {
        // // INV-LOG-1: Ein Frame = ein Change
        if self.is_poisoned.load(Ordering::Acquire) {
            return Err(RedoError::Poisoned {
                cause: "Subsystem ist vergiftet".into(),
            });
        }

        // Frame kodieren
        let (frame, next_chk) = RedoFrame::encode_plain(stamp, durability, ops, self.sm.last_chain)?;
        let frame_bytes = frame.to_bytes();

        let event = WriterEvent::WriteGroup {
            bytes: frame_bytes.clone(),
            last_stamp: stamp,
            frame_count: 1,
            durability,
        };

        let effects = self.sm.step(event);
        let mut committed_stamp = None;

        for effect in effects {
            match effect {
                WriterEffect::Append { offset, bytes } => {
                    self.execute_append(offset, &bytes)?;
                    committed_stamp = Some(stamp);
                }
                WriterEffect::Seal { offset, bytes } => {
                    // Segment voll: siegeln und rotieren (§10.2 Schritt 6, §10.5)
                    self.execute_seal(offset, &bytes)?;
                }
                WriterEffect::Rotate => {
                    self.rotate_to(next_segment_id, entropy)?;
                    // Nach Rotation erneut versuchen
                    return self.write_change(stamp, durability, ops, next_segment_id + 1, entropy);
                }
                WriterEffect::GroupCommitted { last_stamp } => {
                    self.sm.last_chain = next_chk;
                    return Ok(last_stamp);
                }
                WriterEffect::GroupStorageFull => {
                    return Err(RedoError::BusyStorageFull);
                }
                WriterEffect::GroupPoisoned { cause } => {
                    self.mark_poisoned(&cause);
                    return Err(RedoError::Poisoned { cause });
                }
                _ => {}
            }
        }

        self.sm.last_chain = next_chk;
        Ok(committed_stamp.unwrap_or(stamp))
    }

    fn execute_append(&mut self, offset: u64, bytes: &[u8]) -> Result<(), RedoError> {
        let write_res = self.current_file.write_all_at(bytes, offset);

        match write_res {
            Ok(()) => {
                let effects = self.sm.step(WriterEvent::IoAppendSuccess);
                for eff in effects {
                    self.handle_post_append_effect(eff)?;
                }
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::StorageFull => {
                // ENOSPC beim Schreiben: Abschneiden auf den letzten guten Stand (§10.5)
                let effects = self.sm.step(WriterEvent::IoAppendEnospc);
                for eff in effects {
                    if let WriterEffect::Truncate { len } = eff {
                        self.execute_truncate(len)?;
                    }
                }
                Err(RedoError::BusyStorageFull)
            }
            Err(e) => {
                // Anderer Schreibfehler => Poison (INV-LOG-5)
                let cause = e.to_string();
                let effects = self.sm.step(WriterEvent::IoAppendError(cause.clone()));
                self.mark_poisoned(&cause);
                for eff in effects {
                    if let WriterEffect::GroupPoisoned { cause } = eff {
                        return Err(RedoError::Poisoned { cause });
                    }
                }
                Err(RedoError::Poisoned { cause })
            }
        }
    }

    fn execute_truncate(&mut self, len: u64) -> Result<(), RedoError> {
        let trunc_res = self.current_file.set_len(len);
        match trunc_res {
            Ok(()) => {
                let effects = self.sm.step(WriterEvent::IoTruncateSuccess);
                for eff in effects {
                    if let WriterEffect::Sync(_) = eff {
                        let sync_res = self.current_file.sync_data();
                        match sync_res {
                            Ok(()) => {
                                let _ = self.sm.step(WriterEvent::IoSyncSuccess);
                            }
                            Err(e) => {
                                // sync scheitert nach truncate => Poison
                                let cause = e.to_string();
                                self.mark_poisoned(&cause);
                                let _ = self.sm.step(WriterEvent::IoSyncError(cause.clone()));
                                return Err(RedoError::Poisoned { cause });
                            }
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                // Scheitert das Abschneiden => Poison (INV-LOG-5)
                let cause = e.to_string();
                self.mark_poisoned(&cause);
                let _ = self.sm.step(WriterEvent::IoTruncateError(cause.clone()));
                Err(RedoError::Poisoned { cause })
            }
        }
    }

    fn handle_post_append_effect(&mut self, effect: WriterEffect) -> Result<(), RedoError> {
        match effect {
            WriterEffect::Sync(Durability::Always) => {
                let sync_res = self.current_file.sync_data();
                match sync_res {
                    Ok(()) => {
                        let _ = self.sm.step(WriterEvent::IoSyncSuccess);
                        Ok(())
                    }
                    Err(e) => {
                        // sync_data scheitert: Poison (INV-LOG-5), nie wiederholen, nichts in die Memtable
                        let cause = e.to_string();
                        self.mark_poisoned(&cause);
                        let _ = self.sm.step(WriterEvent::IoSyncError(cause.clone()));
                        Err(RedoError::Poisoned { cause })
                    }
                }
            }
            WriterEffect::GroupPoisoned { cause } => {
                self.mark_poisoned(&cause);
                Err(RedoError::Poisoned { cause })
            }
            _ => Ok(()),
        }
    }

    fn execute_seal(&mut self, offset: u64, bytes: &[u8]) -> Result<(), RedoError> {
        // // INV-LOG-6: Beim Rotieren hängt der Leiter einen Seal-Frame an
        self.current_file.write_all_at(bytes, offset)?;
        self.current_file.sync_data()?;
        Ok(())
    }

    fn rotate_to(&mut self, new_segment_id: u64, entropy: &dyn Entropy) -> Result<(), RedoError> {
        // Prüfe zuerst, ob vorallokierte Reserve vorhanden ist (INV-LOG-7)
        let reserve_path = self.base_dir.join(segment_file_name(new_segment_id));
        if self.fs.open(&reserve_path, OpenMode::Read).is_ok() {
            let adopted = Self::adopt_reserve(Arc::clone(&self.fs), &self.base_dir, new_segment_id)?;
            self.current_segment_id = new_segment_id;
            self.current_file = adopted.current_file;
            self.sm = adopted.sm;
            return Ok(());
        }

        // Keine Reserve vorhanden: synchron neu anlegen (SPEC §10.5)
        let new_writer = Self::open_new(
            Arc::clone(&self.fs),
            &self.base_dir,
            new_segment_id,
            self.prealloc_len,
            entropy,
        )?;
        self.current_segment_id = new_segment_id;
        self.current_file = new_writer.current_file;
        self.sm = new_writer.sm;
        Ok(())
    }

    /// Schließt und siegelt das aktuelle Segment ordnungsgemäß ab (INV-LOG-6).
    pub fn seal_and_close(&mut self) -> Result<(), RedoError> {
        let (seal_frame, _) = RedoFrame::new_seal_plain(
            self.sm.last_stamp,
            self.sm.frame_count,
            self.sm.last_chain,
            self.sm.last_chain,
        );
        let seal_bytes = seal_frame.to_bytes();
        self.execute_seal(self.sm.current_offset, &seal_bytes)?;
        let seal_len_u64 = u64::try_from(seal_bytes.len()).unwrap_or(0);
        self.sm.current_offset = self.sm.current_offset.saturating_add(seal_len_u64);
        Ok(())
    }

    fn mark_poisoned(&self, _cause: &str) {
        // // INV-LOG-5: Subsystem wird vergiftet
        self.is_poisoned.store(true, Ordering::Release);
    }

    /// Prüft, ob das Subsystem vergiftet ist.
    pub fn is_poisoned(&self) -> bool {
        self.is_poisoned.load(Ordering::Acquire)
    }

    /// Liefert die aktuelle Segment-ID.
    pub const fn current_segment_id(&self) -> u64 {
        self.current_segment_id
    }
}
