// FILE-CONTEXT
// ZWECK: Extractor für MS-DOC (Binary Word Document Format) (WP M4).
// INVARIANTEN: Zero-Panic, fEncrypted-Erkennung, Piece-Table (Clx/PlcPcd) Dekodierung,
//              ANSI (Windows-1252) / UTF-16LE, Feldcode-Filterung (0x13..0x14 verwerfen),
//              Haupttext/Fußnoten/Kopfzeilen, Fallback bei beschädigter Piece Table.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::formats::ole::cfb_reader::SafeCfbReader;
use crate::ir::{BlockKind, DocumentIr, ExtractWarning, Locator, NoteKind, WarningCode};

/// Mindestlänge des FIB (File Information Block).
const MIN_FIB_LEN: usize = 0x0040;

/// Bitmasken für FIB Flags (offset 0x000A).
const FIB_FLAG_ENCRYPTED: u16 = 0x0100;
const FIB_FLAG_WHICH_TBL: u16 = 0x0200;

/// Steuerzeichen in MS-DOC CPs.
const CHAR_EMBEDDED_OBJ: char = '\x01';
const CHAR_CELL_END: char = '\x07';
const CHAR_LINE_BREAK: char = '\x0B';
const CHAR_PAGE_BREAK: char = '\x0C';
const CHAR_PARA_END: char = '\x0D';
const CHAR_FIELD_BEGIN: char = '\x13';
const CHAR_FIELD_SEP: char = '\x14';
const CHAR_FIELD_END: char = '\x15';

/// Auslesen von 16-Bit Little-Endian Ganzzahlen ohne Panic.
fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
    let bytes = data.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([*bytes.first()?, *bytes.get(1)?]))
}

/// Auslesen von 32-Bit Little-Endian Ganzzahlen ohne Panic.
fn read_u32_le(data: &[u8], offset: usize) -> Option<u32> {
    let bytes = data.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes([
        *bytes.first()?,
        *bytes.get(1)?,
        *bytes.get(2)?,
        *bytes.get(3)?,
    ]))
}

/// Struktur eines Piece-Table-Eintrags.
#[derive(Debug, Clone)]
struct Piece {
    cp_start: u32,
    cp_end: u32,
    is_compressed: bool,
    file_offset: usize,
}

/// Extrahiert MS-DOC aus einer OLE/CFB Struktur.
pub fn extract_doc(
    reader: &mut SafeCfbReader<'_>,
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    let word_doc_bytes = reader.read_stream("WordDocument", ctx)?;

    if word_doc_bytes.len() < MIN_FIB_LEN {
        return Err(IngestError::Corrupt {
            format: Some(FormatId::Doc),
            detail: "WordDocument-Stream zu kurz für FIB".into(),
        });
    }

    // 1. FIB Header parsen
    let w_ident = read_u16_le(&word_doc_bytes, 0).unwrap_or(0);
    if w_ident != 0xA5EC {
        return Err(IngestError::Corrupt {
            format: Some(FormatId::Doc),
            detail: format!("Ungültiger FIB Ident-Wert: 0x{:04X}", w_ident),
        });
    }

    let flags = read_u16_le(&word_doc_bytes, 0x000A).unwrap_or(0);
    let f_encrypted = (flags & FIB_FLAG_ENCRYPTED) != 0;
    if f_encrypted {
        return Err(IngestError::Encrypted {
            format: Some(FormatId::Doc),
        });
    }

    let f_which_tbl = (flags & FIB_FLAG_WHICH_TBL) != 0;
    let table_stream_name = if f_which_tbl { "1Table" } else { "0Table" };

    // FIB Text-Teilbereichslängen
    let ccp_text = read_u32_le(&word_doc_bytes, 0x004C).unwrap_or(0);
    let ccp_ftn = read_u32_le(&word_doc_bytes, 0x0050).unwrap_or(0);
    let ccp_hdd = read_u32_le(&word_doc_bytes, 0x0054).unwrap_or(0);

    // fcClx & lcbClx Offset aus FibRgFcLcb97
    let (fc_clx, lcb_clx) = if word_doc_bytes.len() >= 0x01AA {
        (
            read_u32_le(&word_doc_bytes, 0x01A2).unwrap_or(0),
            read_u32_le(&word_doc_bytes, 0x01A6).unwrap_or(0),
        )
    } else {
        (0, 0)
    };

    let mut ir = DocumentIr::new(FormatId::Doc);

    // 2. Clx/Piece Table aus Table Stream lesen
    let pieces = if reader.exists(table_stream_name) && lcb_clx > 0 {
        match reader.read_stream(table_stream_name, ctx) {
            Ok(table_bytes) => parse_clx(&table_bytes, fc_clx as usize, lcb_clx as usize),
            Err(_) => None,
        }
    } else {
        None
    };

    if let Some(piece_list) = pieces {
        // Dekodieren des CP-Stream-Texts
        let full_text = decode_pieces(&word_doc_bytes, &piece_list);

        // Aufteilen nach FIB-Teilbereichen
        process_cp_ranges(&full_text, ccp_text, ccp_ftn, ccp_hdd, ctx, &mut ir);
    } else {
        // Fallback-Heuristik: Längste lesbare Zeichenfolgen
        ir.warnings.push(ExtractWarning {
            code: WarningCode::FeatureLimited,
            detail: "Piece Table fehlt oder beschädigt; verwende Fallback-Text-Heuristik".into(),
        });
        fallback_heuristics(&word_doc_bytes, &mut ir);
    }

    if ir.blocks.is_empty() {
        return Err(IngestError::NoExtractableText {
            format: FormatId::Doc,
        });
    }

    Ok(ir)
}

