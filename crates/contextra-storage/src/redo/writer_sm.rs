//! # Sans-I/O Writer-Zustandsmaschine
//!
//! SPEC §9.4, §10.2, §10.5 und §18:
//! Zustandsmaschine für den Redo-Schreibpfad: `step(event) -> Vec<Effect>`.

use crate::redo::frame::Durability;

/// Ereignisse, die der Writer-Zustandsmaschine übergeben werden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriterEvent {
    /// Neue Gruppe von serialisierten Frames soll geschrieben werden.
    WriteGroup {
        /// Serialisierte Frames der Gruppe.
        bytes: Vec<u8>,
        /// Höchster Commit-Stamp in dieser Gruppe.
        last_stamp: u64,
        /// Anzahl neuer Frames in der Gruppe.
        frame_count: u64,
        /// Höchste Durability-Anforderung der Gruppe.
        durability: Durability,
    },
    /// Bestätigung: `Append` war erfolgreich.
    IoAppendSuccess,
    /// Bestätigung: `Sync` war erfolgreich.
    IoSyncSuccess,
    /// Bestätigung: `Truncate` war erfolgreich.
    IoTruncateSuccess,
    /// Fehler: `ENOSPC` oder Kurzschreiben beim `Append`.
    IoAppendEnospc,
    /// Fehler: Sonstiger E/A-Fehler beim `Append`.
    IoAppendError(String),
    /// Fehler: `sync_data` fehlgeschlagen (INV-LOG-5).
    IoSyncError(String),
    /// Fehler: `set_len` fehlgeschlagen.
    IoTruncateError(String),
}

/// Effekte, die vom Sans-I/O-Treiber ausgeführt werden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriterEffect {
    /// Daten an den Dateioffset anhängen.
    Append {
        /// Dateioffset.
        offset: u64,
        /// Anzuhängende Bytes.
        bytes: Vec<u8>,
    },
    /// Dateidaten synchronisieren (sync_data).
    Sync(Durability),
    /// Dateilänge auf guten Stand zurücksetzen (set_len).
    Truncate {
        /// Ziellänge in Bytes.
        len: u64,
    },
    /// Segment am Ende siegeln (Seal-Frame schreiben).
    Seal {
        /// Dateioffset für den Seal-Frame.
        offset: u64,
        /// Serialisierte Bytes des Seal-Frames.
        bytes: Vec<u8>,
    },
    /// Auf das nächste Segment rotieren.
    Rotate,
    /// Fataler Fehler: Subsystem vergiften (INV-LOG-5).
    Fail {
        /// Ursache der Vergiftung.
        cause: String,
    },
    /// Quittung an Gruppe: erfolgreich persistiert.
    GroupCommitted {
        /// Höchster Stamp der Gruppe.
        last_stamp: u64,
    },
    /// Gruppe abweisen: Speicher voll (wiederholbar, `Busy{StorageFull}`).
    GroupStorageFull,
    /// Gruppe abweisen: Subsystem vergiftet.
    GroupPoisoned {
        /// Ursache der Vergiftung.
        cause: String,
    },
}

/// Interner Zustand der Writer-Zustandsmaschine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriterState {
    /// Bereit für die nächste Gruppe oder Rotation.
    Ready,
    /// Schreibt Daten für eine Gruppe.
    WritingGroup {
        /// Vorheriger guter Offset vor dem Schreibvorgang.
        good_offset: u64,
        /// Neuer Offset nach dem Schreiben.
        new_offset: u64,
        /// Höchster Stamp der Gruppe.
        last_stamp: u64,
        /// Neue Frame-Anzahl.
        new_frame_count: u64,
        /// Durability-Anforderung.
        durability: Durability,
    },
    /// Wartet auf Synchronisation nach erfolgreichem Schreiben.
    SyncingGroup {
        /// Höchster Stamp der Gruppe.
        last_stamp: u64,
    },
    /// Schneidet Datei nach ENOSPC auf den letzten guten Offset ab.
    TruncatingAfterEnospc {
        /// Guter Offset.
        good_offset: u64,
    },
    /// Synchronisiert nach dem Abschneiden.
    SyncingAfterTruncate,
    /// Wartet auf Sync des Seal-Frames bei Rotation.
    SyncingSeal {
        /// Guter Offset nach dem Siegel.
        sealed_offset: u64,
    },
    /// Subsystem ist vergiftet (INV-LOG-5).
    Poisoned {
        /// Ursache.
        cause: String,
    },
}

