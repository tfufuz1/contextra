//! Modul
//!
//! # Zweck & Architektur
//! Komponente des  Subsystems (Ring 4 Adapter-Schicht).
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// FILE-CONTEXT
// ZWECK: Extractor für MS-PPT (Binary PowerPoint Presentation Format) (WP M4).
// INVARIANTEN: Zero-Panic, RecordHeader-Parsen mit Längenprüfung, CryptSession10Container => Encrypted,
//              SlideListWithText, TextCharsAtom (UTF-16LE), TextBytesAtom (Latin-1),
//              Titel über TextHeaderAtom, SpeakerNotes aus NotesContainer.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::formats::ole::cfb_reader::SafeCfbReader;
use crate::ir::{BlockKind, Boundary, DocumentIr, Locator, NoteKind};

/// Record Types nach MS-PPT Spezifikation.
const REC_TYPE_SLIDE_PERSIST_ATOM: u16 = 0x03F0;
const REC_TYPE_SLIDE_CONTAINER: u16 = 0x03F0; // oder 0x03EC
const REC_TYPE_SLIDE_CONTAINER_ALT: u16 = 0x03EC;
const REC_TYPE_NOTES_CONTAINER: u16 = 0x03F8;
const REC_TYPE_SLIDE_LIST_WITH_TEXT: u16 = 0x0FF5;
const REC_TYPE_TEXT_HEADER_ATOM: u16 = 0x0FA8;
const REC_TYPE_TEXT_CHARS_ATOM: u16 = 0x0FA0;
const REC_TYPE_TEXT_BYTES_ATOM: u16 = 0x0FAB;
const REC_TYPE_CRYPT_SESSION10: u16 = 0x2F14;

/// TextHeaderAtom Typen
const TEXT_TYPE_TITLE: u32 = 0;
const TEXT_TYPE_SUBTITLE: u32 = 5;
const TEXT_TYPE_CENTER_TITLE: u32 = 6;

/// Parsed Record Header
#[derive(Debug, Clone, Copy)]
struct RecordHeader {
    ver: u8,
    instance: u16,
    rec_type: u16,
    rec_len: u32,
}

impl RecordHeader {
    fn is_container(&self) -> bool {
        self.ver == 0x0F
    }
}

/// Liest einen Record Header (8 Bytes) ohne Panic.
fn read_record_header(data: &[u8], offset: usize) -> Option<RecordHeader> {
    let slice = data.get(offset..offset.checked_add(8)?)?;
    let ver_inst = u16::from_le_bytes([*slice.first()?, *slice.get(1)?]);
    let rec_type = u16::from_le_bytes([*slice.get(2)?, *slice.get(3)?]);
    let rec_len = u32::from_le_bytes([
        *slice.get(4)?,
        *slice.get(5)?,
        *slice.get(6)?,
        *slice.get(7)?,
    ]);

    let ver = (ver_inst & 0x000F) as u8;
    let instance = ver_inst >> 4;

    Some(RecordHeader {
        ver,
        instance,
        rec_type,
        rec_len,
    })
}

/// Zustands-Tracker während des PPT Record Traversals.
#[derive(Default)]
struct PptState {
    slide_count: u32,
    active_slide_number: Option<u32>,
    current_slide_title: Option<String>,
    pending_text_type: Option<u32>,
    in_notes: bool,
}

/// Extrahiert MS-PPT aus einer OLE/CFB Struktur.
pub fn extract_ppt(
    reader: &mut SafeCfbReader<'_>,
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    let ppt_bytes = reader.read_stream("PowerPoint Document", ctx)?;

    let mut ir = DocumentIr::new(FormatId::Ppt);
    let mut state = PptState::default();

    parse_ppt_records(&ppt_bytes, 0, ppt_bytes.len(), &mut state, &mut ir)?;

    if ir.blocks.is_empty() {
        return Err(IngestError::NoExtractableText {
            format: FormatId::Ppt,
        });
    }

    Ok(ir)
}

