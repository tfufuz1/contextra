//! # Sans-I/O Replay-Zustandsmaschine
//!
//! SPEC §9.4, §13.1 Schritt 6 und §18:
//! Sans-I/O-Zustandsmaschine zur Analyse und Validierung eines Segment-Bytestroms.

use crate::codec::crc32c::crc32c;
use crate::codec::limits::MAX_CHANGE_BYTES;
use crate::redo::frame::{RedoFrame, FLAG_DURABILITY_MASK, FLAG_SEAL};
use crate::redo::header::{RedoHeader, HEADER_SIZE};
use crate::redo::op::RedoOp;

/// Ereignisse, die der Replay-Zustandsmaschine aus dem gelesenen Bytestrom übergeben werden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayEvent<'a> {
    /// Weitere gelesene Bytes eines Segments.
    Chunk(&'a [u8]),
    /// Dateiende (EOF) erreicht.
    Eof,
}

/// Befunde und Aktionen der Replay-Zustandsmaschine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayEffect {
    /// Segmentkopf erfolgreich validiert.
    HeaderParsed(RedoHeader),
    /// Vollständig gültiger Frame erkannt.
    ValidFrame {
        /// Gelesener Frame.
        frame: RedoFrame,
        /// Dekodierte Operationen.
        ops: Vec<RedoOp>,
        /// Dateioffset nach diesem Frame.
        next_offset: u64,
    },
    /// Gültiger Seal-Frame am Segmentende gefunden (INV-LOG-6).
    FoundSeal {
        /// Im Siegel deklarierte Frame-Anzahl.
        frame_count: u64,
        /// Im Siegel deklarierter Kettenabschluss.
        final_chain: u32,
        /// Siegel-Stamp.
        stamp: u64,
        /// Dateioffset nach dem Siegel.
        sealed_offset: u64,
    },
    /// Nullbereich erreicht (vorallokierter Rest, SPEC §9.4, §10.5).
    ZeroTerminated {
        /// Offset, an dem die Nullen beginnen.
        offset: u64,
    },
    /// Unvollständiger / zerrissener Frame am Segmentende (Torn Tail).
    TornTail {
        /// Letzter guter Offset vor dem Abriss.
        good_offset: u64,
        /// Fehlerursache.
        cause: String,
    },
    /// Unzulässiger Defekt mitten im Segment (INV-LOG-6).
    CorruptDefect {
        /// Dateioffset des Defekts.
        offset: u64,
        /// Fehlerursache.
        cause: String,
    },
}

/// Replay-Zustandsmaschine über gelesene Bytes eines Segments.
pub struct ReplayStateMachine {
    /// Bisher gelesener Offset.
    pub current_offset: u64,
    /// Anzahl bisher gelesener regulärer Frames.
    pub frame_count: u64,
    /// Letzter gültiger Commit-Stamp (INV-LOG-3).
    pub last_stamp: u64,
    /// Letzter Prüfsummen-Kettenwert.
    pub last_chain: u32,
    /// Gepufferte Restbytes.
    buffer: Vec<u8>,
    /// Ob der Segmentkopf bereits gelesen wurde.
    header_parsed: bool,
    /// Ob das Segment ordnungsgemäß durch ein Siegel abgeschlossen wurde.
    pub is_sealed: bool,
    /// Ob das Segment beendet ist (Nullen oder Siegel).
    pub is_terminated: bool,
}

impl ReplayStateMachine {
    /// Erstellt eine neue Replay-Zustandsmaschine.
    pub fn new() -> Self {
        Self {
            current_offset: 0,
            frame_count: 0,
            last_stamp: 0,
            last_chain: 0,
            buffer: Vec::new(),
            header_parsed: false,
            is_sealed: false,
            is_terminated: false,
        }
    }

    /// Verarbeitet ein Replay-Ereignis und liefert Effekte.
    pub fn step(&mut self, event: ReplayEvent<'_>) -> Vec<ReplayEffect> {
        let mut effects = Vec::new();

        if self.is_terminated {
            return effects;
        }

        match event {
            ReplayEvent::Chunk(bytes) => {
                self.buffer.extend_from_slice(bytes);
                self.parse_available(&mut effects);
            }
            ReplayEvent::Eof => {
                self.handle_eof(&mut effects);
            }
        }

        effects
    }