/// Sans-I/O-Zustandsmaschine für den Redo-Log-Writer.
#[derive(Debug)]
pub struct WriterStateMachine {
    /// Aktueller Zustand.
    pub state: WriterState,
    /// Aktueller Dateioffset im Segment.
    pub current_offset: u64,
    /// Bisher geschriebene Frames in diesem Segment.
    pub frame_count: u64,
    /// Letzter geschriebener Commit-Stamp.
    pub last_stamp: u64,
    /// Letzter Kettenprüfwert (für Plain-Modus).
    pub last_chain: u32,
    /// Maximale Segmentgröße in Bytes (Standard 64 MiB).
    pub max_segment_bytes: u64,
    /// Eindeutige Segment-ID.
    pub segment_id: u64,
}

impl WriterStateMachine {
    /// Erstellt eine neue Writer-Zustandsmaschine am Start eines neuen Segments (Offset 48).
    pub fn new(segment_id: u64, chain_seed: u32, max_segment_bytes: u64) -> Self {
        // // INV-LOG-4: Jede Prozessinstanz beginnt ein neues Segment
        Self {
            state: WriterState::Ready,
            current_offset: 48,
            frame_count: 0,
            last_stamp: 0,
            last_chain: chain_seed,
            max_segment_bytes,
            segment_id,
        }
    }