/// Parsen der Clx-Struktur und Extraktion der Piece Table.
fn parse_clx(table_bytes: &[u8], fc_clx: usize, lcb_clx: usize) -> Option<Vec<Piece>> {
    let clx_slice = table_bytes.get(fc_clx..fc_clx.checked_add(lcb_clx)?)?;
    let mut pos = 0usize;

    while pos < clx_slice.len() {
        let clxt = *clx_slice.get(pos)?;
        pos = pos.checked_add(1)?;

        if clxt == 0x01 {
            // PrgPcd / sprm: cbGrpprl (u16) + Payload
            let cb_grpprl = read_u16_le(clx_slice, pos)? as usize;
            pos = pos.checked_add(2)?.checked_add(cb_grpprl)?;
        } else if clxt == 0x02 {
            // PlcPcd!
            let cb_plc_pcd = read_u32_le(clx_slice, pos)? as usize;
            pos = pos.checked_add(4)?;

            let plc_pcd_slice = clx_slice.get(pos..pos.checked_add(cb_plc_pcd)?)?;
            return parse_plc_pcd(plc_pcd_slice);
        } else {
            break;
        }
    }

    None
}

/// Parsen des PlcPcd-Arrays.
fn parse_plc_pcd(slice: &[u8]) -> Option<Vec<Piece>> {
    if slice.len() < 4 {
        return None;
    }

    let n = (slice.len().checked_sub(4)?) / 12;
    if n == 0 {
        return None;
    }

    let mut cp_array = Vec::with_capacity(n.checked_add(1)?);
    for i in 0..=n {
        let cp = read_u32_le(slice, i.checked_mul(4)?)?;
        cp_array.push(cp);
    }

    let pcd_offset_start = (n.checked_add(1)?).checked_mul(4)?;
    let mut pieces = Vec::with_capacity(n);

    for i in 0..n {
        let cp_start = *cp_array.get(i)?;
        let cp_end = *cp_array.get(i.checked_add(1)?)?;
        let pcd_offset = pcd_offset_start.checked_add(i.checked_mul(8)?)?;

        let fc = read_u32_le(slice, pcd_offset.checked_add(2)?)?;
        let is_compressed = (fc & 0x4000_0000) != 0;
        let fc_cleaned = fc & !0x4000_0000;

        let file_offset = if is_compressed {
            (fc_cleaned / 2) as usize
        } else {
            fc_cleaned as usize
        };

        pieces.push(Piece {
            cp_start,
            cp_end,
            is_compressed,
            file_offset,
        });
    }

    Some(pieces)
}

/// Dekodiert Text aus allen Pieces zu einer gesammelten UTF-8/String-CP-Sequenz.
fn decode_pieces(word_doc_bytes: &[u8], pieces: &[Piece]) -> String {
    let mut out = String::new();

    for piece in pieces {
        let len_cp = (piece.cp_end.saturating_sub(piece.cp_start)) as usize;
        if len_cp == 0 {
            continue;
        }

        if piece.is_compressed {
            let end_offset = piece.file_offset.saturating_add(len_cp);
            if let Some(bytes) = word_doc_bytes.get(piece.file_offset..end_offset) {
                for &b in bytes {
                    out.push(windows_1252_to_char(b));
                }
            }
        } else {
            let len_bytes = len_cp.saturating_mul(2);
            let end_offset = piece.file_offset.saturating_add(len_bytes);
            if let Some(bytes) = word_doc_bytes.get(piece.file_offset..end_offset) {
                let u16s: Vec<u16> = bytes
                    .chunks_exact(2)
                    .filter_map(|chunk| match (chunk.first(), chunk.get(1)) {
                        (Some(&b0), Some(&b1)) => Some(u16::from_le_bytes([b0, b1])),
                        _ => None,
                    })
                    .collect();
                out.push_str(&String::from_utf16_lossy(&u16s));
            }
        }
    }

    out
}