    fn parse_available(&mut self, effects: &mut Vec<ReplayEffect>) {
        // 1. Segmentkopf parsen, falls noch nicht geschehen
        if !self.header_parsed {
            if self.buffer.len() < HEADER_SIZE {
                return;
            }
            let header_bytes: [u8; 48] = match self.buffer.get(0..48).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => return,
            };
            match RedoHeader::decode(&header_bytes) {
                Ok(header) => {
                    self.header_parsed = true;
                    self.last_chain = header.chain_seed;
                    self.current_offset = match u64::try_from(HEADER_SIZE) {
                        Ok(v) => v,
                        Err(_) => 48,
                    };
                    self.buffer.drain(0..HEADER_SIZE);
                    effects.push(ReplayEffect::HeaderParsed(header));
                }
                Err(err) => {
                    self.is_terminated = true;
                    effects.push(ReplayEffect::CorruptDefect {
                        offset: 0,
                        cause: err.to_string(),
                    });
                    return;
                }
            }
        }

        // 2. Frames sequentiell parsen
        while !self.is_terminated {
            if self.buffer.len() < 4 {
                // Nicht genug Bytes für len
                return;
            }

            let len_bytes: [u8; 4] = match self.buffer.get(0..4).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => return,
            };
            let len = u32::from_le_bytes(len_bytes);

            // len == 0 beendet das Segment (vorallokierter Nullbereich)
            if len == 0 {
                self.is_terminated = true;
                effects.push(ReplayEffect::ZeroTerminated {
                    offset: self.current_offset,
                });
                return;
            }

            let len_usize = match usize::try_from(len) {
                Ok(v) => v,
                Err(_) => {
                    self.is_terminated = true;
                    effects.push(ReplayEffect::CorruptDefect {
                        offset: self.current_offset,
                        cause: "Frame-Länge übersteigt Adressraum".into(),
                    });
                    return;
                }
            };

            if len_usize < 13 || len_usize > MAX_CHANGE_BYTES {
                self.is_terminated = true;
                effects.push(ReplayEffect::CorruptDefect {
                    offset: self.current_offset,
                    cause: format!("Ungültige Frame-Länge: {len}"),
                });
                return;
            }

            // Für Plain-Modus: 4 Bytes len + len Bytes Daten + 4 Bytes chk
            let total_frame_bytes = len_usize.saturating_add(8);
            if self.buffer.len() < total_frame_bytes {
                // Frame noch nicht vollständig im Puffer
                return;
            }

            let frame_slice = match self.buffer.get(0..total_frame_bytes) {
                Some(s) => s,
                None => return,
            };

            // Prüfe CRC32C über len || stamp || op_count || flags || payload
            let data_to_hash = match frame_slice.get(0..len_usize.saturating_add(4)) {
                Some(s) => s,
                None => return,
            };
            let expected_chk = crc32c(self.last_chain, data_to_hash);