/// Liest iterativ/rekursiv Record-Header und verarbeitet Container und Atoms.
fn parse_ppt_records(
    bytes: &[u8],
    start: usize,
    end: usize,
    state: &mut PptState,
    ir: &mut DocumentIr,
) -> Result<(), IngestError> {
    let mut pos = start;

    while pos < end {
        let header = match read_record_header(bytes, pos) {
            Some(h) => h,
            None => break,
        };

        let payload_start = match pos.checked_add(8) {
            Some(p) => p,
            None => break,
        };

        let payload_len = header.rec_len as usize;
        let payload_end = match payload_start.checked_add(payload_len) {
            Some(e) if e <= end => e,
            _ => break,
        };

        if header.rec_type == REC_TYPE_CRYPT_SESSION10 {
            return Err(IngestError::Encrypted {
                format: Some(FormatId::Ppt),
            });
        }

        if header.is_container() {
            if header.rec_type == REC_TYPE_SLIDE_LIST_WITH_TEXT {
                if header.instance == 0 {
                    state.in_notes = false;
                } else if header.instance == 2 {
                    state.in_notes = true;
                }
            } else if header.rec_type == REC_TYPE_SLIDE_CONTAINER
                || header.rec_type == REC_TYPE_SLIDE_CONTAINER_ALT
            {
                state.in_notes = false;
                state.slide_count = state.slide_count.saturating_add(1);
                state.active_slide_number = Some(state.slide_count);
                state.current_slide_title = None;
            } else if header.rec_type == REC_TYPE_NOTES_CONTAINER {
                state.in_notes = true;
            }

            parse_ppt_records(bytes, payload_start, payload_end, state, ir)?;
        } else {
            let payload = bytes.get(payload_start..payload_end).unwrap_or(&[]);
            parse_ppt_atom(header, payload, state, ir);
        }

        pos = payload_end;
    }

    Ok(())
}

/// Verarbeitet einzelne PPT Atoms (Text, Header, Persist).
fn parse_ppt_atom(header: RecordHeader, payload: &[u8], state: &mut PptState, ir: &mut DocumentIr) {
    match header.rec_type {
        REC_TYPE_SLIDE_PERSIST_ATOM => {
            if !state.in_notes {
                state.slide_count = state.slide_count.saturating_add(1);
                state.active_slide_number = Some(state.slide_count);
                state.current_slide_title = None;
            }
        }
        REC_TYPE_TEXT_HEADER_ATOM => {
            if let (Some(&b0), Some(&b1), Some(&b2), Some(&b3)) =
                (payload.first(), payload.get(1), payload.get(2), payload.get(3))
            {
                let text_type = u32::from_le_bytes([b0, b1, b2, b3]);
                state.pending_text_type = Some(text_type);
            }
        }
        REC_TYPE_TEXT_CHARS_ATOM => {
            let u16s: Vec<u16> = payload
                .chunks_exact(2)
                .filter_map(|chunk| match (chunk.first(), chunk.get(1)) {
                    (Some(&b0), Some(&b1)) => Some(u16::from_le_bytes([b0, b1])),
                    _ => None,
                })
                .collect();
            let text = String::from_utf16_lossy(&u16s);
            process_extracted_text(&text, state, ir);
        }
        REC_TYPE_TEXT_BYTES_ATOM => {
            let text: String = payload.iter().map(|&b| b as char).collect();
            process_extracted_text(&text, state, ir);
        }
        _ => {}
    }
}

/// Ordnet extrahierten Text als Folie, Titel oder Speaker Note zu.
fn process_extracted_text(raw_text: &str, state: &mut PptState, ir: &mut DocumentIr) {
    let trimmed = raw_text.trim();
    if trimmed.is_empty() {
        return;
    }

    let text_type = state.pending_text_type.take().unwrap_or(1);

    if state.in_notes {
        let locator = Locator {
            slide: state.active_slide_number,
            ..Default::default()
        };
        ir.push(
            BlockKind::Note {
                kind: NoteKind::SpeakerNotes,
                text: trimmed.to_string(),
            },
            locator,
        );
    } else {
        let is_title = text_type == TEXT_TYPE_TITLE
            || text_type == TEXT_TYPE_SUBTITLE
            || text_type == TEXT_TYPE_CENTER_TITLE;

        let slide_num = state.active_slide_number.unwrap_or_else(|| {
            state.slide_count = state.slide_count.saturating_add(1);
            state.slide_count
        });
        state.active_slide_number = Some(slide_num);

        if is_title && state.current_slide_title.is_none() {
            state.current_slide_title = Some(trimmed.to_string());

            ir.push(
                BlockKind::Boundary(Boundary::Slide {
                    number: slide_num,
                    title: Some(trimmed.to_string()),
                }),
                Locator {
                    slide: Some(slide_num),
                    ..Default::default()
                },
            );
        } else {
            let need_boundary = match ir.blocks.last() {
                Some(block) => !matches!(
                    block.kind,
                    BlockKind::Boundary(Boundary::Slide { number, .. }) if number == slide_num
                ),
                None => true,
            };

            if need_boundary {
                ir.push(
                    BlockKind::Boundary(Boundary::Slide {
                        number: slide_num,
                        title: state.current_slide_title.clone(),
                    }),
                    Locator {
                        slide: Some(slide_num),
                        ..Default::default()
                    },
                );
            }

            ir.push(
                BlockKind::Paragraph {
                    text: trimmed.to_string(),
                },
                Locator {
                    slide: Some(slide_num),
                    ..Default::default()
                },
            );
        }
    }
}