    /// Führt einen Schritt in der Zustandsmaschine aus und liefert die auszuführenden Effekte.
    pub fn step(&mut self, event: WriterEvent) -> Vec<WriterEffect> {
        let mut effects = Vec::new();

        match (&self.state, event) {
            (WriterState::Poisoned { cause }, WriterEvent::WriteGroup { .. }) => {
                effects.push(WriterEffect::GroupPoisoned { cause: cause.clone() });
            }
            (WriterState::Ready, WriterEvent::WriteGroup { bytes, last_stamp, frame_count: count, durability }) => {
                let bytes_len_u64 = match u64::try_from(bytes.len()) {
                    Ok(v) => v,
                    Err(_) => {
                        effects.push(WriterEffect::Fail { cause: "Gruppengröße übersteigt u64".into() });
                        return effects;
                    }
                };

                // Prüfe, ob Gruppe in das Segment passt (inklusive 33 Bytes Reserve für Seal-Frame)
                const SEAL_RESERVE: u64 = 48;
                let needed = bytes_len_u64.saturating_add(SEAL_RESERVE);
                let remaining = self.max_segment_bytes.saturating_sub(self.current_offset);

                if needed > remaining {
                    // Passt nicht in den Rest => siegeln und rotieren (§10.2 Schritt 6, §10.5)
                    let seal_bytes = self.create_seal_frame_bytes();
                    effects.push(WriterEffect::Seal {
                        offset: self.current_offset,
                        bytes: seal_bytes,
                    });
                    effects.push(WriterEffect::Sync(Durability::Always));
                    effects.push(WriterEffect::Rotate);
                    // Die Gruppe wird nach der Rotation im neuen Segment erneut übergeben
                    return effects;
                }

                let good_offset = self.current_offset;
                let new_offset = good_offset.saturating_add(bytes_len_u64);
                let new_frame_count = self.frame_count.saturating_add(count);

                self.state = WriterState::WritingGroup {
                    good_offset,
                    new_offset,
                    last_stamp,
                    new_frame_count,
                    durability,
                };

                effects.push(WriterEffect::Append {
                    offset: good_offset,
                    bytes,
                });
            }
            (
                WriterState::WritingGroup {
                    good_offset: _,
                    new_offset,
                    last_stamp,
                    new_frame_count,
                    durability,
                },
                WriterEvent::IoAppendSuccess,
            ) => {
                let stamp = *last_stamp;
                let count = *new_frame_count;
                let dur = *durability;
                self.current_offset = *new_offset;
                self.frame_count = count;
                self.last_stamp = stamp;

                if dur == Durability::Always {
                    self.state = WriterState::SyncingGroup { last_stamp: stamp };
                    effects.push(WriterEffect::Sync(Durability::Always));
                } else {
                    self.state = WriterState::Ready;
                    effects.push(WriterEffect::GroupCommitted { last_stamp: stamp });
                }
            }
            (
                WriterState::WritingGroup { good_offset, .. },
                WriterEvent::IoAppendEnospc,
            ) => {
                // ENOSPC beim Schreiben: Abschneiden auf den letzten guten Stand (§10.5)
                let target = *good_offset;
                self.state = WriterState::TruncatingAfterEnospc { good_offset: target };
                effects.push(WriterEffect::Truncate { len: target });
            }
            (
                WriterState::WritingGroup { .. },
                WriterEvent::IoAppendError(err),
            ) => {
                // Anderer Schreibfehler (EIO) => Poison (§10.5, INV-LOG-5)
                self.state = WriterState::Poisoned { cause: err.clone() };
                effects.push(WriterEffect::Fail { cause: err.clone() });
                effects.push(WriterEffect::GroupPoisoned { cause: err });
            }
            (
                WriterState::TruncatingAfterEnospc { good_offset },
                WriterEvent::IoTruncateSuccess,
            ) => {
                self.current_offset = *good_offset;
                self.state = WriterState::SyncingAfterTruncate;
                effects.push(WriterEffect::Sync(Durability::Always));
            }
            (
                WriterState::TruncatingAfterEnospc { .. },
                WriterEvent::IoTruncateError(err),
            ) => {
                // Scheitert das Abschneiden => Poison (§10.5, INV-LOG-5)
                self.state = WriterState::Poisoned { cause: err.clone() };
                effects.push(WriterEffect::Fail { cause: err.clone() });
                effects.push(WriterEffect::GroupPoisoned { cause: err });
            }
            (
                WriterState::SyncingAfterTruncate,
                WriterEvent::IoSyncSuccess,
            ) => {
                self.state = WriterState::Ready;
                effects.push(WriterEffect::GroupStorageFull);
            }
            (
                WriterState::SyncingAfterTruncate,
                WriterEvent::IoSyncError(err),
            ) => {
                self.state = WriterState::Poisoned { cause: err.clone() };
                effects.push(WriterEffect::Fail { cause: err.clone() });
                effects.push(WriterEffect::GroupPoisoned { cause: err });
            }
            (
                WriterState::SyncingGroup { last_stamp },
                WriterEvent::IoSyncSuccess,
            ) => {
                let stamp = *last_stamp;
                self.state = WriterState::Ready;
                effects.push(WriterEffect::GroupCommitted { last_stamp: stamp });
            }
            (
                WriterState::SyncingGroup { .. },
                WriterEvent::IoSyncError(err),
            ) => {
                // sync_data scheitert: Poison (INV-LOG-5), nie wiederholen, nichts in die Memtable
                self.state = WriterState::Poisoned { cause: err.clone() };
                effects.push(WriterEffect::Fail { cause: err.clone() });
                effects.push(WriterEffect::GroupPoisoned { cause: err });
            }
            _ => {}
        }

        effects
    }

    fn create_seal_frame_bytes(&self) -> Vec<u8> {
        let (seal_frame, _) = crate::redo::frame::RedoFrame::new_seal_plain(
            self.last_stamp,
            self.frame_count,
            self.last_chain,
            self.last_chain,
        );
        seal_frame.to_bytes()
    }
}