            let chk_bytes: [u8; 4] = match frame_slice.get(len_usize.saturating_add(4)..total_frame_bytes).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => return,
            };
            let stored_chk = u32::from_le_bytes(chk_bytes);

            if stored_chk != expected_chk {
                self.is_terminated = true;
                effects.push(ReplayEffect::TornTail {
                    good_offset: self.current_offset,
                    cause: format!("Prüfsummenfehler im Frame bei Offset {}", self.current_offset),
                });
                return;
            }

            // Header-Felder extrahieren
            let stamp_bytes: [u8; 8] = match frame_slice.get(4..12).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => return,
            };
            let stamp = u64::from_le_bytes(stamp_bytes);

            let op_count_bytes: [u8; 4] = match frame_slice.get(12..16).and_then(|s| s.try_into().ok()) {
                Some(b) => b,
                None => return,
            };
            let op_count = u32::from_le_bytes(op_count_bytes);

            let flags = match frame_slice.get(16) {
                Some(&f) => f,
                None => return,
            };

            if (flags & !(FLAG_DURABILITY_MASK | FLAG_SEAL)) != 0 {
                self.is_terminated = true;
                effects.push(ReplayEffect::CorruptDefect {
                    offset: self.current_offset,
                    cause: format!("Ungültige Frame-Flags: {flags:#04x}"),
                });
                return;
            }

            let payload_bytes = match frame_slice.get(17..len_usize.saturating_add(4)) {
                Some(p) => p.to_vec(),
                None => Vec::new(),
            };

            let frame = RedoFrame {
                len,
                stamp,
                op_count,
                flags,
                payload: payload_bytes,
                chk: chk_bytes.to_vec(),
            };

            let frame_len_u64 = match u64::try_from(total_frame_bytes) {
                Ok(v) => v,
                Err(_) => 0,
            };
            let next_offset = self.current_offset.saturating_add(frame_len_u64);

            if frame.is_seal() {
                // Seal-Frame prüfen (INV-LOG-6)
                let (declared_count, final_chain) = match frame.parse_seal_payload() {
                    Ok(pair) => pair,
                    Err(err) => {
                        self.is_terminated = true;
                        effects.push(ReplayEffect::CorruptDefect {
                            offset: self.current_offset,
                            cause: err.to_string(),
                        });
                        return;
                    }
                };

                // frame_count im Siegel MUSS zur Zahl der gelesenen Frames passen (INV-LOG-6)
                if declared_count != self.frame_count {
                    self.is_terminated = true;
                    effects.push(ReplayEffect::CorruptDefect {
                        offset: self.current_offset,
                        cause: format!(
                            "INV-LOG-6: frame_count im Siegel ({declared_count}) stimmt nicht mit gelesenen Frames ({}) überein",
                            self.frame_count
                        ),
                    });
                    return;
                }

                if final_chain != self.last_chain {
                    self.is_terminated = true;
                    effects.push(ReplayEffect::CorruptDefect {
                        offset: self.current_offset,
                        cause: format!(
                            "INV-LOG-6: final_chain ({final_chain:#010x}) stimmt nicht mit Kette ({:#010x}) überein",
                            self.last_chain
                        ),
                    });
                    return;
                }

                self.is_sealed = true;
                self.is_terminated = true;
                self.current_offset = next_offset;
                self.buffer.drain(0..total_frame_bytes);

                effects.push(ReplayEffect::FoundSeal {
                    frame_count: declared_count,
                    final_chain,
                    stamp,
                    sealed_offset: next_offset,
                });
                return;
            }

            // Normaler Frame: Stamps müssen streng monoton steigen (INV-LOG-3)
            // INV-LOG-3: Stamps im Log streng aufsteigend (nicht lückenlos)
            if self.frame_count > 0 && stamp <= self.last_stamp {
                self.is_terminated = true;
                effects.push(ReplayEffect::CorruptDefect {
                    offset: self.current_offset,
                    cause: format!(
                        "INV-LOG-3: Stamp {stamp} nicht streng größer als vorheriger {}",
                        self.last_stamp
                    ),
                });
                return;
            }

            // Operationen dekodieren
            let mut ops = Vec::new();
            let mut cursor = frame.payload.as_slice();
            let op_count_usize = match usize::try_from(op_count) {
                Ok(v) => v,
                Err(_) => 0,
            };
            for _ in 0..op_count_usize {
                match RedoOp::decode(&mut cursor) {
                    Ok(op) => ops.push(op),
                    Err(err) => {
                        self.is_terminated = true;
                        effects.push(ReplayEffect::CorruptDefect {
                            offset: self.current_offset,
                            cause: err.to_string(),
                        });
                        return;
                    }
                }
            }

            self.frame_count = self.frame_count.saturating_add(1);
            self.last_stamp = stamp;
            self.last_chain = stored_chk;
            self.current_offset = next_offset;
            self.buffer.drain(0..total_frame_bytes);

            effects.push(ReplayEffect::ValidFrame {
                frame,
                ops,
                next_offset,
            });
        }
    }

    fn handle_eof(&mut self, effects: &mut Vec<ReplayEffect>) {
        if self.is_terminated {
            return;
        }

        if !self.buffer.is_empty() {
            // Unvollständige Bytes am Ende der Datei
            effects.push(ReplayEffect::TornTail {
                good_offset: self.current_offset,
                cause: format!("Unerwartetes Dateiende mit {} Restbytes", self.buffer.len()),
            });
        }
        self.is_terminated = true;
    }
}