/// Einfache Konvertierung von Windows-1252 Byte zu Char.
fn windows_1252_to_char(b: u8) -> char {
    match b {
        0x80 => '€',
        0x82 => '‚',
        0x83 => 'ƒ',
        0x84 => '„',
        0x85 => '…',
        0x86 => '†',
        0x87 => '‡',
        0x88 => 'ˆ',
        0x89 => '‰',
        0x8A => 'Š',
        0x8B => '‹',
        0x8C => 'Œ',
        0x8E => 'Ž',
        0x91 => '‘',
        0x92 => '’',
        0x93 => '“',
        0x94 => '”',
        0x95 => '•',
        0x96 => '–',
        0x97 => '—',
        0x98 => '˜',
        0x99 => '™',
        0x9A => 'š',
        0x9B => '›',
        0x9C => 'œ',
        0x9E => 'ž',
        0x9F => 'Ÿ',
        _ => b as char,
    }
}

/// Verarbeitet CP-Sequenzen in Blöcke nach FIB-Teilbereichen.
fn process_cp_ranges(
    full_text: &str,
    ccp_text: u32,
    ccp_ftn: u32,
    ccp_hdd: u32,
    ctx: &ExtractContext<'_>,
    ir: &mut DocumentIr,
) {
    let chars: Vec<char> = full_text.chars().collect();
    let total_cps = u32::try_from(chars.len()).unwrap_or(u32::MAX);

    let text_end = ccp_text.min(total_cps) as usize;
    let ftn_end = (ccp_text.saturating_add(ccp_ftn)).min(total_cps) as usize;
    let hdd_end =
        (ccp_text.saturating_add(ccp_ftn).saturating_add(ccp_hdd)).min(total_cps) as usize;

    if text_end > 0 {
        if let Some(main_slice) = chars.get(0..text_end) {
            emit_doc_paragraphs(main_slice, None, ir);
        }
    }

    if ftn_end > text_end {
        if let Some(ftn_slice) = chars.get(text_end..ftn_end) {
            emit_doc_paragraphs(ftn_slice, Some(NoteKind::Footnote), ir);
        }
    }

    if hdd_end > ftn_end && ctx.policy.include_headers_footers {
        if let Some(hdd_slice) = chars.get(ftn_end..hdd_end) {
            emit_doc_paragraphs(hdd_slice, Some(NoteKind::Header), ir);
        }
    }
}

/// Wandelt Chars eines Teilbereichs unter Beachtung von Steuerzeichen in Blöcke um.
fn emit_doc_paragraphs(chars: &[char], note_kind: Option<NoteKind>, ir: &mut DocumentIr) {
    let mut current_text = String::new();
    let mut in_field_code = false;

    for &ch in chars {
        match ch {
            CHAR_EMBEDDED_OBJ => {
                ir.warnings.push(ExtractWarning {
                    code: WarningCode::EmbeddedObjectSkipped,
                    detail: "OLE-eingebettetes Objekt im DOC übersprungen".into(),
                });
            }
            CHAR_FIELD_BEGIN => {
                in_field_code = true;
            }
            CHAR_FIELD_SEP => {
                in_field_code = false;
            }
            CHAR_FIELD_END => {
                in_field_code = false;
            }
            CHAR_PARA_END | CHAR_CELL_END | CHAR_PAGE_BREAK => {
                if !in_field_code {
                    let trimmed = current_text.trim();
                    if !trimmed.is_empty() {
                        push_doc_block(trimmed, note_kind, ir);
                        current_text.clear();
                    }
                }
            }
            CHAR_LINE_BREAK => {
                if !in_field_code {
                    current_text.push(' ');
                }
            }
            _ => {
                if !in_field_code {
                    current_text.push(ch);
                }
            }
        }
    }

    let trimmed = current_text.trim();
    if !trimmed.is_empty() {
        push_doc_block(trimmed, note_kind, ir);
    }
}

fn push_doc_block(text: &str, note_kind: Option<NoteKind>, ir: &mut DocumentIr) {
    let kind = match note_kind {
        Some(nk) => BlockKind::Note {
            kind: nk,
            text: text.to_string(),
        },
        None => BlockKind::Paragraph {
            text: text.to_string(),
        },
    };
    ir.push(kind, Locator::default());
}

/// Fallback-Heuristik für beschädigte/fehlende Piece Table.
fn fallback_heuristics(bytes: &[u8], ir: &mut DocumentIr) {
    let mut current = String::new();

    for &b in bytes {
        if b.is_ascii_graphic() || b == b' ' || b >= 0x80 {
            current.push(windows_1252_to_char(b));
        } else if b == b'\r' || b == b'\n' {
            let trimmed = current.trim();
            if trimmed.len() >= 4 {
                ir.push(
                    BlockKind::Paragraph {
                        text: trimmed.to_string(),
                    },
                    Locator::default(),
                );
            }
            current.clear();
        }
    }

    let trimmed = current.trim();
    if trimmed.len() >= 4 {
        ir.push(
            BlockKind::Paragraph {
                text: trimmed.to_string(),
            },
            Locator::default(),
        );
    }
}
